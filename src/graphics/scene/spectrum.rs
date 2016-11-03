use sdl2::render::Renderer;
use sdl2::rect::Point;
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
    fn draw(&mut self,
            renderer: &mut Renderer,
            _: &RenderInfo,
            spectrum: &SpectrumResult,
            _: u64) -> Result<(), String> {
        let rects = spectrum.spectrum.iter().enumerate().map(|(x, &(min, max))| {
            let corrected_min = correct(min);
            let corrected_max = correct(max);
            Point::new(x as i32, (16.0 - (corrected_max + corrected_min) / 2.0) as i32)
        }).collect::<Vec<Point>>();
        renderer.draw_points(&rects)
    }
}


