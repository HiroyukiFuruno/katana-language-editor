use katana_ui_core::egui::text_command_surface::FullTextCommandSurfaceMotionPlan;
use std::path::PathBuf;

const DEFAULT_SMOKE_FRAMES: usize = 120;
const MOTION_ARTIFACT_PATH: &str = "target/acceptance/kle-storybook-motion-artifact";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorybookMode {
    Interactive,
    Smoke,
    InteractionCheck,
    EmojiCheck,
    ContractCheck,
    MotionArtifact,
}

pub struct StorybookArgs {
    pub mode: StorybookMode,
    pub frames: usize,
    frames_explicit: bool,
    pub artifact_output: PathBuf,
}

impl StorybookArgs {
    pub fn parse(args: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut parsed = Self {
            mode: StorybookMode::Interactive,
            frames: DEFAULT_SMOKE_FRAMES,
            frames_explicit: false,
            artifact_output: PathBuf::from(MOTION_ARTIFACT_PATH),
        };
        let mut args = args.peekable();
        while let Some(arg) = args.next() {
            if parsed.select_mode(&arg) {
                continue;
            }
            match arg.as_str() {
                "--frames" => {
                    parsed.frames = parse_frames(next_value(&mut args, "--frames")?)?;
                    parsed.frames_explicit = true;
                }
                "--artifact-output" => {
                    parsed.artifact_output =
                        PathBuf::from(next_value(&mut args, "--artifact-output")?);
                }
                value => return Err(format!("unknown KLE Storybook argument: {value}")),
            }
        }
        Ok(parsed)
    }

    fn select_mode(&mut self, argument: &str) -> bool {
        let mode = match argument {
            "--interactive" => StorybookMode::Interactive,
            "--smoke" => StorybookMode::Smoke,
            "--interaction-check" => StorybookMode::InteractionCheck,
            "--emoji-check" => StorybookMode::EmojiCheck,
            "--contract-check" => StorybookMode::ContractCheck,
            "--motion-artifact" => StorybookMode::MotionArtifact,
            _ => return false,
        };
        self.mode = mode;
        true
    }

    pub fn resolved_artifact_output(&self) -> PathBuf {
        self.artifact_output.clone()
    }

    pub fn resolved_frames(&self) -> usize {
        if self.frames_explicit {
            return self.frames;
        }
        match self.mode {
            StorybookMode::MotionArtifact => default_motion_artifact_frames(),
            _ => self.frames,
        }
    }

    pub fn interactive_frame_limit(&self) -> Option<usize> {
        self.frames_explicit.then_some(self.frames)
    }
}

fn default_motion_artifact_frames() -> usize {
    FullTextCommandSurfaceMotionPlan::minimum_frame_count() * 2
}

fn next_value(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("missing value for KLE Storybook argument: {name}"))
}

fn parse_frames(value: String) -> Result<usize, String> {
    let frames = value
        .parse::<usize>()
        .map_err(|source| format!("invalid --frames value `{value}`: {source}"))?;
    if frames == 0 {
        return Err("--frames must be greater than 0".to_string());
    }
    Ok(frames)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_interaction_check() -> Result<(), String> {
        let args = StorybookArgs::parse(["--interaction-check"].into_iter().map(str::to_string))?;
        assert_eq!(args.mode, StorybookMode::InteractionCheck);
        Ok(())
    }

    #[test]
    fn resolves_default_motion_sequence_output() -> Result<(), String> {
        let args = StorybookArgs::parse(["--motion-artifact"].into_iter().map(str::to_string))?;
        assert_eq!(args.mode, StorybookMode::MotionArtifact);
        assert!(
            args.resolved_artifact_output()
                .ends_with("kle-storybook-motion-artifact")
        );
        assert_eq!(args.resolved_frames(), default_motion_artifact_frames());
        Ok(())
    }

    #[test]
    fn rejects_the_removed_full_acceptance_alias() {
        let result = StorybookArgs::parse(
            ["--live-acceptance-artifact"]
                .into_iter()
                .map(str::to_string),
        );
        assert!(matches!(
            result.as_ref(),
            Err(error) if error.contains("unknown KLE Storybook argument")
        ));
    }

    #[test]
    fn interactive_defaults_to_unlimited_frames() -> Result<(), String> {
        let args = StorybookArgs::parse(std::iter::empty())?;
        assert_eq!(args.mode, StorybookMode::Interactive);
        assert_eq!(args.interactive_frame_limit(), None);
        Ok(())
    }

    #[test]
    fn explicit_interactive_frames_are_a_positive_limit() -> Result<(), String> {
        let args = StorybookArgs::parse(
            ["--interactive", "--frames", "1"]
                .into_iter()
                .map(str::to_string),
        )?;
        assert_eq!(args.interactive_frame_limit(), Some(1));
        Ok(())
    }

    #[test]
    fn rejects_zero_interactive_frames() {
        let result = StorybookArgs::parse(
            ["--interactive", "--frames", "0"]
                .into_iter()
                .map(str::to_string),
        );
        assert!(matches!(result, Err(error) if error.contains("greater than 0")));
    }
}
