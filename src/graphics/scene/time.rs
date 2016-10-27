use std::path::Path;
use sdl2::render::Renderer;
use sdl2_image::LoadTexture;
use sdl2::rect::Point;
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
    fn draw(&mut self, renderer: &mut Renderer, info: &RenderInfo, _: &SpectrumResult) -> Result<(), String> {
        let hours = info.time.format("%H").to_string();
        let minutes = info.time.format("%M").to_string();
        try!(self.font_7x12.text(Point::new(0, 0), &hours, renderer));
        self.font_7x12.text(Point::new(17, 4), &minutes, renderer)
    }
}

