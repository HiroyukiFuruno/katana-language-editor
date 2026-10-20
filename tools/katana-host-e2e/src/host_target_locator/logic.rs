impl HostTargetLocator {
    pub fn from_accessible_name(role: Role, name: &str) -> Self {
        let digest = sha256(name.as_bytes());
        Self {
            role,
            accessible_name_sha256: digest,
            accessible_name_candidates: vec![digest],
            requires_menu_ancestor: false,
        }
    }

    pub fn from_context_menu_case(
        descriptor: &ContextMenuCaseDescriptor,
    ) -> Result<Self, HostTargetSelectionError> {
        let candidates = descriptor
            .locale_label_digests
            .iter()
            .map(|digest| parse_digest(digest))
            .collect::<Result<Vec<_>, _>>()?;
        let first = candidates
            .first()
            .copied()
            .ok_or(HostTargetSelectionError::InvalidDescriptor)?;
        Ok(Self {
            role: Role::Button,
            accessible_name_sha256: first,
            accessible_name_candidates: candidates,
            requires_menu_ancestor: true,
        })
    }

    pub(crate) fn select_current(
        &self,
        update: &TreeUpdate,
        frame_hash: [u8; SHA256_DIGEST_BYTES],
    ) -> Result<CurrentHostTarget, HostTargetSelectionError> {
        let parents = parent_map(update);
        let mut matching: Option<(NodeId, &Node)> = None;
        let mut role_seen = false;
        let mut menu_candidate_seen = false;
        for (node_id, node) in &update.nodes {
            if node.role() != self.role {
                continue;
            }
            role_seen = true;
            if self.requires_menu_ancestor && !has_menu_ancestor(*node_id, update, &parents) {
                continue;
            }
            menu_candidate_seen = true;
            let name = node
                .label()
                .filter(|value| !value.is_empty())
                .ok_or(HostTargetSelectionError::AccessibleNameMissing)?;
            if !self
                .accessible_name_candidates
                .iter()
                .any(|digest| sha256(name.as_bytes()) == *digest)
            {
                continue;
            }
            if matching.replace((*node_id, node)).is_some() {
                return Err(HostTargetSelectionError::AmbiguousTarget);
            }
        }
        let (node_id, node) = matching.ok_or(
            if self.requires_menu_ancestor && role_seen && !menu_candidate_seen {
                HostTargetSelectionError::MenuAncestorMissing
            } else if role_seen {
                HostTargetSelectionError::AccessibleNameMismatch
            } else {
                HostTargetSelectionError::MissingTarget
            },
        )?;
        if node.is_disabled() {
            return Err(HostTargetSelectionError::DisabledTarget);
        }
        let bounds = node
            .bounds()
            .filter(|bounds| {
                bounds.x0.is_finite()
                    && bounds.y0.is_finite()
                    && bounds.x1.is_finite()
                    && bounds.y1.is_finite()
                    && bounds.x1 >= bounds.x0
                    && bounds.y1 >= bounds.y0
            })
            .ok_or(HostTargetSelectionError::BoundsMissing)?;
        Ok(CurrentHostTarget {
            frame_hash,
            node_id,
            bounds,
        })
    }
}

impl fmt::Debug for HostTargetLocator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HostTargetLocator")
            .field("role", &self.role)
            .field("accessible_name_sha256", &self.accessible_name_sha256)
            .finish()
    }
}
fn sha256(bytes: &[u8]) -> [u8; SHA256_DIGEST_BYTES] {
    Sha256::digest(bytes).into()
}

fn parse_digest(value: &str) -> Result<[u8; SHA256_DIGEST_BYTES], HostTargetSelectionError> {
    const PREFIX: &str = "sha256:";
    let payload = value
        .strip_prefix(PREFIX)
        .filter(|payload| payload.len() == SHA256_DIGEST_BYTES * 2)
        .filter(|payload| {
            payload
                .bytes()
                .all(|byte| byte.is_ascii_digit() || byte.is_ascii_lowercase() && byte <= b'f')
        })
        .ok_or(HostTargetSelectionError::InvalidDescriptor)?;
    if payload.len() != SHA256_DIGEST_BYTES * 2 {
        return Err(HostTargetSelectionError::InvalidDescriptor);
    }
    let mut bytes = [0; SHA256_DIGEST_BYTES];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(
            &payload
                [index * HEX_BYTES_PER_DIGEST..index * HEX_BYTES_PER_DIGEST + HEX_BYTES_PER_DIGEST],
            HEX_RADIX,
        )
        .map_err(|_| HostTargetSelectionError::InvalidDescriptor)?;
    }
    Ok(bytes)
}

fn parent_map(update: &TreeUpdate) -> HashMap<NodeId, Vec<NodeId>> {
    let mut parents = HashMap::new();
    for (parent_id, node) in &update.nodes {
        for child_id in node.children() {
            parents
                .entry(*child_id)
                .or_insert_with(Vec::new)
                .push(*parent_id);
        }
    }
    parents
}

fn has_menu_ancestor(
    node_id: NodeId,
    update: &TreeUpdate,
    parents: &HashMap<NodeId, Vec<NodeId>>,
) -> bool {
    let mut current = vec![node_id];
    let mut visited = Vec::new();
    while let Some(id) = current.pop() {
        if visited.contains(&id) {
            return false;
        }
        visited.push(id);
        for parent_id in parents.get(&id).into_iter().flatten() {
            let Some(parent) = update
                .nodes
                .iter()
                .find_map(|(candidate, node)| (*candidate == *parent_id).then_some(node))
            else {
                return false;
            };
            if parent.role() == Role::Menu {
                return true;
            }
            current.push(*parent_id);
        }
    }
    false
}
