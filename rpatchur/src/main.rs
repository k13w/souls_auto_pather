#![windows_subsystem = "windows"]

mod patcher;
mod process;
mod ui;

use log::LevelFilter;
use std::env;
use std::path::PathBuf;
use std::sync::mpsc;

use anyhow::{anyhow, Context, Result};
use simple_logger::SimpleLogger;
use structopt::StructOpt;
use tinyfiledialogs as tfd;

use patcher::{
    patcher_thread_routine, retrieve_patcher_configuration, PatcherCommand, PatcherConfiguration,
};
use ui::native::{NativeUi, PatchingStatus};

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const PKG_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

#[derive(Debug, StructOpt)]
#[structopt(name = PKG_NAME, version = PKG_VERSION, author = PKG_AUTHORS, about = PKG_DESCRIPTION)]
struct Opt {
    /// Sets a custom working directory
    #[structopt(short, long, parse(from_os_str))]
    working_directory: Option<PathBuf>,
}

fn main() -> Result<()> {
    SimpleLogger::new()
        .with_level(LevelFilter::Off)
        .with_module_level(PKG_NAME, LevelFilter::Info)
        .init()
        .with_context(|| "Failed to initalize the logger")?;

    // Parse CLI arguments
    let cli_args = Opt::from_args();
    if let Some(working_directory) = cli_args.working_directory {
        env::set_current_dir(working_directory)
            .with_context(|| "Specified working directory is invalid or inaccessible")?;
    };

    let config = match retrieve_patcher_configuration(None) {
        Ok(config) => config,
        Err(e) => {
            log::error!("Failed to retrieve patcher configuration: {}", e);
            return Ok(());
        }
    };

    let (patching_thread_tx, patching_thread_rx) = mpsc::channel();
    let config_clone = config.clone();

    std::thread::spawn(move || {
        if let Err(e) = patcher_thread_routine(config_clone, patching_thread_rx) {
            log::error!("Patcher thread error: {}", e);
        }
    });

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([config.window.width as f32, config.window.height as f32])
            .with_resizable(config.window.resizable)
            .with_title(&config.window.title),
        ..Default::default()
    };

    // Create native UI
    let native_ui = NativeUi::new(config.clone(), patching_thread_tx.clone());

    // Run native UI
    eframe::run_native(
        &config.window.title,
        native_options,
        Box::new(|_cc| Box::new(native_ui)),
    )
    .map_err(|e| anyhow!("Failed to run native UI: {}", e))
}
