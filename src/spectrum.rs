extern crate dft;

use std::sync::mpsc::{SyncSender};
use pulse_simple::Record;
use dft::{Operation, Plan};

const SAMPLE_RATE: u32 = 48000;
const DFT_WINDOW_SIZE: usize = 1024;
const COLUMNS: usize = 32;

#[derive(Clone)]
pub struct SpectrumResult {
    pub spectrum: Vec<f32>,
    pub amplitude: Vec<[f32; 2]>
}

fn analyze(plan: &Plan<f32>, samples: &Vec<f32>) -> Vec<f32> {
    let mut input = samples.clone();
    dft::transform(&mut input, &plan);
    dft::unpack(&input).iter().map(|frequency| {
        frequency.norm() as f32
    }).collect::<Vec<f32>>()
}

fn get_spectrum(mut plan: &mut Plan<f32>, data: &Vec<f32>) -> Vec<f32> {
    let mut max: f32 = 0.0;
    let frequencies = analyze(&mut plan, data);
    let frequencies_per_column = (DFT_WINDOW_SIZE / 2) / COLUMNS;
    let result = (2 .. COLUMNS + 2).map(|column| {
        let start = column * frequencies_per_column;
        let end = (column + 1) * frequencies_per_column;
        let sum: f32 = (start .. end).map(|index| {
            frequencies[index]
        }).sum();
        max = max.max(sum);
        sum
    }).collect::<Vec<f32>>();
    result.iter().map(|value| value / max).collect::<Vec<f32>>()
}

fn update_amplitude(amplitude: &mut Vec<[f32; 2]>, data: &Vec<f32>) {
    let max = data.iter().cloned().fold(0.0, f32::max);
    let min = data.iter().cloned().fold(0.0, f32::min);
    amplitude.remove(0);
    amplitude.push([min, max]);
}

pub fn loop_spectrum(sender: SyncSender<SpectrumResult>) {
    let record = Record::new("MusicPi Display", "Record", None, SAMPLE_RATE);
    let mut stereo_data = (0 .. DFT_WINDOW_SIZE).map(|_| [0.0, 0.0]).collect::<Vec<[f32;2]>>();
    let mut plan = Plan::new(Operation::Forward, DFT_WINDOW_SIZE);
    let mut amplitude = (0 .. COLUMNS).map(|_| [0.0, 0.0]).collect::<Vec<[f32;2]>>();
    loop {
        record.read(&mut stereo_data[..]);
        let mono_data = stereo_data.iter().map(|samples| (samples[0] + samples[1]) / 2.0).collect::<Vec<f32>>();
        let spectrum = get_spectrum(&mut plan, &mono_data);
        update_amplitude(&mut amplitude, &mono_data);
        sender.try_send(SpectrumResult {
            spectrum: spectrum,
            amplitude: amplitude.clone()
        });
    }
}
