use sdl2::render::Renderer;
use sdl2::rect::Rect;
use graphics::RenderInfo;
use spectrum::SpectrumResult;
use graphics::scene::Scene;

pub struct SceneSpectrum { }

impl SceneSpectrum {
    pub fn new(_: &mut Renderer) -> SceneSpectrum {
        SceneSpectrum { }
    }
}

impl Scene for SceneSpectrum {
    fn draw(&mut self, renderer: &mut Renderer, _: &RenderInfo, spectrum: &SpectrumResult) -> Result<(), String> {
        let mut rects = spectrum.spectrum.iter().enumerate().map(|(x, value)| {
            let height = value.min(1.0) * 15.0;
            Rect::new(x as i32, 16 - height as i32, 1, height as u32)
        }).collect::<Vec<Rect>>();
        rects.push(Rect::new(0, 15, 33, 1));
        renderer.draw_rects(&rects)
    }
}


