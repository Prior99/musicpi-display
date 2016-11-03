use std::path::Path;
use mpd::status::State;
use sdl2::render::{Renderer, Texture};
use sdl2_image::LoadTexture;
use sdl2::rect::{Point, Rect};
use graphics::RenderInfo;
use spectrum::SpectrumResult;
use graphics::scene::Scene;
use graphics::font::FontRenderer;

const SPINNER_FRAMES: i32 = 32;
const STATE_SIZE: u32 = 5;
const SPINNER_SIZE: u32 = 9;

pub struct SceneMedia {
    font_3x5: FontRenderer,
    spinner: Texture,
    playback_state: Texture
}

impl SceneMedia {
    pub fn new(renderer: &mut Renderer) -> SceneMedia {
        let font_3x5 = FontRenderer::new(3, 5, renderer.load_texture(Path::new("assets/3x5.png")).unwrap());
        let spinner = renderer.load_texture(Path::new("assets/spinner.png")).unwrap();
        let playback_state = renderer.load_texture(Path::new("assets/playback-state.png")).unwrap();
        SceneMedia {
            font_3x5: font_3x5,
            spinner: spinner,
            playback_state: playback_state
        }
    }
}

impl Scene for SceneMedia {
    fn draw(&mut self,
            renderer: &mut Renderer,
            info: &RenderInfo,
            _: &SpectrumResult,
            time: u64) -> Result<(), String> {
        let media_text = format!("{} - {}", info.artist, info.song);
        try!(self.font_3x5.marquee(media_text.as_str(), &Point::new(0, 11), time, renderer));
        let elapsed = info.elapsed.num_milliseconds() / 100;
        let duration = info.duration.num_milliseconds() / 100;
        let progress = elapsed as f32 / duration as f32;
        let pixels = (progress * SPINNER_FRAMES as f32) as i32;
        let start = (time as i32 / 100) % SPINNER_FRAMES;
        let state_frame = match info.state {
            State::Play => 0,
            State::Pause => 1,
            State::Stop => 2
        };
        for i in 0 .. pixels {
            let frame = (start + i) % SPINNER_FRAMES;
            let src_pos = Point::new(frame * SPINNER_SIZE as i32, 0 as i32);
            let dest_pos = Point::new(11, 0);
            try!(renderer.copy(
                &self.spinner,
                Some(Rect::new(src_pos.x(), src_pos.y(), SPINNER_SIZE, SPINNER_SIZE)),
                Some(Rect::new(dest_pos.x(), dest_pos.y(), SPINNER_SIZE, SPINNER_SIZE))
            ));
        }
        renderer.copy(
            &self.playback_state,
            Some(Rect::new(state_frame * STATE_SIZE as i32, 0, STATE_SIZE, STATE_SIZE)),
            Some(Rect::new(13, 2, STATE_SIZE, STATE_SIZE))
        )
    }
}
