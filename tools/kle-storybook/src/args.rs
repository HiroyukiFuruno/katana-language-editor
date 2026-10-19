use katana_ui_core::egui::text_command_surface::FullTextCommandSurfaceMotionPlan;
use std::path::PathBuf;

const DEFAULT_SMOKE_FRAMES: usize = 120;
const MOTION_ARTIFACT_PATH: &str = "target/acceptance/kle-storybook-motion-artifact";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorybookMode {
    Interactive,
    Smoke,
    InteractionCheck,
    ContractCheck,
    MotionArtifact,
    ConsumerArtifactPlan,
}

pub struct StorybookArgs {
    pub mode: StorybookMode,
    pub frames: usize,
    frames_explicit: bool,
    pub artifact_output: PathBuf,
    pub consumer_plan_input: Option<PathBuf>,
}

impl StorybookArgs {
    pub fn parse(args: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut parsed = Self {
            mode: StorybookMode::Interactive,
            frames: DEFAULT_SMOKE_FRAMES,
            frames_explicit: false,
            artifact_output: PathBuf::from(MOTION_ARTIFACT_PATH),
            consumer_plan_input: None,
        };
        let mut args = args.peekable();
        while let Some(arg) = args.next() {
            if parsed.select_mode(&arg) {
                continue;
            }
            parsed.apply_option(&arg, &mut args)?;
        }
        Ok(parsed)
    }

    fn apply_option(
        &mut self,
        option: &str,
        args: &mut impl Iterator<Item = String>,
    ) -> Result<(), String> {
        match option {
            "--frames" => self.set_frames(next_value(args, "--frames")?),
            "--artifact-output" => self.set_artifact_output(next_value(args, "--artifact-output")?),
            "--consumer-plan-input" => {
                self.set_consumer_plan_input(next_value(args, "--consumer-plan-input")?)
            }
            value => Err(format!("unknown KLE Storybook argument: {value}")),
        }
    }

    fn set_frames(&mut self, value: String) -> Result<(), String> {
        self.frames = parse_frames(value)?;
        self.frames_explicit = true;
        Ok(())
    }

    fn set_artifact_output(&mut self, value: String) -> Result<(), String> {
        self.artifact_output = PathBuf::from(value);
        Ok(())
    }

    fn set_consumer_plan_input(&mut self, value: String) -> Result<(), String> {
        self.consumer_plan_input = Some(PathBuf::from(value));
        Ok(())
    }

    fn select_mode(&mut self, argument: &str) -> bool {
        let mode = match argument {
            "--interactive" => StorybookMode::Interactive,
            "--smoke" => StorybookMode::Smoke,
            "--interaction-check" => StorybookMode::InteractionCheck,
            "--contract-check" => StorybookMode::ContractCheck,
            "--motion-artifact" => StorybookMode::MotionArtifact,
            "--consumer-artifact-plan" => StorybookMode::ConsumerArtifactPlan,
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

    pub fn required_consumer_plan_input(&self) -> Result<&PathBuf, String> {
        self.consumer_plan_input
            .as_ref()
            .ok_or_else(|| "--consumer-artifact-plan requires --consumer-plan-input".to_string())
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
