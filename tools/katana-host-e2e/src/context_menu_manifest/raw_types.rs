#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawManifest {
    schema_version: String,
    generated_by: String,
    katana_revision: String,
    profile_fingerprint: String,
    source_closure_fingerprint: String,
    surface: Surface,
    menu: Menu,
    leaves: Vec<Leaf>,
    routes: Vec<Route>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Surface {
    source_span_digest: String,
    role: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Menu {
    source_span_digest: String,
    role: String,
    path_digest: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Leaf {
    path_digest: String,
    source_span_digest: String,
    role: String,
    parent_path_digest: String,
    enabled_condition_digest: String,
    locale_label_digests: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
enum Route {
    SecondaryPointer,
    ShiftF10,
    AccesskitInvoke,
}

#[derive(Deserialize)]
struct ProfileRoot {
    root: serde_json::Value,
}
