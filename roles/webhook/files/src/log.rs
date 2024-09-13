use std::{
    io::{self, Write},
    sync::Mutex,
};

pub struct LogWriter {}

impl Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut lock = CAPTURED_LOG.lock().unwrap();

        if let Some(captured) = lock.take() {
            let log = String::from_utf8(buf.to_vec()).unwrap();
            *lock = Some(captured + &log);
            Ok(buf.len())
        } else {
            io::stdout().write(buf)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        io::stdout().flush()
    }
}

static CAPTURED_LOG: Mutex<Option<String>> = Mutex::new(None);

pub fn start_capture() {
    *CAPTURED_LOG.lock().unwrap() = Some(String::new());
}
pub fn stop_capture() -> String {
    let mut lock = CAPTURED_LOG.lock().unwrap();
    lock.take().unwrap_or("".to_owned())
}
