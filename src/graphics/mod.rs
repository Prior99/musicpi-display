mod font;

use sdl2::render::Renderer;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2_image::{self, LoadTexture, INIT_PNG};
use std::path::Path;
use self::font::FontRenderer;
use mpd::status::Status;
use chrono::{DateTime, Local};

#[derive(Clone)]
pub struct RenderInfo {
    pub volume: i8,
    pub ms: u64,
    pub time: DateTime<Local>,
    pub artist: String,
    pub song: String
}

pub fn create_render(init_renderer: &mut Renderer) -> Box<Fn(&mut Renderer, RenderInfo)> {
    sdl2_image::init(INIT_PNG);
    let font_3x5 = FontRenderer::new(3, 5, init_renderer.load_texture(Path::new("fonts/3x5.png")).unwrap());
    let font_5x7 = FontRenderer::new(5, 7, init_renderer.load_texture(Path::new("fonts/5x7.png")).unwrap());
    let font_7x12 = FontRenderer::new(7, 12, init_renderer.load_texture(Path::new("fonts/7x12.png")).unwrap());

    let render_time = Box::new(move |renderer: &mut Renderer, info: RenderInfo| {
        let hours = info.time.format("%H").to_string();
        let minutes = info.time.format("%M").to_string();
        font_7x12.text(Point::new(0, 0), &hours, renderer);
        font_7x12.text(Point::new(17, 4), &minutes, renderer);
    });

    let render_media = Box::new(move |renderer: &mut Renderer, info: RenderInfo| {
        font_5x7.text(Point::new(0, 0), info.artist.as_str(), renderer);
        font_5x7.text(Point::new(0, 9), info.song.as_str(), renderer);
    });

    let renderers: [Box<Fn(&mut Renderer, RenderInfo)>; 1] = [
        //render_time,
        render_media
    ];

    Box::new(move |renderer: &mut Renderer, info: RenderInfo| {
        renderer.set_draw_color(Color::RGBA(255, 255, 255, 0));
        renderer.clear();
        renderer.set_draw_color(Color::RGBA(0, 0, 0, 255));
        let index = (info.ms / 10_000) as usize % renderers.len();
        let ref render = renderers[index];
        render(renderer, info);
    })
}
