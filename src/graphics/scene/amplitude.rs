use sdl2::rect::Rect;
use sdl2::render::Renderer;
use sdl2::pixels::Color;
use info::Info;
use spectrum::SpectrumResult;
use graphics::scene::Scene;

pub struct SceneAmplitude { }

impl SceneAmplitude {
    pub fn new(_: &mut Renderer) -> SceneAmplitude {
        SceneAmplitude { }
    }
}

impl Scene for SceneAmplitude {
    fn draw(&mut self, renderer: &mut Renderer, _: &Info, spectrum: &SpectrumResult, _: u64) -> Result<(), String> {
        renderer.set_draw_color(Color::RGBA(255, 255, 255, 0));
        renderer.clear();
        renderer.set_draw_color(Color::RGBA(0, 0, 0, 255));
        let points = spectrum.amplitude.iter().enumerate().flat_map(|(x, value)| {
            let height = value[1] * 15.0;
            vec![Rect::new(x as i32, 16i32 - height.max(0.0f32) as i32, 1, height as u32)]
        }).collect::<Vec<Rect>>();
        renderer.draw_rects(&points)
    }
}
