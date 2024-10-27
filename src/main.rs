use clap::Parser;
use color_eyre::Result;
use crossterm::event;
use portable_pty::CommandBuilder;
use std::{
    io::{BufWriter, Write},
    time::Duration,
};
use tui_term::vt100;
use utils::{
    child::{spawn_pty_child, CHILD_HAD_EXIT, CHILD_SHOULD_EXIT},
    key2bytes::U8Code,
    recorder::{VtyParser, VtyrecWriter},
};

mod parser;
mod utils;

const DEFAULT_FILE_NAME: &str = "tty.rec";
const DEFAULT_SHELL: &str = "sh";

#[derive(Parser)]
/// Vtyrec is a tty recorder.  It aims to be a rust impl of ttyrec, with extended functions,
/// such as vhs-like script.
struct Cli {
    /// Invoke <command> when ttyrec starts.
    ///
    /// If the variable <SHELL> exists, the shell forked by vtyrec will be that shell.
    /// Otherwise, 'sh' is assumed
    #[arg(short = 'e')]
    command: Option<std::ffi::OsString>,
    /// Append the output to <FILE>, rather than overwriting it.
    #[arg(short = 'a')]
    append: bool,
    #[arg(default_value = DEFAULT_FILE_NAME)]
    file: std::ffi::OsString,
    /// support vhs-like script
    #[arg(short = 's')]
    script: Option<std::ffi::OsString>,
}

fn main() -> Result<()> {
    use portable_pty::{NativePtySystem, PtyPair, PtySize, PtySystem};
    color_eyre::config::HookBuilder::new().install()?;
    let mut cli = Cli::parse();

    let (mut cmd, size, event_list) = if let Some(script) = cli.script {
        let script_host = parser::Parser::from_reader(std::fs::File::open(script)?)?;
        cli.append = false;
        cli.file = script_host.env.file_name.into();
        cli.command = None;
        (
            CommandBuilder::new(script_host.env.shell),
            Some(script_host.env.size)
                .map(|(height, width)| ratatui::layout::Size { height, width }),
            Some(
                script_host
                    .commands
                    .into_iter()
                    .flat_map(|c| c.into_key(script_host.env.typingspeed))
                    .collect(),
            ),
        )
    } else {
        (
            CommandBuilder::new(std::env::var_os("SHELL").unwrap_or(DEFAULT_SHELL.into())),
            None,
            None,
        )
    };
    cmd.cwd(std::env::current_dir()?);

    let mut terminal = ratatui::try_init()?;
    let size = size.unwrap_or(terminal.size()?);

    let parser = VtyParser::new(std::sync::RwLock::new(vt100::Parser::new(
        size.height,
        size.width,
        0,
    )));
    let PtyPair { slave, master } = NativePtySystem::default()
        .openpty(PtySize {
            rows: size.height,
            cols: size.width,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();
    spawn_pty_child(
        cmd,
        master.try_clone_reader().unwrap(),
        slave,
        parser.clone(),
    );

    let mut writer = BufWriter::new(master.take_writer().unwrap());
    if let Some(pgm) = cli.command {
        // waiting for <shell> to be ready
        // this affect ui only, the record file is fine
        std::thread::sleep(Duration::from_millis(20));
        writer.write_all(pgm.as_encoded_bytes())?;
        writer.write_all(&[utils::key2bytes::ascii::ENTER])?;
        writer.flush()?;
    }
    let ttyrec_writer =
        VtyrecWriter::open(cli.file, cli.append).map(|writer| VtyrecWriter::new(writer, parser))?;

    if let Some(events) = event_list {
        run_script(&mut terminal, writer, events, ttyrec_writer)?
    } else {
        run_interactive(&mut terminal, writer, ttyrec_writer)?
    };

    // restore terminal
    drop(terminal);
    ratatui::try_restore()?;
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn run_interactive(
    terminal: &mut ratatui::DefaultTerminal,
    mut pty_writer: BufWriter<Box<dyn Write + Send>>,
    mut rec_writer: VtyrecWriter,
) -> Result<()> {
    use event::{Event, EventStream, KeyEventKind};
    use tokio_stream::StreamExt;
    let mut evs = EventStream::new();
    let mut timeout = tokio::time::interval(Duration::from_millis(20));
    loop {
        let now_screen = rec_writer.tick()?;
        terminal.draw(|f| ui(f, now_screen))?;

        if CHILD_HAD_EXIT.is_completed() {
            return Ok(());
        }

        let ev = tokio::select! {
            e = evs.next() => e,
            _ = timeout.tick() => None,
        };

        if let Some(ev) = ev {
            match ev? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    use utils::key2bytes::ToBytes;
                    let byte = key.into_byte_code();
                    match byte {
                        U8Code::Ascii(byte) => pty_writer.write_all(&[byte])?,
                        U8Code::TriU8(bytes) => pty_writer.write_all(&bytes)?,
                        U8Code::Auto(vec) => pty_writer.write_all(&vec)?,
                    }
                    pty_writer.flush()?;
                }
                Event::Key(_) => tracing::trace!("KeyCode other than Press get, ignore"),
                Event::FocusGained => tracing::trace!("FocusGained"),
                Event::FocusLost => tracing::trace!("FocusLost"),
                Event::Mouse(_) => tracing::trace!("mouse event get, ignored"),
                Event::Paste(_) => unimplemented!("should be handled by outside"),
                Event::Resize(cols, rows) => {
                    rec_writer.as_ref().write().unwrap().set_size(rows, cols);
                }
            }
        }
    }
}

fn run_script(
    terminal: &mut ratatui::DefaultTerminal,
    mut pty_writer: BufWriter<Box<dyn Write + Send>>,
    events: Vec<(U8Code, u64)>,
    mut rec_writer: VtyrecWriter,
) -> Result<()> {
    for (code, delay) in events {
        let now_screen = rec_writer.tick()?;
        terminal.draw(|f| ui(f, now_screen))?;
        std::thread::sleep(Duration::from_millis(delay));
        match code {
            U8Code::Ascii(byte) => pty_writer.write_all(&[byte])?,
            U8Code::TriU8(bytes) => pty_writer.write_all(&bytes)?,
            U8Code::Auto(vec) => pty_writer.write_all(&vec)?,
        }
        pty_writer.flush()?;
    }
    CHILD_SHOULD_EXIT.call_once(|| ());
    // waiting for child
    while !CHILD_HAD_EXIT.is_completed() {}
    Ok(())
}

fn ui(f: &mut ratatui::Frame, screen: &vt100::Screen) {
    use tui_term::widget::PseudoTerminal;
    let pseudo_term = PseudoTerminal::new(screen);
    f.render_widget(pseudo_term, f.area());
}
