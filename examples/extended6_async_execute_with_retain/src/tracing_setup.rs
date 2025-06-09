use std::fmt::Debug;
use tracing::Level;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;

pub fn tracing_init() {
    let subscriber = tracing_subscriber::fmt()
        .with_file(false)
        .with_line_number(false)
        .with_thread_names(false)
        .with_thread_ids(true)
        .with_target(false)
        .with_max_level(Level::DEBUG)
        .with_timer(ShortTime::default())
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ShortTime {
    epoch: chrono::DateTime<chrono::offset::Local>,
}

impl Default for ShortTime {
    fn default() -> Self {
        Self {
            epoch: chrono::Local::now(),
        }
    }
}

impl FormatTime for ShortTime {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        let e = self.epoch;
        write!(w, "{}", e.format("%H:%M:%S"))
    }
}
