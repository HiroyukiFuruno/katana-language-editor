use super::context_menu_contract_entry_types::LeafSpec;

const MENU: &str = "crates/katana-ui/src/views/panels/editor/context_menu.rs";
const INGEST: &str = "crates/katana-ui/src/views/panels/editor/context_menu_image_ingest.rs";
const TEXT_EDIT: &str = "crates/katana-ui/src/views/panels/editor/text_edit.rs";

pub(super) const ROUTE_AND_INGEST_SPECS: &[LeafSpec] = &[
    LeafSpec {
        id: "context.structure.ingest.file",
        parent_path: "root/Ingest",
        root_slot: None,
        path: INGEST,
        line: 13,
        marker: "AppAction::IngestImageFile",
    },
    LeafSpec {
        id: "context.structure.ingest.clipboard-image",
        parent_path: "root/Ingest",
        root_slot: None,
        path: INGEST,
        line: 20,
        marker: "AppAction::IngestClipboardImage",
    },
    LeafSpec {
        id: "context.structure.route.secondary",
        parent_path: "root",
        root_slot: None,
        path: TEXT_EDIT,
        line: 69,
        marker: "EditorContextMenu::render",
    },
    LeafSpec {
        id: "context.structure.route.shift-f10",
        parent_path: "root",
        root_slot: None,
        path: TEXT_EDIT,
        line: 69,
        marker: "EditorContextMenu::render",
    },
    LeafSpec {
        id: "context.structure.route.accesskit",
        parent_path: "root",
        root_slot: None,
        path: TEXT_EDIT,
        line: 69,
        marker: "EditorContextMenu::render",
    },
    LeafSpec {
        id: "context.structure.overflow.visible-record-only",
        parent_path: "root",
        root_slot: None,
        path: MENU,
        line: 9,
        marker: "pub(crate) fn render",
    },
];
