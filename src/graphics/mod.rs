pub mod font;
pub mod scene;

use sdl2::render::{Renderer, Texture};
use sdl2::rect::{Rect, Point};
use sdl2::pixels::{Color, PixelFormatEnum};
use mpd::status::State;
use spectrum::SpectrumResult;
use self::scene::*;
use nalgebra::Vector2;
use nalgebra::Norm;
use core::cmp::Ordering;
use std::mem::replace;
use info::Info;
use std;

const SCENE_TIME: u64 = 20_000;
const TRANSITION_FRAMES: u64 = 4;


pub struct SceneContainer {
    scene: Box<Scene>,
    texture: Texture,
    condition: Box<Fn(&Info) -> bool>
}

impl SceneContainer {
    fn new(scene: Box<Scene>, texture: Texture, condition: Box<Fn(&Info) -> bool>) -> SceneContainer {
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
        Point::new(self.x as i32, self.y as i32)
    }
}

pub struct Graphics {
    frames_in_transition: u64,
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
            frames_in_transition: 0,
            time_in_scene: 0,
            time: time,
            scenes: scenes,
            transition: None,
            current_scene: None
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
            result.push((*min.unwrap_or(target_point), *target_point));
        }
        for origin_point in leftover_origins {
            let min = (&target).iter().min_by(|a, b| {
                let dist_a = (*a - origin_point).norm();
                let dist_b = (*b - origin_point).norm();
                dist_a.partial_cmp(&dist_b).unwrap_or(Ordering::Equal)
            });
            result.push((origin_point, *min.unwrap_or(&Vector2::new(-1.0f32, -1.0f32))));
        }
        result
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
        if (a - b).abs() < std::f32::EPSILON {
            a
        } else if a > b {
            a - 1.0
        } else {
            a + 1.0
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
                if (x - origin.x).abs() > std::f32::EPSILON || (y - origin.y).abs() > std::f32::EPSILON {
                    changed = true;
                }
                if (x - origin.x).abs() > std::f32::EPSILON && (y - origin.y).abs() > std::f32::EPSILON {
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

    fn draw_transition(&mut self, renderer: &mut Renderer) -> Result<(), String> {
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
        if self.frames_in_transition % TRANSITION_FRAMES == 0 {
            self.perform_transition();
        }
        Ok(())
    }

    fn get_pixels_of_scene(mut container: SceneContainer,
            renderer: &mut Renderer,
            info: &Info,
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
            info: &Info,
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
        while !(container.condition)(info) {
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
        self.transition = Some(Graphics::create_transition(old_pixels, new_pixels));
        self.current_scene = Some(swapped_container);
        Ok(())
    }

    fn take_current_scene(
            &mut self,
            renderer: &mut Renderer,
            info: &Info,
            spectrum: &SpectrumResult) -> Option<SceneContainer> {
        if self.current_scene.is_none() {
            self.next_scene(renderer, info, spectrum).expect("Could not switch to next scene.");
        }
        replace(&mut self.current_scene, None)
    }

    fn give_current_scene_back(&mut self, container: SceneContainer) {
        self.current_scene = Some(container);
    }

    fn draw_scene(&mut self, renderer: &mut Renderer, info: &Info, spectrum: &SpectrumResult) -> Result<(), String> {
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

    pub fn draw(&mut self, renderer: &mut Renderer, info: Info, spectrum: SpectrumResult) -> Result<(), String> {
        // Switch scene after timeout
        if info.ms % SCENE_TIME < self.time % SCENE_TIME {
            try!(self.next_scene(renderer, &info, &spectrum));
        }
        let time_delta = info.ms - self.time;
        // Render transition if transition is in progress and else render scene
        let result = if self.transition.is_some() {
            self.frames_in_transition += self.frames_in_transition;
            self.draw_transition(renderer)
        } else {
            self.time_in_scene += time_delta;
            self.draw_scene(renderer, &info, &spectrum)
        };
        self.time = info.ms;
        result
    }
}

#[cfg(test)]
mod tests {
    use sdl2::rect::Rect;
    use nalgebra::Vector2;
    use test_helpers::*;
    use super::*;
    use std;

    #[test]
    fn derasterize_pixels() {
        let mut renderer = create_test_renderer();
        renderer.draw_rect(Rect::new(4, 4, 2, 2)).unwrap();
        let pixels = Graphics::derasterize_pixels(&renderer).unwrap();
        assert_eq!(vec![
                   Vector2::new(4.0, 4.0),
                   Vector2::new(4.0, 5.0),
                   Vector2::new(5.0, 4.0),
                   Vector2::new(5.0, 5.0)
        ], pixels);
    }

    #[test]
    fn create_transition_equal_size() {
        let origin = vec![Vector2::new(1.0, 4.0), Vector2::new(2.0, 4.0), Vector2::new(3.0, 4.0)];
        let target = vec![Vector2::new(1.0, 10.0), Vector2::new(2.0, 10.0), Vector2::new(3.0, 10.0)];
        let result = Graphics::create_transition(origin, target);
        assert_eq!(vec![
                   (Vector2::new(1.0, 4.0), Vector2::new(1.0, 10.0)),
                   (Vector2::new(2.0, 4.0), Vector2::new(2.0, 10.0)),
                   (Vector2::new(3.0, 4.0), Vector2::new(3.0, 10.0))
        ], result)
    }

    #[test]
    fn create_transition_smaller_size() {
        let origin = vec![Vector2::new(1.0, 4.0), Vector2::new(2.0, 4.0)];
        let target = vec![Vector2::new(1.0, 10.0), Vector2::new(2.0, 10.0), Vector2::new(3.0, 10.0)];
        let result = Graphics::create_transition(origin, target);
        assert_eq!(vec![
                   (Vector2::new(1.0, 4.0), Vector2::new(1.0, 10.0)),
                   (Vector2::new(2.0, 4.0), Vector2::new(2.0, 10.0)),
                   (Vector2::new(2.0, 4.0), Vector2::new(3.0, 10.0))
        ], result)
    }

    #[test]
    fn create_transition_bigger_size() {
        let origin = vec![Vector2::new(1.0, 4.0), Vector2::new(2.0, 4.0), Vector2::new(3.0, 4.0)];
        let target = vec![Vector2::new(1.0, 10.0), Vector2::new(2.0, 10.0)];
        let result = Graphics::create_transition(origin, target);
        assert_eq!(vec![
           (Vector2::new(1.0, 4.0), Vector2::new(1.0, 10.0)),
           (Vector2::new(2.0, 4.0), Vector2::new(2.0, 10.0)),
           (Vector2::new(3.0, 4.0), Vector2::new(2.0, 10.0))
        ], result)
    }

    #[test]
    fn approach() {
        assert!(Graphics::approach(5.0, 9.0) - 6.0 < std::f32::EPSILON);
        assert!(Graphics::approach(7.0, 3.0) - 6.0 < std::f32::EPSILON);
        assert!(Graphics::approach(6.0, 6.0) - 6.0 < std::f32::EPSILON);
    }
}
