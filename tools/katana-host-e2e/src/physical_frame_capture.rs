use eframe36::{App, egui::FullOutput};
use sha2::{Digest, Sha256};
use std::fmt;

use crate::host_target_locator::{CurrentHostTarget, HostTargetLocator, HostTargetSelectionError};

const SHA256_DIGEST_BYTES: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccessKitMode {
    Required,
    Disabled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccessKitEvidence {
    pub root_hash: [u8; SHA256_DIGEST_BYTES],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameEvidence {
    pub frame_hash: [u8; SHA256_DIGEST_BYTES],
    pub accesskit: AccessKitEvidence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameCaptureError {
    Disabled,
    TreeUpdateMissing,
    RootMissing,
    Target(HostTargetSelectionError),
}

impl fmt::Display for FrameCaptureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Disabled => "AccessKit was not enabled for the frame",
            Self::TreeUpdateMissing => "the frame had no AccessKit TreeUpdate",
            Self::RootMissing => "the AccessKit TreeUpdate had no root node",
            Self::Target(error) => {
                return write!(formatter, "current AccessKit target failed: {error}");
            }
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for FrameCaptureError {}

pub struct PhysicalFrameCapture {
    accesskit: AccessKitMode,
}

impl PhysicalFrameCapture {
    pub const fn new(accesskit: AccessKitMode) -> Self {
        Self { accesskit }
    }

    pub fn capture_katana_frame<A: App>(
        &self,
        app: &mut A,
        context: &eframe36::egui::Context,
        raw_input: eframe36::egui::RawInput,
    ) -> Result<FrameEvidence, FrameCaptureError> {
        self.capture_katana_frame_with_target(app, context, raw_input, None)
            .map(|(evidence, _)| evidence)
    }

    pub(crate) fn capture_katana_frame_with_target<A: App>(
        &self,
        app: &mut A,
        context: &eframe36::egui::Context,
        raw_input: eframe36::egui::RawInput,
        locator: Option<&HostTargetLocator>,
    ) -> Result<(FrameEvidence, Option<CurrentHostTarget>), FrameCaptureError> {
        let mut frame = eframe36::Frame::_new_kittest();
        let output = context.run_ui(raw_input, |ui| App::ui(app, ui, &mut frame));
        self.capture_with_target(output, locator)
    }

    pub fn capture(&self, output: FullOutput) -> Result<FrameEvidence, FrameCaptureError> {
        self.capture_with_target(output, None)
            .map(|(evidence, _)| evidence)
    }

    pub(crate) fn capture_with_target(
        &self,
        mut output: FullOutput,
        locator: Option<&HostTargetLocator>,
    ) -> Result<(FrameEvidence, Option<CurrentHostTarget>), FrameCaptureError> {
        output.textures_delta.clear();
        if self.accesskit == AccessKitMode::Disabled {
            return Err(FrameCaptureError::Disabled);
        }

        let frame_hash = digest_debug(&output.shapes);
        let update = output
            .platform_output
            .accesskit_update
            .ok_or(FrameCaptureError::TreeUpdateMissing)?;
        let root_id = update
            .tree
            .as_ref()
            .map(|tree| tree.root)
            .ok_or(FrameCaptureError::RootMissing)?;
        let root = update
            .nodes
            .iter()
            .find_map(|(node_id, node)| (*node_id == root_id).then_some(node))
            .ok_or(FrameCaptureError::RootMissing)?;

        let target = locator
            .map(|locator| {
                locator
                    .select_current(&update, frame_hash)
                    .map_err(FrameCaptureError::Target)
            })
            .transpose()?;
        Ok((
            FrameEvidence {
                frame_hash,
                accesskit: AccessKitEvidence {
                    root_hash: digest_debug(root),
                },
            },
            target,
        ))
    }
}

fn digest_debug<T: fmt::Debug>(value: &T) -> [u8; SHA256_DIGEST_BYTES] {
    let mut hasher = Sha256::new();
    hasher.update(format!("{value:?}").as_bytes());
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::{AccessKitMode, FrameCaptureError, PhysicalFrameCapture, SHA256_DIGEST_BYTES};
    use accesskit::{Node, NodeId, Role, Tree, TreeId, TreeUpdate};
    use eframe36::egui::{FullOutput, PlatformOutput};

    fn output_with_update() -> FullOutput {
        let root = NodeId(1);
        let node = Node::new(Role::Window);
        FullOutput {
            platform_output: PlatformOutput {
                accesskit_update: Some(TreeUpdate {
                    nodes: vec![(root, node)],
                    tree: Some(Tree::new(root)),
                    tree_id: TreeId::ROOT,
                    focus: root,
                }),
                ..PlatformOutput::default()
            },
            ..FullOutput::default()
        }
    }

    #[test]
    fn captures_accesskit_root_without_exposing_tree() {
        let evidence = PhysicalFrameCapture::new(AccessKitMode::Required)
            .capture(output_with_update())
            .expect("valid AccessKit frame");
        assert_ne!(evidence.frame_hash, [0; SHA256_DIGEST_BYTES]);
        assert_ne!(evidence.accesskit.root_hash, [0; SHA256_DIGEST_BYTES]);
    }

    #[test]
    fn disabled_accesskit_is_typed_error() {
        assert_eq!(
            PhysicalFrameCapture::new(AccessKitMode::Disabled).capture(FullOutput::default()),
            Err(FrameCaptureError::Disabled)
        );
    }

    #[test]
    fn missing_tree_update_is_typed_error() {
        assert_eq!(
            PhysicalFrameCapture::new(AccessKitMode::Required).capture(FullOutput::default()),
            Err(FrameCaptureError::TreeUpdateMissing)
        );
    }
}
