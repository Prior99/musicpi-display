pub mod amplitude;
pub mod media;
pub mod spectrum;
pub mod time;

pub use self::amplitude::SceneAmplitude;
pub use self::media::SceneMedia;
pub use self::spectrum::SceneSpectrum;
pub use self::time::SceneTime;

use sdl2::render::Renderer;
use info::Info;
use spectrum::SpectrumResult;

pub trait Scene {
    fn draw(&mut self,
            renderer: &mut Renderer,
            info: &Info,
            spectrum: &SpectrumResult,
            time: u64) -> Result<(), String>;
}
