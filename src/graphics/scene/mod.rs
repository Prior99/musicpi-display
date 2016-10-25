mod amplitude;
mod media;
mod spectrum;
mod time;

pub use self::amplitude::SceneAmplitude;
pub use self::media::SceneMedia;
pub use self::spectrum::SceneSpectrum;
pub use self::time::SceneTime;

use sdl2::surface::{Surface, SurfaceRef};
use sdl2::render::{Renderer, Texture};
use graphics::RenderInfo;
use spectrum::SpectrumResult;

pub trait Scene {
    fn draw(&mut self, renderer: &mut Renderer, info: &RenderInfo, spectrum: &SpectrumResult);
}
