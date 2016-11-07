#![cfg(test)]
use sdl2::surface::Surface;
use sdl2::render::Renderer;
use sdl2::pixels::{PixelFormatEnum, Color};

pub fn create_test_renderer() -> Renderer<'static> {
    let surface = Surface::new(32, 16, PixelFormatEnum::RGBA8888).unwrap();
    let mut renderer = Renderer::from_surface(surface).unwrap();
    renderer.set_draw_color(Color::RGBA(255, 255, 255, 0));
    renderer.clear();
    renderer.set_draw_color(Color::RGBA(0, 0, 0, 255));
    renderer
}
