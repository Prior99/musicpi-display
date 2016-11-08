use sdl2;
use sdl2::event::Event;
use sdl2::EventPump;
use info::Info;
use spectrum::SpectrumResult;
use std::sync::mpsc::Receiver;
use target::{BaseTarget, Target};

pub struct TargetWindow {
    base_target: BaseTarget,
    events: EventPump
}

impl TargetWindow {
    pub fn new(info_receiver: Receiver<Info>,
            spectrum_receiver: Receiver<SpectrumResult>) -> Result<TargetWindow, String> {
        let sdl_context = sdl2::init().unwrap();
        let video = sdl_context.video().unwrap();
        let window = video.window("musicpi-display", 320, 160)
            .build()
            .unwrap();
        let mut renderer = window.renderer().build().unwrap();
        let result = renderer.set_scale(10.0, 10.0);
        if !result.is_ok() {
            return Err(result.err().unwrap());
        }
        let info = info_receiver.recv().unwrap();
        let spectrum = spectrum_receiver.recv().unwrap();
        Ok(TargetWindow {
            events: sdl_context.event_pump().unwrap(),
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

impl Target for TargetWindow {
    fn base_target(&mut self) -> &mut BaseTarget {
        &mut self.base_target
    }

    fn render(&mut self) -> bool {
        for event in self.events.poll_iter() {
            if let Event::Quit {..} = event { return false }
        }
        self.base_target.renderer.present();
        true
    }
}

