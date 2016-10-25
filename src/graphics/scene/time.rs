use std::path::Path;
use sdl2::surface::{Surface, SurfaceRef};
use sdl2::render::{Renderer, Texture};
use sdl2::pixels::PixelFormatEnum;
use sdl2_image::LoadTexture;
use sdl2::rect::Point;
use sdl2::pixels::Color;
use graphics::RenderInfo;
use spectrum::SpectrumResult;
use graphics::scene::Scene;
use graphics::font::FontRenderer;

pub struct SceneTime {
    font_7x12: FontRenderer
}

impl SceneTime {
    pub fn new(renderer: &mut Renderer) -> SceneTime {
        let font_7x12 = FontRenderer::new(7, 12, renderer.load_texture(Path::new("assets/7x12.png")).unwrap());
        SceneTime {
            font_7x12: font_7x12
        }
    }
}

impl Scene for SceneTime {
    fn draw(&mut self, renderer: &mut Renderer, info: &RenderInfo, _: &SpectrumResult) {
        renderer.set_draw_color(Color::RGBA(255, 255, 255, 0));
        renderer.clear();
        renderer.set_draw_color(Color::RGBA(0, 0, 0, 255));
        let hours = info.time.format("%H").to_string();
        let minutes = info.time.format("%M").to_string();
        self.font_7x12.text(Point::new(0, 0), &hours, renderer);
        self.font_7x12.text(Point::new(17, 4), &minutes, renderer);
    }
}

