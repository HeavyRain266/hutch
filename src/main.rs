use slog::Drain;

mod handlers;

mod grabs;
mod input;
mod state;
mod winit;

use smithay::reexports::{
    calloop::EventLoop,
    wayland_server::Display
};

pub use state::Hutch;

pub struct CalloopData {
    state: Hutch,
    display: Display<Hutch>,
}

fn init_log() -> slog::Logger {
    let terminal_drain = slog_envlogger::LogBuilder::new(
        slog_term::CompactFormat::new(
            slog_term::TermDecorator::new()
                .stderr()
                .build()
            )
            .build()
            .fuse(),
    )
    .filter(Some("hutch"), slog::FilterLevel::Trace)
    .filter(Some("smithay"), slog::FilterLevel::Info)
    .build()
    .fuse();

    let terminal_drain = slog_async::Async::default(terminal_drain).fuse();

    let log = slog::Logger::root(terminal_drain.fuse(), slog::o!());

    slog_stdlog::init().expect("Could not setup log backend");

    log
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log = init_log();

    let mut evloop: EventLoop<CalloopData> = EventLoop::try_new()?;

    let mut display: Display<Hutch> = Display::new()?;
    let state = Hutch::new(&mut evloop, &mut display, log.clone());

    let mut data = CalloopData { state, display };

    crate::winit::window(&mut evloop, &mut data, log.clone())?;

    let mut args = std::env::args().skip(1);
    let flag = args.next();
    let arg = args.next();

    if let (Some("-c"), Some(cmd)) = (flag.as_deref(), arg) {
        std::process::Command::new(cmd).spawn().ok();
    } else {
        slog::warn!(log, "Missing startup client.");
        slog::info!(log, "Hint: use hutch -c <client> or WAYLAND_DISPLAY");
    }

    evloop.run(None, &mut data, move |_| {
        // Hutch is running
    })?;

    Ok(())
}
