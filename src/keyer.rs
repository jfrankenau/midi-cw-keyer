use std::io::{stdout, StdoutLock, Write};
use std::time::Duration;

use eyre::Result;
use ringbuffer::{AllocRingBuffer, RingBuffer};
use tokio::sync::mpsc;

use crate::audio::Sidetone;
use crate::{Mode, MorseSign};

#[derive(Clone, Copy, Default, Debug)]
struct PaddleState {
    dit: bool,
    dah: bool,
}

impl PaddleState {
    fn key(&mut self, sign: MorseSign, pressed: bool) {
        match sign {
            MorseSign::Dit => self.dit = pressed,
            MorseSign::Dah => self.dah = pressed,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum KeyerState {
    #[default]
    Idle,
    Dit,
    Dah,
}

impl KeyerState {
    fn opposite(&self) -> Self {
        use KeyerState::*;
        match self {
            Dit => Dah,
            Dah => Dit,
            _ => unimplemented!(),
        }
    }
}

impl From<MorseSign> for KeyerState {
    fn from(sign: MorseSign) -> Self {
        match sign {
            MorseSign::Dit => KeyerState::Dit,
            MorseSign::Dah => KeyerState::Dah,
        }
    }
}

impl From<KeyerState> for MorseSign {
    fn from(state: KeyerState) -> Self {
        match state {
            KeyerState::Dit => MorseSign::Dit,
            KeyerState::Dah => MorseSign::Dah,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug)]
struct KeyerSettings {
    mode: Mode,
    freq: f32,
    dit_length: Duration,
    dah_length: Duration,
}

impl KeyerSettings {
    fn new(mode: Mode, freq: f32, dit_length: Duration) -> Self {
        Self {
            mode,
            freq,
            dit_length,
            dah_length: dit_length * 3,
        }
    }
}

pub struct Keyer {
    settings: KeyerSettings,
    state: KeyerState,
    paddle: PaddleState,
    queue: AllocRingBuffer<MorseSign>,
}

impl Keyer {
    pub fn new(freq: f32, wpm: u8, mode: Mode, buffer_size: u8) -> Self {
        let dit_length = Duration::from_millis((1200. / wpm as f32).round() as u64);
        let settings = KeyerSettings::new(mode, freq, dit_length);

        Self {
            settings,
            state: Default::default(),
            paddle: Default::default(),
            queue: AllocRingBuffer::new(buffer_size.into()),
        }
    }

    pub async fn run(&mut self, mut rx: mpsc::Receiver<(MorseSign, bool)>) -> Result<()> {
        use KeyerState::*;

        let tone = Sidetone::new(
            self.settings.freq,
            self.settings.dit_length,
            self.settings.dah_length,
        )?;
        let mut stdout = stdout().lock();

        loop {
            match self.state {
                Idle => {
                    // at the start or after a dit/dah plus pause we only need
                    // to listen for a key down event which immediately will
                    // put us into state Dit or state Dah; no queue needed
                    if let Some((sign, true)) = rx.recv().await {
                        self.paddle.key(sign, true);
                        self.state = sign.into();
                    }
                }
                Dit | Dah => {
                    // we now have to emit the sidetone
                    tone.sound_sign(self.state);
                    print_sign(&mut stdout, self.state)?;

                    // we need to sleep for the length of the dit/dah plus the
                    // ensuing space
                    let delay = tokio::time::sleep(match self.state {
                        Dit => self.settings.dit_length + self.settings.dit_length,
                        Dah => self.settings.dah_length + self.settings.dit_length,
                        _ => unreachable!(),
                    });

                    // this ensures that sleeping is resumed after a key event
                    // has been processed
                    tokio::pin!(delay);

                    loop {
                        tokio::select! {
                            // when sleep is done, continue with normal operation below
                            _ = &mut delay => break,
                            // while sleeping we also want to process key events
                            Some((sign, pressed)) = rx.recv() => {
                                self.paddle.key(sign, pressed);
                                if pressed {
                                    self.queue.enqueue(sign);
                                }
                            },
                        }
                    }

                    let dit = self.paddle.dit;
                    let dah = self.paddle.dah;

                    self.state = if let Some(sign) = self.queue.dequeue() {
                        sign.into()
                    } else if dit && dah {
                        match self.settings.mode {
                            Mode::IambicA => self.state.opposite(),
                            Mode::Ultimatic => self.state,
                        }
                    } else if dit {
                        Dit
                    } else if dah {
                        Dah
                    } else {
                        print_sign(&mut stdout, Idle)?;
                        Idle
                    };
                }
            }
        }
    }
}

fn print_sign(out: &mut StdoutLock, state: KeyerState) -> Result<()> {
    use KeyerState::*;
    let string = match state {
        Dit => ".",
        Dah => "-",
        Idle => " ",
    };
    write!(out, "{string}")?;
    Ok(out.flush()?)
}
