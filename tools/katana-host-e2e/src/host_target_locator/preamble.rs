use accesskit::{Node, NodeId, Role, TreeUpdate};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt;

use crate::context_menu_manifest::ContextMenuCaseDescriptor;

pub const SHA256_DIGEST_BYTES: usize = 32;
const HEX_BYTES_PER_DIGEST: usize = 2;
const HEX_RADIX: u32 = 16;
