pub mod font;
pub mod scene;

use sdl2::render::{Renderer, Texture};
use sdl2::rect::{Rect, Point};
use sdl2::pixels::{Color, PixelFormatEnum};
use chrono::{DateTime, Local, Duration};
use mpd::status::State;
use spectrum::SpectrumResult;
use self::scene::*;
use nalgebra::Vector2;
use nalgebra::Norm;
use core::cmp::Ordering;
use std::mem::replace;

const SCENE_TIME: u64 = 5_000;
const TRANSITION_FRAME_DURATION: u64 = 10;

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


trait ToSdlPoint {
    fn to_sdl(&self) -> Point;
}

impl ToSdlPoint for Vector2<f32> {
    fn to_sdl(&self) -> Point {
        return Point::new(self.x as i32, self.y as i32);
    }
}

fn create_transition(origin: Vec<Vector2<f32>>, target: Vec<Vector2<f32>>) -> Vec<(Vector2<f32>, Vector2<f32>)> {
    let mut leftover_origins = origin.clone();
    let mut result: Vec<(Vector2<f32>, Vector2<f32>)> = Vec::new();
    for target_point in &target {
        let min = (&origin).iter().min_by(|a, b| {
            let dist_a = (*a - target_point).norm();
            let dist_b = (*b - target_point).norm();
            dist_a.partial_cmp(&dist_b).unwrap_or(Ordering::Equal)
        });
        if min.is_some() {
            leftover_origins.retain(|point| point != min.unwrap());
        }
        result.push((min.unwrap_or(target_point).clone(), target_point.clone()));
    }
    for origin_point in leftover_origins {
        let min = (&target).iter().min_by(|a, b| {
            let dist_a = (*a - origin_point).norm();
            let dist_b = (*b - origin_point).norm();
            dist_a.partial_cmp(&dist_b).unwrap_or(Ordering::Equal)
        });
        result.push((origin_point.clone(), min.unwrap_or(&Vector2::new(-1.0f32, -1.0f32)).clone()));
    }
    result
}

pub struct Graphics {
    time_in_transition: u64,
    time_in_scene: u64,
    time: u64,
    scenes: Vec<SceneContainer>,
    current_scene: Option<SceneContainer>,
    transition: Option<Vec<(Vector2<f32>, Vector2<f32>)>>
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
            time_in_transition: 0,
            time_in_scene: 0,
            time: time,
            scenes: scenes,
            transition: None,
            current_scene: None
        }
    }

    fn derasterize_pixels(renderer: &Renderer) -> Result<Vec<Vector2<f32>>, String> {
        let pixels = try!(renderer.read_pixels(None, PixelFormatEnum::RGBA8888));
        let mut result: Vec<Vector2<f32>> = Vec::new();
        for x in 0 .. 32 {
            for y in 0 .. 16 {
                let index = (x + y * 32) * 4;
                if pixels[index] == 255 {
                    result.push(Vector2::new(x as f32, y as f32));
                }
            }
        }
        Ok(result)
    }

    fn approach(a: f32, b: f32) -> f32 {
        if a == b {
            a
        } else {
            if a > b {
                a - 1.0f32
            } else {
                a + 1.0f32
            }
        }
    }

    fn perform_transition(&mut self) {
        if self.transition.is_none() {
            return;
        }
        let transition = self.transition.clone().unwrap();
        let mut changed = false;
        let new_transition = transition.iter().map(|&(origin, target)| {
            if origin == target {
                (origin, target)
            } else {
                let mut x = Graphics::approach(origin.x, target.x);
                let mut y = Graphics::approach(origin.y, target.y);
                if x != origin.x || y != origin.y {
                    changed = true;
                }
                if x != origin.x && y != origin.y {
                    if self.time % 2 == 1 {
                        x = origin.x;
                    } else {
                        y = origin.y;
                    }
                }
                (Vector2::new(x, y), target)
            }
        }).collect::<Vec<(Vector2<f32>, Vector2<f32>)>>();
        self.transition = if changed {
            Some(new_transition)
        } else {
            None
        };
    }

    fn draw_transition(&mut self, renderer: &mut Renderer, info: &RenderInfo) -> Result<(), String> {
        renderer.set_draw_color(Color::RGBA(255, 255, 255, 0));
        renderer.clear();
        let origins = self.transition.clone()
            .unwrap()
            .iter()
            .map(|&(origin, _)| origin.to_sdl())
            .collect::<Vec<Point>>();
        try!(renderer.render_target().unwrap().reset());
        renderer.set_draw_color(Color::RGBA(0, 0, 0, 255));
        try!(renderer.draw_points(&origins));
        if info.ms % TRANSITION_FRAME_DURATION < self.time % TRANSITION_FRAME_DURATION {
            self.perform_transition();
        }
        Ok(())
    }

    fn get_pixels_of_scene(mut container: SceneContainer,
            renderer: &mut Renderer,
            info: &RenderInfo,
            spectrum: &SpectrumResult,
            time: u64) -> Result<(SceneContainer, Vec<Vector2<f32>>), String> {
        try!(renderer.render_target().unwrap().set(container.texture));
        // Clear the texture
        renderer.set_draw_color(Color::RGBA(255, 255, 255, 0));
        renderer.clear();
        renderer.set_draw_color(Color::RGBA(0, 0, 0, 255));
        // Draw current scene once, to get an up-to-date version of the pixels 
        try!(container.scene.draw(renderer, info, spectrum, time));
        let pixels = try!(Graphics::derasterize_pixels(renderer));
        let new_texture = renderer.render_target().unwrap().reset().unwrap().unwrap();
        Ok((SceneContainer::new(container.scene, new_texture, container.condition), pixels))
    }

    fn next_scene(
            &mut self,
            renderer: &mut Renderer,
            info: &RenderInfo,
            spectrum: &SpectrumResult) -> Result<(), String> {
        // Return old scene into front of queue and grep derasterized pixels of it
        let old_pixels = if self.current_scene.is_some() {
            let scene = replace(&mut self.current_scene, None);
            let (swapped_scene, pixels) = Graphics::get_pixels_of_scene(scene.unwrap(),
                renderer,
                info,
                spectrum,
                self.time_in_scene).expect("Unabled to read pixels from scene.");
            self.scenes.insert(0, swapped_scene);
            pixels
        } else {
            Vec::new()
        };
        // Take buffers from top and return them to front if condition not matching
        // until a scene with a matching condition was found
        let mut container = self.scenes.pop().unwrap();
        while !(container.condition)(&info) {
            self.scenes.insert(0, container);
            container = self.scenes.pop().unwrap();
        }
        // Grab derasterized pixels of new scene
        let (swapped_container, new_pixels) = Graphics::get_pixels_of_scene(
            container,
            renderer,
            info,
            spectrum,
            self.time_in_scene).expect("Error when reading pixels from scene.");
        // Store that one as current scene
        self.transition = Some(create_transition(old_pixels, new_pixels));
        self.current_scene = Some(swapped_container);
        Ok(())
    }

    fn take_current_scene(
            &mut self,
            renderer: &mut Renderer,
            info: &RenderInfo,
            spectrum: &SpectrumResult) -> Option<SceneContainer> {
        if self.current_scene.is_none() {
            self.next_scene(renderer, info, spectrum).expect("Could not switch to next scene.");
        }
        replace(&mut self.current_scene, None)
    }

    fn give_current_scene_back(&mut self, container: SceneContainer) {
        self.current_scene = Some(container);
    }

    fn draw_scene(&mut self, renderer: &mut Renderer, info: &RenderInfo, spectrum: &SpectrumResult) -> Result<(), String> {
        let mut container = self.take_current_scene(renderer, info, spectrum).unwrap();
        renderer.set_draw_color(Color::RGBA(255, 255, 255, 0));
        // Clear window texture
        renderer.clear();
        try!(renderer.render_target().unwrap().set(container.texture));
        // Clear scene texture
        renderer.clear();
        renderer.set_draw_color(Color::RGBA(0, 0, 0, 255));
        // Draw current scene 
        try!(container.scene.draw(renderer, info, spectrum, self.time_in_scene));
        // Reset texture back to wondow texture
        let updated_scene_texture = renderer.render_target().unwrap().reset().unwrap().unwrap();
        // Render the scene texture onto the window texture
        try!(renderer.copy(&updated_scene_texture, Some(Rect::new(0, 0, 32, 16)), Some(Rect::new(0, 0, 32, 16))));
        self.give_current_scene_back(SceneContainer::new(container.scene, updated_scene_texture, container.condition));
        Ok(())
    }

    pub fn draw(&mut self, renderer: &mut Renderer, info: RenderInfo, spectrum: SpectrumResult) -> Result<(), String> {
        // Switch scene after timeout
        if info.ms % SCENE_TIME < self.time % SCENE_TIME {
            try!(self.next_scene(renderer, &info, &spectrum));
        }
        let time_delta = info.ms - self.time;
        // Render transition if transition is in progress and else render scene
        let result = if self.transition.is_some() {
            self.time_in_transition += time_delta;
            self.draw_transition(renderer, &info)
        } else {
            self.time_in_scene += time_delta;
            self.draw_scene(renderer, &info, &spectrum)
        };
        self.time = info.ms;
        result
    }
}
