use sdl2::render::Renderer;
use sdl2::rect::Rect;
use info::Info;
use spectrum::SpectrumResult;
use graphics::scene::Scene;

pub struct SceneSpectrum { }

impl SceneSpectrum {
    pub fn new(_: &mut Renderer) -> SceneSpectrum {
        SceneSpectrum { }
    }
}

impl Scene for SceneSpectrum {
    fn draw(&mut self,
            renderer: &mut Renderer,
            _: &Info,
            spectrum: &SpectrumResult,
            _: u64) -> Result<(), String> {
        let rects = spectrum.spectrum.iter().enumerate().map(|(x, &(_, max))| {
            let value = max * 15.0;
            Rect::new(x as i32, 16i32 - value.max(0.0f32) as i32, 1, value as u32)
        }).collect::<Vec<Rect>>();
        renderer.draw_rects(&rects)
    }
}


