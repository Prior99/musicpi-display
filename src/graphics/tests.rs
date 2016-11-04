use sdl2::surface::Surface;
use sdl2::render::Renderer;
use sdl2::pixels::{PixelFormatEnum, Color};
use sdl2::rect::Rect;
use nalgebra::Vector2;
use super::*;

fn create_renderer() -> Renderer<'static> {
    let surface = Surface::new(32, 16, PixelFormatEnum::RGBA8888).unwrap();
    let mut renderer = Renderer::from_surface(surface).unwrap();
    renderer.set_draw_color(Color::RGBA(255, 255, 255, 0));
    renderer.clear();
    renderer.set_draw_color(Color::RGBA(0, 0, 0, 255));
    renderer
}

#[test]
fn derasterize_pixels() {
    let mut renderer = create_renderer();
    renderer.draw_rect(Rect::new(4, 4, 2, 2)).unwrap();
    let pixels = Graphics::derasterize_pixels(&mut renderer).unwrap();
    assert_eq!(vec![
        Vector2::new(4.0, 4.0),
        Vector2::new(4.0, 5.0),
        Vector2::new(5.0, 4.0),
        Vector2::new(5.0, 5.0)
    ], pixels);
}

#[test]
fn create_transition_equal_size() {
    let origin = vec![Vector2::new(1.0, 4.0), Vector2::new(2.0, 4.0), Vector2::new(3.0, 4.0)];
    let target = vec![Vector2::new(1.0, 10.0), Vector2::new(2.0, 10.0), Vector2::new(3.0, 10.0)];
    let result = Graphics::create_transition(origin, target);
    assert_eq!(vec![
        (Vector2::new(1.0, 4.0), Vector2::new(1.0, 10.0)),
        (Vector2::new(2.0, 4.0), Vector2::new(2.0, 10.0)),
        (Vector2::new(3.0, 4.0), Vector2::new(3.0, 10.0))
    ], result)
}

#[test]
fn create_transition_smaller_size() {
    let origin = vec![Vector2::new(1.0, 4.0), Vector2::new(2.0, 4.0)];
    let target = vec![Vector2::new(1.0, 10.0), Vector2::new(2.0, 10.0), Vector2::new(3.0, 10.0)];
    let result = Graphics::create_transition(origin, target);
    assert_eq!(vec![
        (Vector2::new(1.0, 4.0), Vector2::new(1.0, 10.0)),
        (Vector2::new(2.0, 4.0), Vector2::new(2.0, 10.0)),
        (Vector2::new(2.0, 4.0), Vector2::new(3.0, 10.0))
    ], result)
}

#[test]
fn create_transition_bigger_size() {
    let origin = vec![Vector2::new(1.0, 4.0), Vector2::new(2.0, 4.0), Vector2::new(3.0, 4.0)];
    let target = vec![Vector2::new(1.0, 10.0), Vector2::new(2.0, 10.0)];
    let result = Graphics::create_transition(origin, target);
    assert_eq!(vec![
        (Vector2::new(1.0, 4.0), Vector2::new(1.0, 10.0)),
        (Vector2::new(2.0, 4.0), Vector2::new(2.0, 10.0)),
        (Vector2::new(3.0, 4.0), Vector2::new(2.0, 10.0))
    ], result)
}

#[test]
fn approach() {
    assert_eq!(6.0, Graphics::approach(5.0, 9.0));
    assert_eq!(6.0, Graphics::approach(7.0, 3.0));
    assert_eq!(6.0, Graphics::approach(6.0, 6.0));
}
