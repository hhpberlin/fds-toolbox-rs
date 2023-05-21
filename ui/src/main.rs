// TODO: Re-enable and fix
// #![warn(clippy::pedantic)]

// #![warn(clippy::nursery)]
// #![warn(clippy::cargo)]
#![warn(clippy::complexity)]
#![warn(clippy::correctness)]
#![warn(clippy::perf)]
#![warn(clippy::style)]
#![warn(clippy::suspicious)]
#![warn(clippy::print_stdout)]
#![warn(clippy::print_stderr)]
// #![warn(clippy::todo)]
// #![warn(clippy::unimplemented)]
// #![warn(clippy::dbg_macro)]
// #![warn(clippy::unreachable)]
// #![warn(clippy::panic)]

// #![warn(clippy::unwrap_used)]
// #![warn(clippy::expect_used)]

// TODO: Remove this and remove dead-code once prototyping is done
#![allow(dead_code)]

use app::FdsToolbox;
use iced::{Application, Settings};
// use old::FdsToolbox;

// pub mod old;
mod app;
// mod sidebar;

/// # Errors
///
/// Errors if UI fails to start
pub fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    FdsToolbox::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}