pub mod display;
pub mod window;

use graphics::{draw, RenderInfo};
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

pub trait Target {
    fn run(& mut self) {
        sdl2_image::init(INIT_PNG);
        loop {
            {
                let mut base_renderer = self.get_base_renderer();
                let result = base_renderer.info_receiver.try_recv();
                if result.is_ok() {
                    base_renderer.info = result.unwrap();
                }
                let spectrum_result = base_renderer.spectrum_receiver.recv();
                if spectrum_result.is_ok() {
                    base_renderer.spectrum = spectrum_result.unwrap();
                }
                draw(&mut base_renderer.renderer, base_renderer.info.clone(), base_renderer.spectrum.clone());
            }
            print!("render ... ");
            self.render();
            println!("done.");
            thread::sleep(time::Duration::from_millis(1000/60));
        }
    }

    fn get_base_renderer(&mut self) -> &mut BaseTarget;

    fn render(&mut self);
}

