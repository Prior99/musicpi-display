use sdl2;
use sdl2::event::Event;
use sdl2::render::Renderer;
use sdl2::EventPump;
use graphics::RenderInfo;
use spectrum::SpectrumResult;
use std::sync::mpsc::Receiver;
use target::{BaseTarget, Target};

pub struct TargetWindow {
    base_target: BaseTarget,
    events: EventPump
}

impl TargetWindow {
    pub fn new(info_receiver: Receiver<RenderInfo>, spectrum_receiver: Receiver<SpectrumResult>) -> TargetWindow {
        let sdl_context = sdl2::init().unwrap();
        let video = sdl_context.video().unwrap();
        let window = video.window("musicpi-display", 320, 160)
            .build()
            .unwrap();
        let mut renderer = window.renderer().build().unwrap();
        renderer.set_scale(10.0, 10.0);
        let info = info_receiver.recv().unwrap();
        let spectrum = spectrum_receiver.recv().unwrap();
        TargetWindow {
            events: sdl_context.event_pump().unwrap(),
            base_target: BaseTarget {
                renderer: renderer,
                info: info,
                spectrum: spectrum,
                info_receiver: info_receiver,
                spectrum_receiver: spectrum_receiver
            }
        }
    }

}

impl Target for TargetWindow {
    fn base_target(&mut self) -> &mut BaseTarget {
        &mut self.base_target
    }

    fn render(&mut self) {
        self.base_target.renderer.present();
    }
}

