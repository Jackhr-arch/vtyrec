use crate::{DEFAULT_FILE_NAME, DEFAULT_SHELL};

use super::error::ParseError;
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Envs {
    // unify to ms
    pub typingspeed: u64,
    pub file_name: String,
    pub shell: String,
    pub size: (u16, u16),
}
impl Default for Envs {
    fn default() -> Self {
        Self {
            typingspeed: 200,
            file_name: DEFAULT_FILE_NAME.to_string(),
            shell: DEFAULT_SHELL.to_string(),
            size: (16, 80),
        }
    }
}
impl Envs {
    pub fn set(&mut self, new: EnVar) {
        match new {
            EnVar::TypingSpeed(s) => self.typingspeed = s,
            EnVar::Shell(s) => self.shell = s,
            EnVar::FontSize(_) => unimplemented!("Not support"),
            EnVar::Width(w) => self.size.1 = w,
            EnVar::Height(h) => self.size.0 = h,
        }
    }
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub enum EnVar {
    TypingSpeed(u64),
    Shell(String),
    FontSize(u8),
    Width(u16),
    Height(u16),
}
impl core::fmt::Display for EnVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnVar::TypingSpeed(v) => write!(f, "TypingSpeed {v}"),
            EnVar::Shell(s) => write!(f, "Shell {s}"),
            EnVar::FontSize(n) => write!(f, "FontSize {n}"),
            EnVar::Width(w) => write!(f, "Width {w}"),
            EnVar::Height(h) => write!(f, "Height {h}"),
        }
    }
}
const ENVS: [&str; 5] = ["TypingSpeed ", "Shell ", "FontSize ", "Width ", "Height "];
impl core::str::FromStr for EnVar {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for pat in ENVS {
            if !s.starts_with(pat) {
                continue;
            }
            let s = s.strip_prefix(pat).unwrap().trim();
            return match pat {
                "TypingSpeed "=>super::utils::parse_sleep(s).map(Self::TypingSpeed).map_err(|_|ParseError(Box::from("Failed to parse `TypingSpeed`, make sure it's like `Set TypingSpeed 500ms`"))),
                "Shell " => Ok(Self::Shell(s.trim().to_string())),
                "FontSize " => s.parse::<u8>().map(Self::FontSize).map_err(|e|ParseError(e.to_string().into_boxed_str())),
                "Width " => s.parse::<u16>().map(Self::Width).map_err(|e|ParseError(e.to_string().into_boxed_str())),
                "Height "=>s.parse::<u16>().map(Self::Height).map_err(|e|ParseError(e.to_string().into_boxed_str())),
                _=>unreachable!()
            };
        }
        Err(ParseError(
            format!("Failed to parse WorkEnv`{s}`, what's that?").into(),
        ))
    }
}
