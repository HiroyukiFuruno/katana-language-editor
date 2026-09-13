use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

const FIXED_REVISION: &str = "4f6a6287c650a38633c7baeb544a92e739c68567";
const SOURCE_CASE_COUNT: usize = 6;
const CONTEXT_MENU_SOURCE: &str = "crates/katana-ui/src/views/panels/editor/context_menu.rs";
const CONTEXT_MENU_INGEST_SOURCE: &str =
    "crates/katana-ui/src/views/panels/editor/context_menu_image_ingest.rs";
