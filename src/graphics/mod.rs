mod font;
mod scene;

use sdl2::render::{Renderer, Texture};
use sdl2::rect::Rect;
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

pub struct Graphics {
    time: u64,
    scenes: Vec<SceneContainer>
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
                Box::new(|info| info.state == State::Play)),
        ];
        Graphics {
            time: time,
            scenes: scenes
        }
    }

    pub fn draw(&mut self, renderer: &mut Renderer, info: RenderInfo, spectrum: SpectrumResult) {
        renderer.set_draw_color(Color::RGBA(255, 255, 255, 0));
        renderer.clear();
        let mut container = self.scenes.pop().unwrap();
        while !(container.condition)(&info) {
            self.scenes.insert(0, container);
            container = self.scenes.pop().unwrap();
        }
        renderer.render_target().unwrap().set(container.texture);
        container.scene.draw(renderer, &info, &spectrum);
        let updated_scene_texture = renderer.render_target().unwrap().reset().unwrap().unwrap();
        renderer.copy(&updated_scene_texture, Some(Rect::new(0, 0, 32, 16)), Some(Rect::new(0, 0, 32, 16)));
        let new_container = SceneContainer::new(container.scene, updated_scene_texture, container.condition);
        if info.ms % SCENE_TIME < self.time % SCENE_TIME {
            self.scenes.insert(0, new_container);
        } else {
            self.scenes.push(new_container);
        }
        self.time = info.ms;
    }
}
