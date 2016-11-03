use std::path::Path;
use sdl2::render::Renderer;
use sdl2_image::LoadTexture;
use sdl2::rect::Point;
use graphics::RenderInfo;
use spectrum::SpectrumResult;
use graphics::scene::Scene;
use graphics::font::FontRenderer;
use chrono::Timelike;

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
    fn draw(&mut self, renderer: &mut Renderer, info: &RenderInfo, _: &SpectrumResult, _: u64) -> Result<(), String> {
        let hours = info.time.format("%H").to_string();
        let minutes = info.time.format("%M").to_string();
        let second = info.time.second() as f32;
        let bar1_width: i32 = (15.0 * ((second -  0.0).max(0.0) / 15.0).min(1.0)) as i32;
        let bar2_width: i32 = (15.0 * ((second - 15.0).max(0.0) / 15.0).min(1.0)) as i32;
        let bar3_width: i32 = (15.0 * ((second - 30.0).max(0.0) / 15.0).min(1.0)) as i32;
        let bar4_width: i32 = (15.0 * ((second - 45.0).max(0.0) / 15.0).min(1.0)) as i32;
        let mut points = (0 .. bar1_width).map(|x| {
            let y = 14 + x % 2;
            Point::new(x, y)
        }).collect::<Vec<Point>>();
        points.extend((0 .. bar2_width).map(|x| {
            let y = x % 2;
            Point::new(17 + x, y)
        }));
        points.extend((0 .. bar3_width).map(|x| {
            let y = 14 + (x + 1) % 2;
            Point::new(x, y)
        }));
        points.extend((0 .. bar4_width).map(|x| {
            let y = (x + 1) % 2;
            Point::new(17 + x, y)
        }));
        try!(renderer.draw_points(&points));
        try!(self.font_7x12.text(Point::new(0, 0), &hours, renderer));
        self.font_7x12.text(Point::new(17, 4), &minutes, renderer)
    }
}

