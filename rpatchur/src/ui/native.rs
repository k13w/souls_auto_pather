use std::sync::mpsc;
use eframe::egui;
use crate::patcher::{PatcherCommand, PatcherConfiguration};
use crate::process::start_executable;

pub struct NativeUi {
    patcher_config: PatcherConfiguration,
    patching_thread_tx: mpsc::Sender<PatcherCommand>,
    patching_in_progress: bool,
    download_progress: f32,
    download_status: String,
    error_message: Option<String>,
    status_rx: mpsc::Receiver<PatchingStatus>,
}

impl NativeUi {
    pub fn new(patcher_config: PatcherConfiguration, patching_thread_tx: mpsc::Sender<PatcherCommand>) -> Self {
        let (status_tx, status_rx) = mpsc::channel();
        Self {
            patcher_config,
            patching_thread_tx,
            patching_in_progress: false,
            download_progress: 0.0,
            download_status: "Ready".to_string(),
            error_message: None,
            status_rx,
        }
    }

    pub fn set_patching_status(&mut self, status: PatchingStatus) {
        match status {
            PatchingStatus::Ready => {
                self.download_progress = 0.0;
                self.download_status = "Ready".to_string();
                self.error_message = None;
            }
            PatchingStatus::Error(msg) => {
                self.download_progress = 0.0;
                self.download_status = "Error".to_string();
                self.error_message = Some(msg);
            }
            PatchingStatus::DownloadInProgress(nb_downloaded, nb_total, bytes_per_sec) => {
                self.download_progress = (nb_downloaded as f32) / (nb_total as f32);
                let speed = if bytes_per_sec > 0 {
                    format!(" - {:.2} MB/s", bytes_per_sec as f32 / 1_000_000.0)
                } else {
                    String::new()
                };
                self.download_status = format!("Downloading: {}/{} {}", nb_downloaded, nb_total, speed);
            }
            PatchingStatus::InstallationInProgress(nb_installed, nb_total) => {
                self.download_progress = (nb_installed as f32) / (nb_total as f32);
                self.download_status = format!("Installing: {}/{}", nb_installed, nb_total);
            }
            PatchingStatus::ManualPatchApplied(name) => {
                self.download_progress = 0.0;
                self.download_status = format!("Patch applied: {}", name);
            }
        }
    }

    pub fn set_patching_in_progress(&mut self, value: bool) {
        self.patching_in_progress = value;
    }
}

impl eframe::App for NativeUi {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process any pending status updates
        while let Ok(status) = self.status_rx.try_recv() {
            self.set_patching_status(status);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(&self.patcher_config.window.title);
            ui.add_space(10.0);

            // Progress bar
            ui.add(egui::ProgressBar::new(self.download_progress).text(&self.download_status));
            
            if let Some(error) = &self.error_message {
                ui.add_space(5.0);
                ui.label(egui::RichText::new(error).color(egui::Color32::RED));
            }

            ui.add_space(10.0);

            // Buttons
            ui.horizontal(|ui| {
                if ui.add_enabled(!self.patching_in_progress, egui::Button::new("Start Update")).clicked() {
                    let _ = self.patching_thread_tx.send(PatcherCommand::StartUpdate);
                }

                if ui.add_enabled(self.patching_in_progress, egui::Button::new("Cancel Update")).clicked() {
                    let _ = self.patching_thread_tx.send(PatcherCommand::CancelUpdate);
                }

                if ui.add_enabled(!self.patching_in_progress, egui::Button::new("Reset Cache")).clicked() {
                    let _ = self.patching_thread_tx.send(PatcherCommand::ResetCache);
                }

                if ui.add_enabled(!self.patching_in_progress, egui::Button::new("Manual Patch")).clicked() {
                    let _ = self.patching_thread_tx.send(PatcherCommand::ManualPatch);
                }
            });

            ui.add_space(10.0);

            // Game launch buttons
            ui.horizontal(|ui| {
                if ui.button("Play").clicked() {
                    let _ = start_executable(
                        &self.patcher_config.play.path,
                        &self.patcher_config.play.arguments,
                    );
                }

                if ui.button("Setup").clicked() {
                    let _ = start_executable(
                        &self.patcher_config.setup.path,
                        &self.patcher_config.setup.arguments,
                    );
                }
            });
        });
    }
}

pub enum PatchingStatus {
    Ready,
    Error(String),
    DownloadInProgress(usize, usize, u64),
    InstallationInProgress(usize, usize),
    ManualPatchApplied(String),
} 