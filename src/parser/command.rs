use super::error::ParseError;
use crate::utils::key2bytes::{self as Keys, U8Code as Key};

pub enum Commands {
    Output(String),
    Set(super::env::EnVar),

    Enter(usize, Option<u64>),
    Escape(usize, Option<u64>),

    Tab(usize, Option<u64>),
    Space(usize, Option<u64>),
    Up(usize, Option<u64>),
    Down(usize, Option<u64>),
    Left(usize, Option<u64>),
    Right(usize, Option<u64>),
    BackSpace(usize, Option<u64>),

    Sleep(u64),

    Type(String, Option<u64>),
    Null,
}
impl Commands {
    pub fn into_key(self, default_delay: u64) -> Vec<(Key, u64)> {
        fn repeat_with_delay(
            key: Key,
            delay: Option<u64>,
            default_delay: u64,
            times: usize,
        ) -> Vec<(Key, u64)> {
            let delay = delay.unwrap_or(default_delay);
            vec![(key, delay); times]
        }
        match self {
            Commands::Output(_) => unimplemented!(),
            Commands::Set(_) => unimplemented!(),

            Commands::Enter(times, delay) => {
                repeat_with_delay(Key::Ascii(Keys::ascii::ENTER), delay, default_delay, times)
            }
            Commands::Escape(times, delay) => {
                repeat_with_delay(Key::Ascii(Keys::ascii::ESC), delay, default_delay, times)
            }

            Commands::Tab(times, delay) => {
                repeat_with_delay(Key::Ascii(Keys::ascii::TAB), delay, default_delay, times)
            }
            Commands::Space(times, delay) => {
                repeat_with_delay(Key::Ascii(Keys::ascii::SPACE), delay, default_delay, times)
            }
            Commands::Up(times, delay) => {
                repeat_with_delay(Key::TriU8(Keys::UP), delay, default_delay, times)
            }
            Commands::Down(times, delay) => {
                repeat_with_delay(Key::TriU8(Keys::DOWN), delay, default_delay, times)
            }
            Commands::Left(times, delay) => {
                repeat_with_delay(Key::TriU8(Keys::LEFT), delay, default_delay, times)
            }
            Commands::Right(times, delay) => {
                repeat_with_delay(Key::TriU8(Keys::RIGHT), delay, default_delay, times)
            }
            Commands::BackSpace(times, delay) => repeat_with_delay(
                Key::Ascii(Keys::ascii::BACKSPACE),
                delay,
                default_delay,
                times,
            ),

            Commands::Sleep(length) => vec![(
                // Seem to be harmless
                Key::Ascii(Keys::ascii::NULL),
                length,
            )],

            Commands::Type(s, sp) => {
                vec![(Key::Auto(s.into_bytes()), sp.unwrap_or(default_delay))]
            }
            Commands::Null => unimplemented!(),
        }
    }
}
impl core::fmt::Display for Commands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn format_command(cmd: &str, times: &usize, time: &Option<u64>) -> std::string::String {
            format!(
                "{cmd}{} {times}",
                time.map(|s| format!("@{s}ms")).unwrap_or_default()
            )
        }
        write!(
            f,
            "{}",
            match self {
                Commands::Output(f) => format!("Output {f}"),
                Commands::Set(v) => format!("Set {v}"),

                Commands::Enter(n, sp) => format_command("Enter", n, sp),
                Commands::Escape(n, sp) => format_command("Escape", n, sp),
                Commands::Tab(n, sp) => format_command("Tab", n, sp),
                Commands::Space(n, sp) => format_command("Space", n, sp),
                Commands::Up(n, sp) => format_command("Up", n, sp),
                Commands::Down(n, sp) => format_command("Down", n, sp),
                Commands::Left(n, sp) => format_command("Left", n, sp),
                Commands::Right(n, sp) => format_command("Right", n, sp),
                Commands::BackSpace(n, sp) => format_command("BackSpace", n, sp),
                Commands::Sleep(v) => format!("Sleep {v}ms"),

                Commands::Type(v, sp) => format!(
                    "Type{} \"{v}\"",
                    sp.map(|s| format!("@{s}ms")).unwrap_or_default()
                ),
                Commands::Null => return Err(std::fmt::Error),
            }
        )
    }
}
const COMMANDS: [&str; 14] = [
    "Output ",
    "Set ",
    "Enter",
    "Escape",
    "Tab",
    "Space",
    "Up",
    "Down",
    "Left",
    "Right",
    "BackSpace",
    "Sleep ",
    "Type",
    "#",
];
impl core::str::FromStr for Commands {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim_start();
        if s.is_empty() {
            return Ok(Commands::Null);
        }
        for pat in COMMANDS {
            use super::utils::{parse_with_delay_or, parse_with_delay_times};
            if !s.starts_with(pat) {
                continue;
            }
            let s = s.strip_prefix(pat).unwrap().trim();
            return match pat {
                "Output " => Ok(Commands::Output(s.into())),
                "Set " => s.parse().map(Commands::Set),

                "Enter" => Ok(parse_with_delay_times(s).map(|(s,n)|Commands::Enter(s, n))?),
                "Escape" => Ok(parse_with_delay_times(s).map(|(s,n)|Commands::Escape(s,n))?),
                "Tab" => Ok(parse_with_delay_times(s).map(|(s,n)|Commands::Tab(s, n))?),
                "Space" => Ok(parse_with_delay_times(s).map(|(s,n)|Commands::Space(s, n))?),
                "Up" => Ok(parse_with_delay_times(s).map(|(s,n)|Commands::Up(s, n))?),
                "Down" => Ok(parse_with_delay_times(s).map(|(s,n)|Commands::Down(s, n))?),
                "Left" => Ok(parse_with_delay_times(s).map(|(s,n)|Commands::Left(s, n))?),
                "Right" => Ok(parse_with_delay_times(s).map(|(s,n)|Commands::Right(s, n))?),
                "BackSpace" => Ok(parse_with_delay_times(s).map(|(s,n)|Commands::BackSpace(s, n))?),
                "Sleep " => super::utils::parse_sleep(s)
                    .map(Commands::Sleep)
                    .map_err(|_| {
                        ParseError(Box::from(
                            "Failed to parse `Sleep`, make sure it's like `Sleep 500ms`/`Sleep 1s`",
                        ))
                    }),
                "Type" => parse_with_delay_or(s).map(|(s,n)|Commands::Type(s.to_string(), n))
                    .map_err(|_| {
                        ParseError(Box::from(
                            "Failed to parse `Type`, make sure it's like `Type@200ms \"test\"`/`Type@0.1s` \"test\" or `Type \"test\"`",
                        ))
                    }),
                "#" => Ok(Commands::Null),
                _ => unreachable!(),
            };
        }
        Err(ParseError(
            format!("Failed to parse Commands`{s}`, what's that?").into(),
        ))
    }
}

#[cfg(test)]
#[test]
fn test() {
    let tape = "Output demo.gif
# test
Type \"ttytape\"
Sleep 500ms
Type@100ms \"ttyrec\"

Set TypingSpeed 100ms
Set FontSize 17
Set Width 81
Set Height 41
Enter 2
Sleep 1s
Tab
Escape@100ms
BackSpace@0.1s 3
";
    fn print(tape: &str) {
        tape.lines()
            .inspect(|s| print!("{s} => "))
            .map(|s| s.parse().unwrap())
            .filter(|c| if let &Commands::Null = c { false } else { true })
            .for_each(|v: Commands| println!("{v}"));
    }
    print(tape)
}
