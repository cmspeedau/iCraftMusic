//! MusicCraft — a native desktop music player written entirely in Rust.
//!
//! The crate is split into a library and a thin binary (`src/main.rs`) so the
//! application modules can be exercised by `cargo test` without opening a
//! desktop window.
//!
//! Module layout follows `AGENT.md` §4. Stage 0 and Stage 1 of `ROADMAP.md`
//! provide only the foundation and the visual shell; audio, library, database,
//! playlist and search modules are added by the stages that require them.

pub mod app;
pub mod audio;
pub mod config;
pub mod database;
pub mod error;
pub mod library;
pub mod logging;
pub mod playlists;
pub mod ui;

/// Human-readable application name, used for window titles and config paths.
pub const APP_NAME: &str = "MusicCraft";

/// Application version, taken from `Cargo.toml` at compile time.
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
