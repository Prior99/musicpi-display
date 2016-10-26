pub mod display;
pub mod window;

use graphics::{Graphics, RenderInfo};
use spectrum::SpectrumResult;
use sdl2::render::Renderer;
use std::sync::mpsc::Receiver;
use std::{thread, time};
use sdl2_image::{self, INIT_PNG};

pub struct BaseTarget {
    renderer: Renderer<'static>,
    info: RenderInfo,
    spectrum: SpectrumResult,
    info_receiver: Receiver<RenderInfo>,
    spectrum_receiver: Receiver<SpectrumResult>
}

impl BaseTarget {
    pub fn renderer(&mut self) -> &mut Renderer<'static> {
        &mut self.renderer
    }

    pub fn info(&self) -> &RenderInfo {
        &self.info
    } 

    pub fn spectrum(&self) -> &SpectrumResult {
        &self.spectrum
    } 
}

pub trait Target {
    fn run(& mut self) {
        sdl2_image::init(INIT_PNG);
        let mut graphics = {
            let base_target = self.base_target();
            let time = base_target.info().ms;
            let renderer = base_target.renderer();
            Graphics::new(renderer, time)
        };
        loop {
            {
                let mut base_target = self.base_target();
                let result = base_target.info_receiver.try_recv();
                if result.is_ok() {
                    base_target.info = result.unwrap();
                }
                let spectrum_result = base_target.spectrum_receiver.try_recv();
                if spectrum_result.is_ok() {
                    base_target.spectrum = spectrum_result.unwrap();
                }
                graphics.draw(&mut base_target.renderer, base_target.info.clone(), base_target.spectrum.clone());
            }
            self.render();
            thread::sleep(time::Duration::from_millis(1000/60));
        }
    }

    fn base_target(&mut self) -> &mut BaseTarget;

    fn render(&mut self);
}

