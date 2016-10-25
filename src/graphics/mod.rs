mod font;
mod scene;

use sdl2::render::{Renderer, Texture};
use sdl2::rect::Rect;
use sdl2::pixels::{Color, PixelFormatEnum};
use chrono::{DateTime, Local, Duration};
use mpd::status::State;
use spectrum::SpectrumResult;
use self::scene::*;

#[derive(Clone)]
pub struct RenderInfo {
    pub volume: i8,
    pub ms: u64,
    pub time: DateTime<Local>,
    pub artist: String,
    pub song: String,
    pub duration: Duration,
    pub elapsed: Duration,
    pub state: State
}

fn prepare_texture(renderer: &mut Renderer) -> Texture {
    renderer.create_texture_target(PixelFormatEnum::RGBA8888, 32, 16).unwrap()
}

pub fn draw(renderer: &mut Renderer, info: RenderInfo, spectrum: SpectrumResult) {
    renderer.set_draw_color(Color::RGBA(255, 255, 255, 0));
    renderer.clear();
    let mut scenes: Vec<(Box<Scene>, Texture)> = Vec::new();
    scenes.push((Box::new(SceneTime::new(renderer)), prepare_texture(renderer)));
    if info.state == State::Pause || info.state == State::Play {
        scenes.push((Box::new(SceneMedia::new(renderer)), prepare_texture(renderer)));
    }
    if info.state == State::Play {
        scenes.push((Box::new(SceneSpectrum::new(renderer)), prepare_texture(renderer)));
        scenes.push((Box::new(SceneAmplitude::new(renderer)), prepare_texture(renderer)));
    }
    let (mut scene, texture) = scenes.pop().unwrap();
    renderer.render_target().unwrap().set(texture);
    scene.draw(renderer, &info, &spectrum);
    let updated_scene_texture = renderer.render_target().unwrap().reset().unwrap().unwrap();
    renderer.copy(&updated_scene_texture, Some(Rect::new(0, 0, 32, 16)), Some(Rect::new(0, 0, 32, 16)));
    if info.ms % 5000 == 0 {
        scenes.insert(0, (scene, updated_scene_texture));
    } else {
        scenes.push((scene, updated_scene_texture));
    }
}
