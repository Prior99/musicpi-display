pub mod font;
pub mod scene;

use sdl2::render::{Renderer, Texture};
use sdl2::rect::{Rect, Point};
use sdl2::pixels::{Color, PixelFormatEnum};
use chrono::{DateTime, Local, Duration};
use mpd::status::State;
use spectrum::SpectrumResult;
use self::scene::*;

const SCENE_TIME: u64 = 10_000;

#[derive(Clone)]
pub struct RenderInfo {
    pub volume: i8,
    pub ms: u64,
    pub time: DateTime<Local>,
    pub artist: String,
    pub song: String,
    pub duration: Duration,
    pub elapsed: Duration,
    pub state: State
}

pub struct SceneContainer {
    scene: Box<Scene>,
    texture: Texture,
    condition: Box<Fn(&RenderInfo) -> bool>
}

impl SceneContainer {
    fn new(scene: Box<Scene>, texture: Texture, condition: Box<Fn(&RenderInfo) -> bool>) -> SceneContainer {
        SceneContainer {
            texture: texture,
            scene: scene,
            condition: condition
        }
    }
}

fn derasterize_pixels(renderer: &Renderer) -> Result<Vec<Point>, String> {
    let pixels = try!(renderer.read_pixels(None, PixelFormatEnum::RGBA8888));
    let mut result: Vec<Point> = Vec::new();
    for x in 0 .. 32 {
        for y in 0 .. 16 {
            let index = (x + y * 32) * 4;
            if pixels[index] == 255 {
                result.push(Point::new(x as i32, y as i32));
            }
        }
    }
    Ok(result)
}

fn calc_distance(a: &Point, b: &Point) -> f32 {
    let x = a.x() - b.x();
    let y = a.y() - b.y();
    ((x * x + y * y) as f32).sqrt()
}

fn create_transition(origin: Vec<Point>, target: Vec<Point>) -> Vec<(Point, Point)> {
    let mut leftover_origins = origin.clone();
    let mut result: Vec<(Point, Point)> = Vec::new();
    for target_point in &target {
        let mut min_distance: f32 = 100.0f32;
        let mut minimum: Option<Point> = None;
        let mut min_index = 0;
        for (index, origin_point) in (&leftover_origins).iter().enumerate() {
            let distance = calc_distance(&origin_point, &target_point);
            if minimum.is_none() || distance < min_distance {
                minimum = Some(origin_point.clone());
                min_index = index;
                min_distance = distance;
            }
        }
        if minimum.is_some() {
            leftover_origins.retain(|point| point.x() == minimum.unwrap().x() && point.y() == minimum.unwrap().y());
        }
        result.push((minimum.unwrap_or(target_point.clone()), target_point.clone()));
    }
    /*for origin_point in leftover_origins {
        let mut min_distance: f32 = 100.0f32;
        let mut minimum = Point::new(origin_point.x(), -1);
        let mut min_index = 0;
        for (index, target_point) in (&target).iter().enumerate() {
            let distance = calc_distance(&origin_point, &target_point);
            if distance < min_distance {
                minimum = target_point.clone();
                min_index = index;
                min_distance = distance;
            }
        }
        result.push((origin_point.clone(), minimum));
    }*/
    result
}

pub struct Graphics {
    time: u64,
    scenes: Vec<SceneContainer>,
    transition: Option<Vec<(Point, Point)>>
}

fn prepare_texture(renderer: &mut Renderer) -> Texture {
    renderer.create_texture_target(PixelFormatEnum::RGBA8888, 32, 16).unwrap()
}

impl Graphics {
    pub fn new(renderer: &mut Renderer, time: u64) -> Graphics {
        let scenes = vec![
            SceneContainer::new(Box::new(SceneTime::new(renderer)),
                prepare_texture(renderer),
                Box::new(|_| true)),
            SceneContainer::new(Box::new(SceneMedia::new(renderer)),
                prepare_texture(renderer),
                Box::new(|info| info.state != State::Stop)),
            SceneContainer::new(Box::new(SceneSpectrum::new(renderer)),
                prepare_texture(renderer),
                Box::new(|info| info.state == State::Play)),
            SceneContainer::new(Box::new(SceneAmplitude::new(renderer)),
                prepare_texture(renderer),
                Box::new(|info| info.state == State::Play))
        ];
        Graphics {
            time: time,
            scenes: scenes,
            transition: None
        }
    }

    fn approach(a: i32, b: i32) -> i32 {
        if a == b {
            a
        } else {
            if a > b {
                a - 1
            } else {
                a + 1
            }
        }
    }

    fn perform_transition(&mut self) {
        if self.transition.is_none() {
            return;
        }
        let transition = self.transition.clone().unwrap();
        self.transition = Some(transition.iter().map(|&(origin, target)| {
            if origin == target {
                (origin.clone(), target.clone())
            } else {
                let x = Graphics::approach(origin.x(), target.x());
                let y = Graphics::approach(origin.y(), target.y());
                (Point::new(x, y), target.clone())
            }
        }).collect::<Vec<(Point, Point)>>());
    }

    pub fn draw(&mut self, renderer: &mut Renderer, info: RenderInfo, spectrum: SpectrumResult) -> Result<(), String> {
        if self.transition.is_none() {
            self.scenes.pop().unwrap();
            self.scenes.pop().unwrap();
            let mut scene2 = self.scenes.pop().unwrap();
            let mut scene1 = self.scenes.pop().unwrap();
            try!(renderer.render_target().unwrap().set(scene1.texture));
            renderer.set_draw_color(Color::RGBA(255, 255, 255, 0));
            renderer.clear();
            renderer.set_draw_color(Color::RGBA(0, 0, 0, 255));
            try!(scene1.scene.draw(renderer, &info, &spectrum));
            let pixels1 = derasterize_pixels(&renderer).unwrap();
            let origin_texture = renderer.render_target().unwrap().set(scene2.texture).unwrap().unwrap();
            renderer.set_draw_color(Color::RGBA(255, 255, 255, 0));
            renderer.clear();
            renderer.set_draw_color(Color::RGBA(0, 0, 0, 255));
            try!(scene2.scene.draw(renderer, &info, &spectrum));
            let pixels2 = derasterize_pixels(&renderer).unwrap();
            self.transition = Some(create_transition(pixels1, pixels2));
            renderer.render_target().unwrap().reset();
            renderer.set_draw_color(Color::RGBA(255, 255, 255, 0));
            renderer.clear();
            renderer.set_draw_color(Color::RGBA(0, 0, 0, 255));
            renderer.copy(&origin_texture, None, None);

        } else {
            let origins = self.transition.clone()
                .unwrap()
                .iter()
                .map(|&(origin, _)| origin)
                .collect::<Vec<Point>>();
            renderer.render_target().unwrap().reset();
            renderer.set_draw_color(Color::RGBA(255, 255, 255, 0));
            renderer.clear();
            renderer.set_draw_color(Color::RGBA(0, 0, 0, 255));
            renderer.draw_points(&origins);
            self.perform_transition();
        }
        Ok(())
    }
}
