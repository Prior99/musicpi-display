mod font;

use sdl2::render::Renderer;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2_image::{self, LoadTexture, INIT_PNG};

use std::path::Path;
use std::time::Instant;

use chrono::{DateTime, Local};

use self::font::FontRenderer;


pub fn create_render(init_renderer: &mut Renderer) -> Box<Fn(&mut Renderer)> {
    let start_time = Instant::now();

    sdl2_image::init(INIT_PNG);
    let font_3x5 = FontRenderer::new(3, 5, init_renderer.load_texture(Path::new("fonts/3x5.png")).unwrap());
    let font_5x7 = FontRenderer::new(5, 7, init_renderer.load_texture(Path::new("fonts/5x7.png")).unwrap());
    let font_7x12 = FontRenderer::new(7, 12, init_renderer.load_texture(Path::new("fonts/7x12.png")).unwrap());

    let render_time = move |renderer: &mut Renderer| {
        let actual_time: DateTime<Local> = Local::now();
        let hours = actual_time.format("%H").to_string();
        let minutes = actual_time.format("%M").to_string();
        font_7x12.text(Point::new(0, 0), &hours, renderer);
        font_7x12.text(Point::new(17, 4), &minutes, renderer);
    };

    Box::new(move |renderer: &mut Renderer| {

        let elapsed = Instant::now().duration_since(start_time);
        let ms = (1_000_000_000 * elapsed.as_secs() + elapsed.subsec_nanos() as u64)/(1_000_000);

        renderer.set_draw_color(Color::RGBA(255, 255, 255, 0));
        renderer.clear();
        renderer.set_draw_color(Color::RGBA(0, 0, 0, 255));
        render_time(renderer);
    })
}
