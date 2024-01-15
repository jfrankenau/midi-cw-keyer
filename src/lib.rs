mod audio;
pub mod keyer;

use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Mode {
    IambicA,
    Ultimatic,
}

impl FromStr for Mode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Mode::*;
        match s {
            "a" => Ok(IambicA),
            "u" => Ok(Ultimatic),
            _ => Err("invalid mode"),
        }
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub enum MorseSign {
    #[default]
    Dit,
    Dah,
}
