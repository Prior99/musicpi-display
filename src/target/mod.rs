pub mod display;
pub mod window;

use graphics::{Graphics, RenderInfo};
use spectrum::SpectrumResult;
use sdl2::render::Renderer;
use std::sync::mpsc::Receiver;
use std::thread;
use sdl2_image::{self, INIT_PNG};
use bus::BusReader;
use ControlStatus;
use std::time::{SystemTime, Duration};

const MILLISECONDS_PER_FRAME: u64 = 1000/60;

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
}

pub trait Target {
    fn run(& mut self, mut control_rx: BusReader<ControlStatus>) -> Result<(), String> {
        try!(sdl2_image::init(INIT_PNG));
        let mut graphics = {
            let base_target = self.base_target();
            let time = base_target.info.ms;
            let renderer = base_target.renderer();
            Graphics::new(renderer, time)
        };
        'a: loop {
            let begin = SystemTime::now();
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
                try!(graphics.draw(&mut base_target.renderer, base_target.info.clone(), base_target.spectrum.clone()));
            }
            if !self.render() {
                break 'a;
            }
            match control_rx.try_recv() {
                Ok(status) => {
                    if status == ControlStatus::Abort {
                        break 'a;
                    }
                }
                _ => {}
            }
            let elapsed = SystemTime::now().duration_since(begin).expect("System time error occured.");
            let desired_duration = Duration::from_millis(MILLISECONDS_PER_FRAME);
            if elapsed < desired_duration {
                let sleep_time = desired_duration - elapsed;
                thread::sleep(Duration::from_millis(200));
            } else {
                println!("Warning, rendering took too long: {:.3}ms", elapsed.subsec_nanos() / 1_000_000);
            }
        }
        Ok(())
    }

    fn base_target(&mut self) -> &mut BaseTarget;

    fn render(&mut self) -> bool;
}

