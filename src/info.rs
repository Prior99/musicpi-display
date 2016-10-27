use graphics::RenderInfo;
use chrono::{DateTime, Local, Duration};
use std::time::{Instant};
use mpd::Client;
use std::sync::mpsc::{SyncSender, SendError};
use bus::{BusReader};
use ControlStatus;
use std::thread;

fn get_render_info(mpd: &mut Client, start_time: Instant) -> RenderInfo {
    let elapsed = Instant::now().duration_since(start_time);
    let ms = (1_000_000_000 * elapsed.as_secs() + elapsed.subsec_nanos() as u64)/(1_000_000);
    let actual_time: DateTime<Local> = Local::now();
    let status = mpd.status().unwrap();
    let optional_song = mpd.currentsong().unwrap();
    let (title, artist) = if optional_song.is_some() {
        let song = optional_song.unwrap();
        (song.title.unwrap_or(String::from("")), song.tags.get("Artist").unwrap_or(&String::from("")).clone())
    } else {
        (String::from(""), String::from(""))
    };
    let (elapsed, duration) = status.time.unwrap_or((Duration::seconds(0), Duration::seconds(0)));
    RenderInfo {
        volume: status.volume,
        ms: ms,
        time: actual_time,
        song: title,
        artist: artist,
        duration: duration,
        elapsed: elapsed,
        state: status.state
    }
}

pub fn run(mut control_rx: BusReader<ControlStatus>, sender: SyncSender<RenderInfo>) -> Result<(), SendError<RenderInfo>> {
    let mut mpd = Client::connect("127.0.0.1:6600").unwrap();
    let start_time = Instant::now();
    loop {
        let result = sender.send(get_render_info(&mut mpd, start_time));
        if !result.is_ok() {
            return result;
        }
        match control_rx.try_recv() {
            Ok(status) => {
                if status == ControlStatus::Abort {
                    return Ok(())
                }
            }
            _ => {}
        }
        thread::yield_now();
    }
}
