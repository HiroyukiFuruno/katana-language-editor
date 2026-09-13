use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fmt;
use std::path::Path;

const FIXED_REVISION: &str = "4f6a6287c650a38633c7baeb544a92e739c68567";
const SCHEMA_VERSION: &str = "1";
const GENERATED_BY: &str = "katana-source-closure-context-menu-generator";
const CONTEXT_SOURCE: &str = "crates/katana-ui/src/views/panels/editor/context_menu.rs";
const CODE_SOURCE: &str = "crates/katana-ui/src/markdown_authoring_op.rs";
const INGEST_SOURCE: &str = "crates/katana-ui/src/views/panels/editor/context_menu_image_ingest.rs";
const LOCALES: &str = "crates/katana-ui/locales";
const MENU_PATH: &str = "editor.text_surface/context_menu";
const LEAF_COUNT: usize = 34;
const ROUTE_COUNT: usize = 3;
const AUTHORING_COUNT: usize = 13;
const CODE_KIND_COUNT: usize = 17;
const DIGEST_HEX_LENGTH: usize = 64;
