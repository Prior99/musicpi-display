use graphics::RenderInfo;
use chrono::{DateTime, Local};
use std::time::Instant;
use mpd::Client;

pub fn get_render_info(mpd: &mut Client, start_time: Instant) -> RenderInfo {
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
    RenderInfo {
        volume: status.volume,
        ms: ms,
        time: actual_time,
        song: title,
        artist: artist
    }
}
