#![allow(clippy::needless_return)]
#![forbid(clippy::unwrap_used)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod app;
pub mod engine;
pub mod models;
pub mod schema;
pub mod state;
pub mod twitch;
pub mod ui;
pub mod window;
pub mod workers;

use eframe::{EframePumpStatus, UserEvent};
use std::io;
use tokio::task::LocalSet;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use winit::event_loop::{ControlFlow, EventLoop};

use crate::{app::App, window::get_window_options};

#[cfg(unix)]
fn main() -> io::Result<()> {
    use std::os::fd::AsRawFd;
    use tokio::io::unix::AsyncFd;

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "ruey=trace".to_string().into()))
        .with(fmt::layer().with_writer(std::io::stdout))
        .init();

    let mut egui_eventloop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .expect("Failed to create event loop");
    egui_eventloop.set_control_flow(ControlFlow::Poll);

    let mut egui_app = eframe::create_native(
        env!("CARGO_PKG_NAME"),
        get_window_options(),
        Box::new(|cctx| Ok(App::new(cctx))),
        &egui_eventloop,
    );

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");

    return LocalSet::new().block_on(&runtime, async {
        let eventloop_fd = AsyncFd::new(egui_eventloop.as_raw_fd())?;
        let mut control_flow = ControlFlow::Poll;

        loop {
            let mut guard = match control_flow {
                ControlFlow::Poll => None,
                ControlFlow::Wait => Some(eventloop_fd.readable().await?),
                ControlFlow::WaitUntil(deadline) => tokio::time::timeout_at(deadline.into(), eventloop_fd.readable())
                    .await
                    .ok()
                    .transpose()?,
            };

            match egui_app.pump_eframe_app(&mut egui_eventloop, None) {
                EframePumpStatus::Continue(next) => control_flow = next,
                EframePumpStatus::Exit(code) => {
                    info!("exit code: {code}");
                    break;
                }
            }

            if let Some(mut guard) = guard.take() {
                guard.clear_ready();
            }
        }

        return Ok::<_, io::Error>(());
    });
}

#[cfg(windows)]
fn main() -> io::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "ruey=debug".to_string().into()))
        .with(fmt::layer().with_writer(std::io::stdout))
        .init();

    let mut egui_eventloop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .expect("Failed to create event loop");
    egui_eventloop.set_control_flow(ControlFlow::Wait);

    let mut egui_app = eframe::create_native(
        env!("CARGO_PKG_NAME"),
        get_window_options(),
        Box::new(|cctx| Ok(App::new(cctx))),
        &egui_eventloop,
    );

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");

    return LocalSet::new().block_on(&runtime, async {
        loop {
            match egui_app.pump_eframe_app(&mut egui_eventloop, None) {
                EframePumpStatus::Continue(_) => {}
                EframePumpStatus::Exit(code) => {
                    info!("exit code: {code}");
                    break;
                }
            }
        }

        return Ok::<_, io::Error>(());
    });
}
