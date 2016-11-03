use sdl2::rect::Point;
use sdl2::render::Renderer;
use sdl2::pixels::Color;
use graphics::RenderInfo;
use spectrum::SpectrumResult;
use graphics::scene::Scene;

pub struct SceneAmplitude { }

impl SceneAmplitude {
    pub fn new(_: &mut Renderer) -> SceneAmplitude {
        SceneAmplitude { }
    }
}

impl Scene for SceneAmplitude {
    fn draw(&mut self, renderer: &mut Renderer, _: &RenderInfo, spectrum: &SpectrumResult, _: u64) -> Result<(), String> {
        renderer.set_draw_color(Color::RGBA(255, 255, 255, 0));
        renderer.clear();
        renderer.set_draw_color(Color::RGBA(0, 0, 0, 255));
        let points = spectrum.amplitude.iter().enumerate().flat_map(|(x, value)| {
            let height_min = value[0] * -7.0;
            let height_max = value[1] * 8.0;
            vec![Point::new(x as i32, height_min as i32 + 8), Point::new(x as i32, 8 - height_max as i32)]
        }).collect::<Vec<Point>>();
        renderer.draw_points(&points)
    }
}


