include!("context_menu_manifest/preamble.rs");
include!("context_menu_manifest/types.rs");
include!("context_menu_manifest/raw_types.rs");
include!("context_menu_manifest/logic.rs");
include!("context_menu_manifest/expected.rs");
include!("context_menu_manifest/helpers.rs");

#[cfg(test)]
mod tests {
    include!("context_menu_manifest/tests.rs");
}
