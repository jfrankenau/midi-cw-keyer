use eyre::{OptionExt, Result};
use midir::MidiInput;
use tokio::sync::mpsc;
use wmidi::MidiMessage;

use midi_cw_keyer::keyer::Keyer;
use midi_cw_keyer::{Mode, MorseSign};

const HELP: &str = "\
midi-cw-keyer

USAGE:
  midi-cw-keyer [OPTIONS]

FLAGS:
  -h, --help             Show this help information
  --version              Show version

OPTIONS:
  -f NUM, --freq NUM     Sidetone frequency in Hz [default: 523.25]
  -w NUM, --wpm NUM      Words per minute [default: 20]
  -m MODE, --mode MODE   Keyer mode (a, u) [default: u]
  -b NUM, --buffer NUM   Buffer size [default: 1]
";

#[derive(Debug)]
struct Args {
    freq: f32,
    wpm: u8,
    mode: Mode,
    buffer_size: u8,
}

fn parse_args() -> Result<Args> {
    use lexopt::prelude::*;

    let mut freq = 523.25; // C5
    let mut wpm = 20;
    let mut mode = Mode::Ultimatic;
    let mut buffer_size = 1;

    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Short('h') | Long("help") => {
                print!("{}", HELP);
                std::process::exit(0);
            }
            Long("version") => {
                println!("{}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            Short('f') | Long("freq") => freq = parser.value()?.parse()?,
            Short('w') | Long("wpm") => wpm = parser.value()?.parse()?,
            Short('m') | Long("mode") => mode = parser.value()?.parse()?,
            Short('b') | Long("buffer") => buffer_size = parser.value()?.parse()?,
            _ => return Err(arg.unexpected().into()),
        }
    }

    Ok(Args {
        freq,
        wpm,
        mode,
        buffer_size,
    })
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = parse_args()?;

    let midi = MidiInput::new("midi-cw-keyer")?;
    let port_name = "MidiStomp";
    let midi_port = midi
        .ports()
        .into_iter()
        .find(|x| midi.port_name(x).unwrap().contains(port_name))
        .ok_or_eyre("Could not find MIDI device")?;

    let (tx, rx) = mpsc::channel(32);

    let _midi_conn = midi.connect(
        &midi_port,
        "midi-cw-keyer-read-input",
        move |_, message, _| {
            process_midi_message(&tx, message).unwrap();
        },
        (),
    );

    Keyer::new(args.freq, args.wpm, args.mode, args.buffer_size)
        .run(rx)
        .await
}

fn process_midi_message(tx: &mpsc::Sender<(MorseSign, bool)>, bytes: &[u8]) -> Result<()> {
    use MorseSign::*;

    match MidiMessage::try_from(bytes)? {
        MidiMessage::NoteOn(_, note, _) => match note.into() {
            1 => tx.try_send((Dit, true))?,
            2 => tx.try_send((Dah, true))?,
            _ => {}
        },
        MidiMessage::NoteOff(_, note, _) => match note.into() {
            1 => tx.try_send((Dit, false))?,
            2 => tx.try_send((Dah, false))?,
            _ => {}
        },
        _ => {}
    }

    Ok(())
}
