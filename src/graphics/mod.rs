mod font;

use sdl2::render::Renderer;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2_image::{self, LoadTexture, INIT_PNG};
use std::path::Path;
use self::font::FontRenderer;
use chrono::{DateTime, Local, Duration};
use mpd::status::State;

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

fn marquee(font: &FontRenderer, text: &str, start: &Point, ms: u64, renderer: &mut Renderer) {
    let full_width = (font.width as i32 + 1) * text.len() as i32;
    let time_index = (ms / 50) as i32;
    let x = 32 - time_index % (full_width + 32);
    let point = start.offset(x as i32, 0);
    font.text(point, text, renderer);
}

pub fn create_render(init_renderer: &mut Renderer) -> Box<Fn(&mut Renderer, RenderInfo, Vec<f32>)> {
    sdl2_image::init(INIT_PNG);

    let spinner = init_renderer.load_texture(Path::new("assets/spinner.png")).unwrap();
    let playback_state = init_renderer.load_texture(Path::new("assets/playback-state.png")).unwrap();
    let font_3x5 = FontRenderer::new(3, 5, init_renderer.load_texture(Path::new("assets/3x5.png")).unwrap());
    let font_5x7 = FontRenderer::new(5, 7, init_renderer.load_texture(Path::new("assets/5x7.png")).unwrap());
    let font_7x12 = FontRenderer::new(7, 12, init_renderer.load_texture(Path::new("assets/7x12.png")).unwrap());

    let render_time = Box::new(move |renderer: &mut Renderer, info: RenderInfo, spectrum: &Vec<f32>| {
        let hours = info.time.format("%H").to_string();
        let minutes = info.time.format("%M").to_string();
        font_7x12.text(Point::new(0, 0), &hours, renderer);
        font_7x12.text(Point::new(17, 4), &minutes, renderer);
    });

    const SPINNER_FRAMES: i32 = 32;
    const STATE_SIZE: u32 = 5;
    const SPINNER_SIZE: u32 = 9;

    let render_media = Box::new(move |renderer: &mut Renderer, info: RenderInfo, spectrum: &Vec<f32>| {
        marquee(&font_3x5, format!("{} - {}", info.artist, info.song).as_str(), &Point::new(0, 11), info.ms, renderer);
        let elapsed = info.elapsed.num_milliseconds() / 100;
        let duration = info.duration.num_milliseconds() / 100;
        let progress = elapsed as f32 / duration as f32;
        let pixels = (progress * SPINNER_FRAMES as f32) as i32;
        let start = (info.ms as i32 / 100) % SPINNER_FRAMES;
        let state_frame = match info.state {
            State::Play => 0,
            State::Pause => 1,
            State::Stop => 2
        };
        for i in 0 .. pixels {
            let frame = (start + i) % SPINNER_FRAMES;
            let src_pos = Point::new(frame * SPINNER_SIZE as i32, 0 as i32);
            let dest_pos = Point::new(11, 0);
            renderer.copy(
                &spinner,
                Some(Rect::new(src_pos.x(), src_pos.y(), SPINNER_SIZE, SPINNER_SIZE)),
                Some(Rect::new(dest_pos.x(), dest_pos.y(), SPINNER_SIZE, SPINNER_SIZE))
            );
        }
        renderer.copy(
            &playback_state,
            Some(Rect::new(state_frame * STATE_SIZE as i32, 0, STATE_SIZE, STATE_SIZE)),
            Some(Rect::new(13, 2, STATE_SIZE, STATE_SIZE))
        );
    });

    let render_spectrum = Box::new(move |renderer: &mut Renderer, info: RenderInfo, spectrum: &Vec<f32>| {
        let rects = spectrum.iter().enumerate().map(|(x, value)| {
            let height = value * 16.0;
            Rect::new(x as i32, 15 - height as i32, 1, height as u32)
        }).collect::<Vec<Rect>>();
        renderer.draw_rects(&rects);
        renderer.draw_rect(Rect::new(0, 15, 32, 1));
    });

    let renderers: [Box<Fn(&mut Renderer, RenderInfo, &Vec<f32>)>; 2] = [
        render_media,
        render_spectrum
    ];

    Box::new(move |renderer: &mut Renderer, info: RenderInfo, spectrum: Vec<f32>| {
        renderer.set_draw_color(Color::RGBA(255, 255, 255, 0));
        renderer.clear();
        renderer.set_draw_color(Color::RGBA(0, 0, 0, 255));
        let index = (info.ms / 10_000) as usize % renderers.len();
        if info.state == State::Stop {
            render_time(renderer, info, &spectrum);
        } else {
            let ref render = renderers[index];
            render(renderer, info, &spectrum);
        }
    })
}
