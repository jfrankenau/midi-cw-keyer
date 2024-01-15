use std::f32::consts::PI;
use std::time::Duration;

use eyre::Result;
use rodio::source::Buffered;
use rodio::{buffer::SamplesBuffer, OutputStream, Sink, Source};

use crate::keyer::KeyerState;

fn sine_wave(frequency: f32, length: usize, sample_rate: u32) -> Vec<f32> {
    let mut samples = Vec::with_capacity(length);

    for i in 0..length {
        let t = i as f32 / sample_rate as f32;
        samples.push((2.0 * PI * frequency * t).sin());
    }

    fade_in_and_out(&mut samples);

    samples
}

fn fade_in_and_out(samples: &mut Vec<f32>) {
    let len = samples.len();

    let fade_length = len / 15;

    for i in 0..fade_length {
        let factor = i as f32 / fade_length as f32;
        samples[i] *= factor;
        samples[len - fade_length + i] *= 1.0 - factor;
    }
}

pub struct Sidetone {
    _stream: OutputStream,
    sink: Sink,
    dit_sound: Buffered<SamplesBuffer<f32>>,
    dah_sound: Buffered<SamplesBuffer<f32>>,
}

impl Sidetone {
    pub fn new(freq: f32, dit_length: Duration, dah_length: Duration) -> Result<Self> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle).unwrap();

        let fs = 44100;
        let dit_length = (dit_length.as_secs_f32() * fs as f32) as usize;
        let dah_length = (dah_length.as_secs_f32() * fs as f32) as usize;
        let dit_sound = SamplesBuffer::new(1, fs, sine_wave(freq, dit_length, fs)).buffered();
        let dah_sound = SamplesBuffer::new(1, fs, sine_wave(freq, dah_length, fs)).buffered();

        Ok(Self {
            _stream: stream,
            sink,
            dit_sound,
            dah_sound,
        })
    }

    pub fn sound_sign(&self, state: KeyerState) {
        match state {
            KeyerState::Dit => self.sink.append(self.dit_sound.clone()),
            KeyerState::Dah => self.sink.append(self.dah_sound.clone()),
            _ => (),
        }
    }
}
