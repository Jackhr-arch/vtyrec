use std::io::{BufRead, BufReader, Read};

mod command;
mod env;
mod error;
mod utils;

pub struct Parser {
    pub env: env::Envs,
    pub commands: Vec<command::Commands>,
}
impl Parser {
    pub fn from_reader<R: Read>(rdr: R) -> color_eyre::Result<Self> {
        let rdr = BufReader::new(rdr);
        let mut vec = Vec::with_capacity(1024);
        let mut env = env::Envs::default();
        for line in rdr.lines() {
            let cmd = line?.parse()?;
            match cmd {
                command::Commands::Output(name) => env.file_name = name,
                command::Commands::Set(en_var) => env.set(en_var),
                command::Commands::Null => (),
                _ => vec.push(cmd),
            }
        }
        Ok(Self { commands: vec, env })
    }
}
