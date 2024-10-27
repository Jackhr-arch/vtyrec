use std::sync::{Arc, RwLock};
use tui_term::vt100;

pub type VtyParser = Arc<RwLock<vt100::Parser>>;
type TtyWriter = ttyrec::blocking::Writer<std::fs::File>;

pub struct VtyrecWriter {
    writer: TtyWriter,
    parser: VtyParser,
    prev_screen: vt100::Screen,
}

impl VtyrecWriter {
    pub fn open(
        file: impl AsRef<std::path::Path>,
        append_or_truncate: bool,
    ) -> std::io::Result<TtyWriter> {
        std::fs::OpenOptions::new()
            .truncate(!append_or_truncate) // overwrite all
            .append(append_or_truncate) // or append it
            .create(true)
            .write(true)
            .open(file)
            .map(TtyWriter::new)
    }
    pub fn new(writer: TtyWriter, parser: VtyParser) -> Self {
        let prev_screen = parser.read().unwrap().screen().clone();
        Self {
            writer,
            parser,
            prev_screen,
        }
    }
    pub fn tick(&mut self) -> ttyrec::Result<&vt100::Screen> {
        let now_screen = self.parser.read().unwrap().screen().clone();
        let diff = now_screen.contents_diff(&self.prev_screen);
        if !diff.is_empty() {
            self.writer.frame(&diff)?;
            self.prev_screen = now_screen;
        }
        Ok(&self.prev_screen)
    }
}

impl AsRef<RwLock<vt100::Parser>> for VtyrecWriter {
    fn as_ref(&self) -> &RwLock<vt100::Parser> {
        &self.parser
    }
}
