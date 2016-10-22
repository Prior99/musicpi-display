extern crate dft;

use std::sync::mpsc::{SyncSender};
use pulse_simple::Record;
use dft::{Operation, Plan};

const SAMPLE_RATE: u32 = 48000;
const DFT_WINDOW_SIZE: usize = 1024;
const COLUMNS: usize = 32;

fn analyze(plan: &Plan<f32>, samples: Vec<f32>) -> Vec<f32> {
    let mut input = samples.clone();
    dft::transform(&mut input, &plan);
    dft::unpack(&input).iter().map(|frequency| {
        frequency.norm() as f32
    }).collect::<Vec<f32>>()
}

pub fn loop_spectrum(sender: SyncSender<Vec<f32>>) {
    let record = Record::new("MusicPi Display", "Record", None, SAMPLE_RATE);
    let mut data = (0 .. DFT_WINDOW_SIZE)
        .map(|_| [0.0, 0.0])
        .collect::<Vec<[f32;2]>>();
    let mut plan = Plan::new(Operation::Forward, DFT_WINDOW_SIZE);

    loop {
        let mut max: f32 = 0.0;
        record.read(&mut data[..]);
        let frequencies = analyze(&mut plan, data.iter().map(|samples| {
            (samples[0] + samples[1]) / 2.0
        }).collect::<Vec<f32>>());
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
        sender.try_send(result.iter().map(|value| {
            value / max
        }).collect::<Vec<f32>>());
    }
}
