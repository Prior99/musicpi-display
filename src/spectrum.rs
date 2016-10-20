extern crate dft;

use std::sync::mpsc::{Sender};
use pulse_simple::Record;
use dft::{Operation, Plan};

const SAMPLE_RATE: u32 = 16000;
const DFT_WINDOW_SIZE: usize = 512;
const COLUMNS: usize = 32;

fn analyze(plan: &Plan<f32>, samples: Vec<f32>) -> Vec<f32> {
    let mut input = samples.clone();
    dft::transform(&mut input, &plan);
    dft::unpack(&input).iter().map(|frequency| {
        frequency.norm() as f32
    }).collect::<Vec<f32>>()
}

pub fn loop_spectrum(sender: Sender<Vec<f32>>) {
    let record = Record::new("MusicPi Display", "Record", None, SAMPLE_RATE);
    let mut data = (0 .. DFT_WINDOW_SIZE)
        .map(|_| [0.0, 0.0])
        .collect::<Vec<[f32;2]>>();
    let mut plan = Plan::new(Operation::Forward, DFT_WINDOW_SIZE);
    let mut max: f32 = 0.0;

    loop {
        max = max * 0.8;
        record.read(&mut data[..]);
        let frequencies = analyze(&mut plan, data.iter().map(|samples| {
            (samples[0] + samples[1]) / 2.0
        }).collect::<Vec<f32>>());
        let frequencies_per_column = frequencies.len() / COLUMNS;
        let result = (0 .. COLUMNS).map(|column| {
            let start = column * frequencies_per_column;
            let end = (column + 1) * frequencies_per_column;
            let sum: f32 = (start .. end).map(|index| {
                frequencies[index]
            }).sum();
            max = max.max(sum);
            sum
        }).collect::<Vec<f32>>();
        sender.send(result.iter().map(|value| {
            value / max
        }).collect::<Vec<f32>>());
    }
}
