use std::collections::BTreeSet;

use super::context_menu_contract_entries::ContextMenuContractEntries;

const ROOT_SLOT_ORDER: [Option<usize>; 4] = [Some(0), Some(1), Some(2), Some(3)];
const DIRECT_MARKDOWN_LEAF_COUNT: usize = 13;
const NESTED_CODE_LEAF_COUNT: usize = 17;
const VISIBLE_INGEST_LEAF_COUNT: usize = 2;
const ACTUAL_INPUT_ROUTE_COUNT: usize = 3;

pub(super) struct ContextMenuInventoryValidator;

impl ContextMenuInventoryValidator {
    pub(super) fn validate(
        ids: &BTreeSet<&'static str>,
        root_slots: &[(Option<usize>, &'static str)],
    ) -> Result<(), String> {
        if root_slots.len() != ROOT_SLOT_ORDER.len()
            || root_slots.iter().map(|(slot, _)| *slot).collect::<Vec<_>>() != ROOT_SLOT_ORDER
        {
            return Err(
                "context-menu root order must be Save -> conditional Format -> Edit -> Ingest"
                    .to_string(),
            );
        }
        Self::require_invariants(ids)?;
        Self::require_count(
            ids,
            "context.structure.edit.",
            DIRECT_MARKDOWN_LEAF_COUNT,
            |id| {
                id != "context.structure.edit.parent-path"
                    && id != "context.structure.edit.code-submenu"
            },
            "13 direct Markdown leaves",
        )?;
        Self::require_count(
            ids,
            "context.structure.code.",
            NESTED_CODE_LEAF_COUNT,
            |_| true,
            "17 nested code leaves",
        )?;
        Self::require_count(
            ids,
            "context.structure.ingest.",
            VISIBLE_INGEST_LEAF_COUNT,
            |id| id != "context.structure.ingest.parent-path",
            "2 visible ingest leaves",
        )?;
        Self::require_count(
            ids,
            "context.structure.route.",
            ACTUAL_INPUT_ROUTE_COUNT,
            |_| true,
            "secondary, Shift+F10, and AccessKit routes",
        )?;
        (ids == &ContextMenuContractEntries::expected_leaf_ids())
            .then_some(())
            .ok_or_else(|| {
                "context-menu structural leaf IDs do not match the required inventory".to_string()
            })
    }

    pub(super) fn expected_parent_path(id: &str) -> Option<&'static str> {
        if id.starts_with("context.structure.root.")
            || id.starts_with("context.structure.route.")
            || id == "context.structure.overflow.visible-record-only"
        {
            return Some("root");
        }
        if id == "context.structure.format.editable-markdown-only" {
            return Some("root/Format");
        }
        if id == "context.structure.edit.parent-path" || id.starts_with("context.structure.edit.") {
            return Some("root/Edit");
        }
        if id == "context.structure.ingest.parent-path"
            || id.starts_with("context.structure.ingest.")
        {
            return Some("root/Ingest");
        }
        id.starts_with("context.structure.code.")
            .then_some("root/Edit/Code")
    }

    fn require_invariants(ids: &BTreeSet<&'static str>) -> Result<(), String> {
        [
            "context.structure.format.editable-markdown-only",
            "context.structure.edit.parent-path",
            "context.structure.ingest.parent-path",
            "context.structure.overflow.visible-record-only",
        ]
        .iter()
        .all(|id| ids.contains(id))
        .then_some(())
        .ok_or_else(|| {
            "context-menu structural inventory is missing format, parent path, or overflow invariant"
                .to_string()
        })
    }

    fn require_count(
        ids: &BTreeSet<&'static str>,
        prefix: &str,
        expected: usize,
        keep: impl Fn(&str) -> bool,
        description: &str,
    ) -> Result<(), String> {
        (ids.iter()
            .filter(|id| id.starts_with(prefix) && keep(id))
            .count()
            == expected)
            .then_some(())
            .ok_or_else(|| {
                format!("context-menu structural inventory must contain exactly {description}")
            })
    }
}
