use sdl2::surface::Surface;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Renderer;
use display::Display;
use graphics::RenderInfo;
use spectrum::SpectrumResult;
use std::sync::mpsc::Receiver;
use std::slice::from_raw_parts;
use target::{BaseTarget, Target};

fn update_display(renderer: &Renderer, display: &mut Display) {
    let pixels = unsafe { from_raw_parts((*renderer.surface().unwrap().raw()).pixels as *const u32, 32 * 16) };
    let display_data = pixels.into_iter()
        .map(|pixel| *pixel & 0xFF == 0xFFu32)
        .collect::<Vec<_>>();
    display.display(&display_data).unwrap();
}

pub struct TargetDisplay {
    display: Display,
    base_target: BaseTarget
}

impl TargetDisplay {
    pub fn new(info_receiver: Receiver<RenderInfo>,
            spectrum_receiver: Receiver<SpectrumResult>) -> Result<TargetDisplay, String> {
        let surface = Surface::new(32, 16, PixelFormatEnum::RGBA8888).unwrap();
        let renderer = Renderer::from_surface(surface).unwrap();
        let mut display = Display::new(4, 2).unwrap();
        display.clear().unwrap();
        display.set_intensity(1).unwrap();
        let info = info_receiver.recv().unwrap();
        let spectrum = spectrum_receiver.recv().unwrap();
        Ok(TargetDisplay {
            display: display,
            base_target: BaseTarget {
                renderer: renderer,
                info: info,
                spectrum: spectrum,
                info_receiver: info_receiver,
                spectrum_receiver: spectrum_receiver
            }
        })
    }

}

impl Target for TargetDisplay {
    fn base_target(&mut self) -> &mut BaseTarget {
        &mut self.base_target
    }

    fn render(&mut self) -> bool {
        update_display(&self.base_target.renderer, &mut self.display);
        true
    }
}


