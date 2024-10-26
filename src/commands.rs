use clap::Parser;

pub struct CmdsPak(Cmds);
impl CmdsPak {
    pub fn from_env() -> Self {
        CmdsPak(Cmds::parse())
    }
}

/// vtyrec
///
/// A tool to record terminal
#[derive(Parser)]
struct Cmds {}
