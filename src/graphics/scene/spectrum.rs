use sdl2::surface::{Surface, SurfaceRef};
use sdl2::render::Renderer;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::pixels::Color;
use graphics::RenderInfo;
use spectrum::SpectrumResult;
use graphics::scene::Scene;

pub struct SceneSpectrum { }

impl SceneSpectrum {
    pub fn new(renderer: &mut Renderer) -> SceneSpectrum {
        SceneSpectrum { }
    }
}

impl Scene for SceneSpectrum {
    fn draw(&mut self, renderer: &mut Renderer, _: &RenderInfo, spectrum: &SpectrumResult) {
        renderer.set_draw_color(Color::RGBA(255, 255, 255, 0));
        renderer.clear();
        renderer.set_draw_color(Color::RGBA(0, 0, 0, 255));
        let rects = spectrum.spectrum.iter().enumerate().map(|(x, value)| {
            let height = value.min(1.0) * 15.0;
            Rect::new(x as i32, 15 - height as i32, 1, height as u32)
        }).collect::<Vec<Rect>>();
        renderer.draw_rects(&rects);
        renderer.draw_rect(Rect::new(0, 15, 32, 1));
    }
}


