extern crate spidev;
extern crate sdl2;
extern crate sdl2_image;
extern crate chrono;
extern crate mpd;
#[macro_use]
extern crate clap;
extern crate pulse_simple;
extern crate dft;

mod graphics;
mod info;
mod spectrum;
pub mod display;

use sdl2::surface::Surface;
use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Renderer;
use sdl2::EventPump;
use std::slice::from_raw_parts;
use std::{thread, time};
use std::sync::mpsc::{sync_channel, Receiver};
use clap::{App};

use display::Display;
use graphics::RenderInfo;
use info::loop_info;
use spectrum::loop_spectrum;

fn update_display(renderer: &Renderer, display: &mut Display) {
    let pixels = unsafe { from_raw_parts((*renderer.surface().unwrap().raw()).pixels as *const u32, 32 * 16) };
    let display_data = pixels.into_iter()
        .map(|pixel| *pixel & 0xFF == 0xFFu32)
        .collect::<Vec<_>>();
    display.display(&display_data).unwrap();
}

struct BaseRenderer {
    renderer: Renderer<'static>,
    info: RenderInfo,
    spectrum: Vec<f32>,
    info_receiver: Receiver<RenderInfo>,
    spectrum_receiver: Receiver<Vec<f32>>
}

trait RenderTarget {
    fn render_loop(& mut self) {
        let render = graphics::create_render(&mut self.get_base_renderer().renderer);
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
                render(&mut base_renderer.renderer, base_renderer.info.clone(), base_renderer.spectrum.clone());
            }
            self.render();
            thread::sleep(time::Duration::from_millis(1000/60));
        }
    }

    fn get_base_renderer(& mut self) -> & mut BaseRenderer;

    fn render(&mut self);
}

struct DisplayRenderer {
    display: Display,
    base_renderer: BaseRenderer
}

impl DisplayRenderer {
    fn new(info_receiver: Receiver<RenderInfo>, spectrum_receiver: Receiver<Vec<f32>>) -> DisplayRenderer {
        let surface = Surface::new(32, 16, PixelFormatEnum::RGBA8888).unwrap();
        let renderer = Renderer::from_surface(surface).unwrap();
        let mut display = Display::new(4, 2).unwrap();
        display.clear().unwrap();
        display.set_intensity(1).unwrap();
        let info = info_receiver.recv().unwrap();
        let spectrum = spectrum_receiver.recv().unwrap();
        DisplayRenderer {
            display: display,
            base_renderer: BaseRenderer {
                renderer: renderer,
                info: info,
                spectrum: spectrum,
                info_receiver: info_receiver,
                spectrum_receiver: spectrum_receiver
            }
        }
    }

}

impl RenderTarget for DisplayRenderer {
    fn get_base_renderer(&mut self) -> &mut BaseRenderer {
        &mut self.base_renderer
    }

    fn render(&mut self) {
        update_display(&self.base_renderer.renderer, &mut self.display);
    }
}

struct WindowRenderer {
    base_renderer: BaseRenderer,
    events: EventPump
}

impl WindowRenderer {
    fn new(info_receiver: Receiver<RenderInfo>, spectrum_receiver: Receiver<Vec<f32>>) -> WindowRenderer {
        let sdl_context = sdl2::init().unwrap();
        let video = sdl_context.video().unwrap();
        let window = video.window("musicpi-display", 320, 160)
            .build()
            .unwrap();
        let mut renderer = window.renderer().build().unwrap();
        renderer.set_scale(10.0, 10.0);
        let info = info_receiver.recv().unwrap();
        let spectrum = spectrum_receiver.recv().unwrap();
        WindowRenderer {
            events: sdl_context.event_pump().unwrap(),
            base_renderer: BaseRenderer {
                renderer: renderer,
                info: info,
                spectrum: spectrum,
                info_receiver: info_receiver,
                spectrum_receiver: spectrum_receiver
            }
        }
    }

}

impl RenderTarget for WindowRenderer {
    fn get_base_renderer(&mut self) -> &mut BaseRenderer {
        &mut self.base_renderer
    }

    fn render(&mut self) {
        self.base_renderer.renderer.present();
    }
}

fn main() {
    let yaml = load_yaml!("commandline.yml");
    let arguments = App::from_yaml(yaml).get_matches();
    let use_display = !arguments.is_present("window");
    let (info_sender, info_receiver) = sync_channel(0);
    let (spectrum_sender, spectrum_receiver) = sync_channel(0);
    let render_thread = thread::spawn(move || {
        if use_display {
            let mut renderer = DisplayRenderer::new(info_receiver, spectrum_receiver);
            renderer.render_loop();
        } else {
            let mut renderer = WindowRenderer::new(info_receiver, spectrum_receiver);
            renderer.render_loop();
        }
    });
    let update_thread = thread::spawn(move || {
        loop_info(info_sender);
    });
    let spectrum_thread = thread::spawn(move || {
        loop_spectrum(spectrum_sender);
    });
    render_thread.join();
    update_thread.join();
    spectrum_thread.join();
}

