use crate::host_types::{STORYBOOK_VIEWPORT_HEIGHT, STORYBOOK_VIEWPORT_WIDTH, StorybookHost};
use eframe::egui;
use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct StorybookWindow {
    frames: usize,
}

impl StorybookWindow {
    pub const fn new(frames: usize) -> Self {
        Self { frames }
    }

    pub fn run(
        self,
        host: StorybookHost,
        frame_limit: Option<usize>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let lifecycle_error = Arc::new(Mutex::new(None));
        let app = StorybookWindowApp {
            host,
            frame_limit,
            frames_rendered: 0,
            lifecycle_error: Arc::clone(&lifecycle_error),
        };
        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([
                STORYBOOK_VIEWPORT_WIDTH as f32,
                STORYBOOK_VIEWPORT_HEIGHT as f32,
            ]),
            ..eframe::NativeOptions::default()
        };
        eframe::run_native(
            "KLE Storybook",
            native_options,
            Box::new(|_| Ok(Box::new(app))),
        )
        .map_err(|error| Box::new(error) as Box<dyn std::error::Error>)?;

        take_lifecycle_error(&lifecycle_error)
    }

    pub fn run_smoke(&self, mut host: StorybookHost) -> Result<(), Box<dyn std::error::Error>> {
        host.write_live_artifact(Path::new("target/kle-storybook-smoke"), self.frames)
    }
}

fn take_lifecycle_error(
    lifecycle_error: &Mutex<Option<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    match lifecycle_error.lock() {
        Ok(mut error) => match error.take() {
            Some(error) => Err(io::Error::other(error).into()),
            None => Ok(()),
        },
        Err(poisoned) => {
            let error = match poisoned.into_inner().take() {
                Some(error) => error,
                None => "interactive Storybook lifecycle error state was poisoned".to_string(),
            };
            Err(io::Error::other(error).into())
        }
    }
}

struct StorybookWindowApp {
    host: StorybookHost,
    frame_limit: Option<usize>,
    frames_rendered: usize,
    lifecycle_error: Arc<Mutex<Option<String>>>,
}

impl eframe::App for StorybookWindowApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut rendering_error = None;
        egui::CentralPanel::default().show(ui, |ui| {
            if let Err(error) = self.host.show_interactive(ui) {
                rendering_error = Some(error.to_string());
            }
        });

        if let Some(error) = rendering_error {
            match self.lifecycle_error.lock() {
                Ok(mut slot) => *slot = Some(error),
                Err(poisoned) => *poisoned.into_inner() = Some(error),
            }
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        self.frames_rendered = self.frames_rendered.saturating_add(1);
        match self.frame_limit {
            Some(_) if reached_frame_limit(self.frame_limit, self.frames_rendered) => {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Some(_) => ui.ctx().request_repaint(),
            None => {}
        }
    }
}

fn reached_frame_limit(limit: Option<usize>, rendered: usize) -> bool {
    limit.is_some_and(|limit| rendered >= limit)
}

#[cfg(test)]
mod tests {
    use super::{reached_frame_limit, take_lifecycle_error};
    use std::sync::Mutex;

    #[test]
    fn rendering_failure_is_not_a_successful_exit() {
        let result = take_lifecycle_error(&Mutex::new(Some("render failed".to_string())));
        assert!(matches!(result, Err(error) if error.to_string() == "render failed"));
        assert!(take_lifecycle_error(&Mutex::new(None)).is_ok());
    }

    #[test]
    fn interactive_default_never_reaches_a_frame_limit() {
        for frames in [0, 120, 121, usize::MAX] {
            assert!(!reached_frame_limit(None, frames));
        }
    }

    #[test]
    fn explicit_limit_closes_only_at_its_boundary() {
        assert!(!reached_frame_limit(Some(2), 1));
        assert!(reached_frame_limit(Some(2), 2));
        assert!(reached_frame_limit(Some(2), 3));
    }
}
