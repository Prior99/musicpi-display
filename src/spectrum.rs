extern crate dft;

use std::sync::mpsc::{Sender, SendError};
use pulse_simple::Record;
use dft::{Operation, Plan};
use bus::{BusReader};
use ControlStatus;
use std::thread;
use core::cmp::Ordering;

const SAMPLE_RATE: u32 = 48000;
const DFT_WINDOW_SIZE: usize = 2048;
const COLUMNS: usize = 32;

#[derive(Clone)]
pub struct SpectrumResult {
    pub spectrum: Vec<(f32, f32)>,
    pub amplitude: Vec<[f32; 2]>
}

fn analyze(plan: &Plan<f32>, samples: &[f32]) -> Vec<f32> {
    let mut input = samples.to_vec();
    dft::transform(&mut input, plan);
    dft::unpack(&input).iter().map(|frequency| {
        frequency.norm() as f32
    }).collect::<Vec<f32>>()
}

fn get_spectrum(plan: &mut Plan<f32>, data: &[f32], max_volume: &mut f32) -> Vec<(f32, f32)> {
    let frequencies = analyze(plan, data);
    let frequencies_per_column = (DFT_WINDOW_SIZE / 16) / COLUMNS;
    let mut top_freq_volume = 0.0;

    for (i, volume) in frequencies.iter().enumerate() {
        if i > 0 && i < frequencies.len() / 16 && volume >= &top_freq_volume {
            // top_freq = i as f32 * SAMPLE_RATE as f32 / frequencies.len() as f32;
            top_freq_volume = *volume;
        }
    }

    if top_freq_volume > *max_volume {
        *max_volume = top_freq_volume;
    } else if *max_volume > 0.0 {
        *max_volume -= 1.0f32;
    }

    (0 .. COLUMNS).map(|column| {
        let start = column * frequencies_per_column;
        let end = (column + 1) * frequencies_per_column;
        let column_min: f32 = (start .. end)
            .map(|index| frequencies[index])
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .unwrap_or(0.0f32);
        let column_max: f32 = (start .. end)
            .map(|index| frequencies[index])
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .unwrap_or(0.0f32);
        (column_min / *max_volume, column_max / *max_volume)
    }).collect::<Vec<(f32, f32)>>()
}

fn update_amplitude(amplitude: &mut Vec<[f32; 2]>, data: &[f32], max_amplitude: &mut f32) {
    let max = data.iter().cloned().fold(0.0, f32::max);
    let min = data.iter().cloned().fold(0.0, f32::min);
    if max > *max_amplitude {
        *max_amplitude = max;
    } else if *max_amplitude > 0.0 {
        *max_amplitude -= 0.001f32;
    }
    amplitude.remove(0);
    amplitude.push([min / *max_amplitude, max / *max_amplitude]);
}

pub fn run(mut control_rx: BusReader<ControlStatus>, sender: Sender<SpectrumResult>) -> Result<(), SendError<SpectrumResult>> {
    let record = Record::new("MusicPi Display", "Record", None, SAMPLE_RATE);
    let mut stereo_data = (0 .. DFT_WINDOW_SIZE).map(|_| [0.0, 0.0]).collect::<Vec<[f32;2]>>();
    let mut plan = Plan::new(Operation::Forward, DFT_WINDOW_SIZE);
    let mut amplitude = (0 .. COLUMNS).map(|_| [0.0, 0.0]).collect::<Vec<[f32;2]>>();
    let mut max_volume = 0.0;
    let mut max_amplitude = 0.0;
    loop {
        record.read(&mut stereo_data[..]);
        let mono_data = stereo_data.iter().map(|samples| (samples[0] + samples[1]) / 2.0).collect::<Vec<f32>>();
        let spectrum = get_spectrum(&mut plan, &mono_data, &mut max_volume);
        update_amplitude(&mut amplitude, &mono_data, &mut max_amplitude);
        let result = sender.send(SpectrumResult {
            spectrum: spectrum,
            amplitude: amplitude.clone()
        });
        if !result.is_ok() {
            return result;
        }
        if let Ok(status) = control_rx.try_recv() {
            if status == ControlStatus::Abort {
                return Ok(())
            }
        }
        thread::yield_now();
    }
}
