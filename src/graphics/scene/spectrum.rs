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

fn correct(value: f32) -> f32 {
    let log_value = (value * 10000.0).log10() * 4.0;
    if !log_value.is_finite() {
        0.0f32
    } else {
        log_value.min(16.0)
    }
}

impl Scene for SceneSpectrum {


    fn draw(&mut self, renderer: &mut Renderer, _: &RenderInfo, spectrum: &SpectrumResult) -> Result<(), String> {
        let rects = spectrum.spectrum.iter().enumerate().map(|(x, &(min, max))| {
            let corrected_min = correct(min);
            let corrected_max = correct(max);
            let height = corrected_max - corrected_min;
            Rect::new(x as i32, 16 - corrected_min as i32, 1, height as u32)
        }).collect::<Vec<Rect>>();
        renderer.draw_rects(&rects)
    }
}


