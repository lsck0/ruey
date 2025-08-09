#![allow(clippy::needless_return)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod models;
pub mod schema;
pub mod state;
pub mod ui;
pub mod window;

use std::{io, os::fd::AsRawFd as _};

use eframe::{EframePumpStatus, UserEvent};
use tokio::task::LocalSet;
use winit::event_loop::{ControlFlow, EventLoop};

use crate::{state::App, window::get_window_options};

fn main() -> io::Result<()> {
    egui_logger::builder().init().unwrap();

    let mut egui_eventloop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
    egui_eventloop.set_control_flow(ControlFlow::Poll);

    let mut egui_app = eframe::create_native(
        env!("CARGO_PKG_NAME"),
        get_window_options(),
        Box::new(|_| Ok(App::new())),
        &egui_eventloop,
    );

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    return LocalSet::new().block_on(&runtime, async {
        let eventloop_fd = tokio::io::unix::AsyncFd::new(egui_eventloop.as_raw_fd())?;
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
                    log::info!("exit code: {code}");
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
