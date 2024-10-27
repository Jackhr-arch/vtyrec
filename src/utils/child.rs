use crate::utils::recorder::VtyParser;
use portable_pty::{CommandBuilder, SlavePty};
use std::sync::Once;

pub static CHILD_SHOULD_EXIT: Once = Once::new();
pub static CHILD_HAD_EXIT: Once = Once::new();

pub fn spawn_pty_child(
    cmd: CommandBuilder,
    mut rdr: Box<dyn std::io::Read + Send>,
    slave: Box<dyn SlavePty + Send>,
    parser: VtyParser,
) {
    let mut child = slave.spawn_command(cmd).unwrap();
    let _join = std::thread::spawn(move || {
        // Consume the output from the child
        // Can't read the full buffer, since that would wait for EOF
        let mut buf = [0u8; 8192];
        let mut processed_buf = Vec::with_capacity(8192);
        loop {
            if CHILD_SHOULD_EXIT.is_completed() {
                break;
            } else if let Some(_s) = child.try_wait().transpose() {
                break;
            } else {
                // A known issue:
                // `reader read` will block this thread, which
                // blocks `child try_wait`, causing oneshot send
                // after `reader read` finish, leading program
                // to shutdown after another press after `exit`
                let size = rdr.read(&mut buf).unwrap();
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
        CHILD_HAD_EXIT.call_once(|| ());
        drop(slave);
    });
}
