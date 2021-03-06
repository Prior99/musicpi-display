use chrono::{DateTime, Local, Duration};
use std::time::{Instant};
use mpd::Client;
use mpd::status::State;
use std::sync::mpsc::{SyncSender, SendError};
use bus::{BusReader};
use ControlStatus;
use std::thread;

#[derive(Clone)]
pub struct Info {
    pub volume: i8,
    pub ms: u64,
    pub time: DateTime<Local>,
    pub artist: String,
    pub song: String,
    pub duration: Duration,
    pub elapsed: Duration,
    pub state: State
}

fn get_render_info(mpd: &mut Client, start_time: Instant) -> Info {
    let elapsed = Instant::now().duration_since(start_time);
    let ms = (1_000_000_000 * elapsed.as_secs() + elapsed.subsec_nanos() as u64)/(1_000_000) + 2000;
    let actual_time: DateTime<Local> = Local::now();
    let status = mpd.status().unwrap();
    let optional_song = mpd.currentsong().unwrap();
    let (title, artist) = if optional_song.is_some() {
        let song = optional_song.unwrap();
        (song.title.unwrap_or_else(|| String::from("")), song.tags.get("Artist").unwrap_or(&String::from("")).clone())
    } else {
        (String::from(""), String::from(""))
    };
    let (elapsed, duration) = status.time.unwrap_or((Duration::seconds(0), Duration::seconds(0)));
    Info {
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

pub fn run(mut control_rx: BusReader<ControlStatus>, sender: SyncSender<Info>) -> Result<(), SendError<Info>> {
    let mut mpd = Client::connect("127.0.0.1:6600").unwrap();
    let start_time = Instant::now();
    loop {
        let result = sender.send(get_render_info(&mut mpd, start_time));
        if !result.is_ok() {
            return result;
        }
        if let Ok(status) = control_rx.try_recv() {
            if status == ControlStatus::Abort {
                return Ok(())
            }
        }
        thread::yield_now();
    }
}
