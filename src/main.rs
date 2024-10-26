use clap::Parser;
use color_eyre::Result;
use crossterm::event;
use std::{
    io::{BufWriter, Write},
    sync::{Arc, RwLock},
};
use tokio::task;
use tui_term::vt100;

mod key2bytes;

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
    #[arg(default_value = "tty.rec")]
    file: std::ffi::OsString,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    use portable_pty::{CommandBuilder, NativePtySystem, PtyPair, PtySize, PtySystem};
    let cli = Cli::parse();

    color_eyre::config::HookBuilder::new().install()?;
    let mut terminal = ratatui::try_init()?;
    let size = terminal.size()?;

    let parser = Arc::new(RwLock::new(vt100::Parser::new(size.height, size.width, 0)));
    let PtyPair { slave, master } = NativePtySystem::default()
        .openpty(PtySize {
            rows: size.height,
            cols: size.width,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();
    // Wait for the child to complete
    let child_exit = {
        let mut cmd = CommandBuilder::new(std::env::var_os("SHELL").unwrap_or("sh".into()));
        cmd.cwd(std::env::current_dir()?);
        let (tx, rx) = tokio::sync::oneshot::channel();
        let mut child = slave
            .spawn_command(cmd)
            .map_err(|e| color_eyre::eyre::anyhow!("{}", e))?;
        let mut reader = master.try_clone_reader().unwrap();
        let parser = parser.clone();
        task::spawn_blocking(move || {
            // Consume the output from the child
            // Can't read the full buffer, since that would wait for EOF
            let mut buf = [0u8; 8192];
            let mut processed_buf = Vec::with_capacity(8192);
            loop {
                if let Some(s) = child.try_wait().transpose() {
                    tx.send(s).unwrap();
                    drop(slave);
                    break;
                } else {
                    // A known issue:
                    // `reader read` will block this thread, which
                    // blocks `child try_wait`, causing oneshot send
                    // after `reader read` finish, leading program
                    // to shutdown after another press after `exit`
                    let size = reader.read(&mut buf).unwrap();
                    if size > 0 {
                        processed_buf.extend_from_slice(&buf[..size]);
                        let mut parser = parser.write().unwrap();
                        parser.process(&processed_buf);

                        // Clear the processed portion of the buffer
                        processed_buf.clear();
                    } else {
                        break;
                    }
                }
            }
        });
        rx
    };

    let mut writer = BufWriter::new(
        master
            .take_writer()
            .map_err(|e| color_eyre::eyre::anyhow!("{}", e))?,
    );
    if let Some(pgm) = cli.command {
        writer.write_all(pgm.as_encoded_bytes())?;
        writer.write_all(&[key2bytes::ascii::ENTER])?;
        writer.flush()?;
    }
    let ttyrec_writer = ttyrec::Writer::new(
        tokio::fs::OpenOptions::new()
            .truncate(!cli.append) // overwrite all
            .append(cli.append)
            .create(true)
            .write(true)
            .open(cli.file)
            .await?,
    );

    let _child_exit_code = run(&mut terminal, parser, writer, child_exit, ttyrec_writer).await?;

    // restore terminal
    drop(terminal);
    ratatui::try_restore()?;
    Ok(())
}
async fn run(
    terminal: &mut ratatui::DefaultTerminal,
    parser: Arc<RwLock<vt100::Parser>>,
    mut sender: BufWriter<Box<dyn Write + Send>>,
    mut exit_status: tokio::sync::oneshot::Receiver<std::io::Result<portable_pty::ExitStatus>>,
    mut writer: ttyrec::Writer<tokio::fs::File>,
) -> Result<u32> {
    use event::{Event, EventStream, KeyEventKind};
    let mut evs = EventStream::new();
    let mut timeout = tokio::time::interval(std::time::Duration::from_millis(20));
    let mut prev_screen = parser.read().unwrap().screen().clone();
    loop {
        let now_screen = parser.read().unwrap().screen().clone();
        terminal.draw(|f| ui(f, &now_screen))?;

        let diff = now_screen.contents_diff(&prev_screen);
        if !diff.is_empty() {
            writer.frame(&diff).await?;
            prev_screen = now_screen;
        }

        let ev = tokio::select! {
            e = {
                use tokio_stream::StreamExt;
                evs.next()
            } => e,
            _ = timeout.tick() => None,
            s = &mut exit_status => return Ok(s.unwrap()?.exit_code()),
        };

        if let Some(ev) = ev {
            match ev? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    use key2bytes::{ToBytes, U8Code};
                    let byte = key.code.into_byte_code();
                    match byte {
                        U8Code::Ascii(byte) => sender.write_all(&[byte])?,
                        U8Code::TriU8(bytes) => sender.write_all(&bytes)?,
                        U8Code::Auto(vec) => sender.write_all(&vec)?,
                    }
                    sender.flush()?;
                }
                Event::Key(_) => tracing::trace!("KeyCode other than Press get, ignore"),
                Event::FocusGained => tracing::trace!("FocusGained"),
                Event::FocusLost => tracing::trace!("FocusLost"),
                Event::Mouse(_) => tracing::trace!("mouse event get, ignored"),
                Event::Paste(_) => unreachable!("bracketed-paste should be not enabled"),
                Event::Resize(cols, rows) => {
                    parser.write().unwrap().set_size(rows, cols);
                }
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, screen: &vt100::Screen) {
    use tui_term::widget::PseudoTerminal;
    let pseudo_term = PseudoTerminal::new(screen);
    f.render_widget(pseudo_term, f.area());
}
