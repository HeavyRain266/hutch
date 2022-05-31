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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log = slog::Logger::root(slog_stdlog::StdLog.fuse(), slog::o!());

    slog_stdlog::init().expect("Cannot initialize logger");

    let mut evloop: EventLoop<CalloopData> = EventLoop::try_new()?;

    let mut display: Display<Hutch> = Display::new()?;
    let state = Hutch::new(&mut evloop, &mut display, log.clone());

    let mut data = CalloopData { state, display };

    crate::winit::window(&mut evloop, &mut data, log)?;

    let mut args = std::env::args().skip(1);
    let flag = args.next();
    let arg = args.next();

    if let (Some("-c"), Some(cmd)) = (flag.as_deref(), arg) {
        std::process::Command::new(cmd).spawn().ok();
    } else {
        println!("Missing startup client");
    }

    evloop.run(None, &mut data, move |_| {
        // Hutch is running
    })?;

    Ok(())
}
