use colored::*;
use env_logger::Builder;
use indicatif::ProgressBar;
use log::{Level, LevelFilter};
use std::{
    fmt,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock,
    },
};

static MAX_WINDOW_WIDTH: AtomicUsize = AtomicUsize::new(0);

lazy_static::lazy_static! {
    /// Stores the progress bar for the logging facility.
    static ref PROGRESS_BAR: RwLock<Option<Arc<ProgressBar>>> = RwLock::new(None);
}

struct Padded<T> {
    value: T,
    width: usize,
}

impl<T: fmt::Display> fmt::Display for Padded<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: <width$}", self.value, width = self.width)
    }
}

fn max_target_width(target: &str) -> usize {
    let max_width = MAX_WINDOW_WIDTH.load(Ordering::Relaxed);
    if max_width < target.len() {
        MAX_WINDOW_WIDTH.store(target.len(), Ordering::Relaxed);
        target.len()
    } else {
        max_width
    }
}

fn colored_level(level: Level) -> ColoredString {
    match level {
        Level::Trace => "TRACE".magenta().bold(),
        Level::Debug => "DEBUG".blue().bold(),
        Level::Info => "INFO ".green().bold(),
        Level::Warn => "WARN ".yellow().bold(),
        Level::Error => "ERROR".red().bold(),
    }
}

pub fn init(level: Option<Level>) {
    let mut builder = Builder::new();

    builder.filter_level(LevelFilter::Warn);

    if let Ok(s) = ::std::env::var("RUST_LOG") {
        builder.parse_filters(&s);
    }

    if let Some(level) = level {
        builder.filter_level(level.to_level_filter());
    }

    builder.format(move |f, record| {
        let target = record.target();
        let max_width = max_target_width(target);

        let level = colored_level(record.level());

        let mut style = f.style();
        let target = style.set_bold(true).value(Padded {
            value: target,
            width: max_width,
        });

        let guard = PROGRESS_BAR.write().unwrap();
        if let Some(pb) = &*guard {
            pb.println(format!(" {} {} > {}", level, target, record.args()));
        } else {
            println!(" {} {} > {}", level, target, record.args());
        }

        Ok(())
    });

    builder.try_init().unwrap();
}

/// Sets the current progress bar in store for the logging facility.
pub fn set_progress_bar(progress: Arc<ProgressBar>) {
    let mut guard = PROGRESS_BAR.write().unwrap();
    *guard = Some(progress);
}

/// Writes an error to the log.
/// This can be used for unwraps/eprintlns/etc.
pub fn eprintln(message: impl AsRef<str>) {
    let guard = PROGRESS_BAR.write().unwrap();
    if let Some(pb) = &*guard {
        pb.println(message.as_ref());
    } else {
        eprintln!("{}", message.as_ref());
    }
}

/// Writes an error to the log.
/// This can be used for unwraps/eprintlns/etc.
pub fn println(message: impl AsRef<str>) {
    let guard = PROGRESS_BAR.write().unwrap();
    if let Some(pb) = &*guard {
        pb.println(message.as_ref());
    } else {
        println!("{}", message.as_ref());
    }
}
