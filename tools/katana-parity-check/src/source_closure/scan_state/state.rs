use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::Span;

use super::super::ast_resolution::span_to_text;
use super::super::model::{ClosureEdge, LexicalResolutionStatus};
use super::action::{ActionConstruction, ActionDefinition};
use super::branch::{DispatchArm, DispatchFallthrough};
use super::method::{FreeFunctionDefinition, MethodDefinition};

#[derive(Debug)]
pub struct FileRecord {
    pub sha256: String,
    pub incoming_edges: BTreeSet<String>,
    pub classification: String,
    pub classification_rationale: String,
}

#[derive(Default)]
pub struct ScanState {
    pub files: BTreeMap<String, FileRecord>,
    pub edges: Vec<ClosureEdge>,
    pub unresolved_edges: Vec<ClosureEdge>,
    pub pending_incoming_edges: BTreeMap<String, BTreeSet<String>>,
    pub action_definitions: Vec<ActionDefinition>,
    pub action_constructions: Vec<ActionConstruction>,
    pub method_definitions: Vec<MethodDefinition>,
    pub free_function_definitions: Vec<FreeFunctionDefinition>,
    pub dispatch_arms: Vec<DispatchArm>,
    pub dispatch_fallthroughs: Vec<DispatchFallthrough>,
    next_edge_id: usize,
}

impl ScanState {
    pub fn record_action_definition(
        &mut self,
        file: &str,
        symbol: &str,
        enum_span: String,
        variant: String,
        variant_span: String,
    ) {
        self.action_definitions.push(ActionDefinition {
            file: file.to_string(),
            symbol: symbol.to_string(),
            enum_span,
            variant,
            variant_span,
        });
    }

    pub fn record_action_construction(&mut self, construction: ActionConstruction) {
        self.action_constructions.push(construction);
    }

    pub fn record_method_definition(&mut self, definition: MethodDefinition) {
        self.method_definitions.push(definition);
    }

    pub fn record_free_function_definition(&mut self, definition: FreeFunctionDefinition) {
        self.free_function_definitions.push(definition);
    }

    pub fn record_dispatch_arm(&mut self, arm: DispatchArm) {
        self.dispatch_arms.push(arm);
    }

    pub fn record_dispatch_fallthrough(&mut self, fallthrough: DispatchFallthrough) {
        self.dispatch_fallthroughs.push(fallthrough);
    }

    pub fn record_file(&mut self, path: String, sha256: String) {
        let incoming_edges = self
            .pending_incoming_edges
            .remove(&path)
            .unwrap_or_default();
        self.files
            .entry(path)
            .and_modify(|file| {
                file.sha256 = sha256.clone();
                file.incoming_edges.extend(incoming_edges.iter().cloned());
            })
            .or_insert(FileRecord {
                sha256,
                incoming_edges,
                classification: "editor_behavior".to_string(),
                classification_rationale: "syntax-derived source-closure slice".to_string(),
            });
    }

    fn next_id(&mut self) -> String {
        let id = format!("edge:{:06}", self.next_edge_id);
        self.next_edge_id += 1;
        id
    }

    pub fn add_edge(
        &mut self,
        from_file: &str,
        kind: &str,
        from_symbol: &str,
        to_path: Option<&str>,
        to_symbol: Option<&str>,
        span: Span,
    ) -> String {
        let id = self.next_id();
        let edge = ClosureEdge {
            id: id.clone(),
            from_file: from_file.to_string(),
            kind: kind.to_string(),
            from_symbol: from_symbol.to_string(),
            to_path: to_path.map(std::string::ToString::to_string),
            to_symbol: to_symbol.map(std::string::ToString::to_string),
            lexical_resolution: None,
            span: span_to_text(from_file, &span),
        };
        self.edges.push(edge.clone());
        if let Some(target) = &edge.to_path {
            if !self.files.contains_key(target) {
                self.pending_incoming_edges
                    .entry(target.clone())
                    .or_default()
                    .insert(id.clone());
            }
        } else {
            self.unresolved_edges.push(edge);
        }
        id
    }

    pub fn set_lexical_resolution(&mut self, edge_id: &str, status: LexicalResolutionStatus) {
        set_edge_resolution(&mut self.edges, edge_id, &status);
        set_edge_resolution(&mut self.unresolved_edges, edge_id, &status);
    }

    pub fn mark_target_unresolved(&mut self, target: &str, reason: &std::io::Error) {
        let Some(edge_ids) = self.pending_incoming_edges.remove(target) else {
            return;
        };
        for edge_id in edge_ids {
            if let Some(edge) = self.edges.iter().find(|edge| edge.id == edge_id) {
                let mut unresolved = edge.clone();
                unresolved.to_symbol = Some(format!(
                    "{} (target materialization failed: {reason})",
                    unresolved.to_symbol.as_deref().unwrap_or("unknown")
                ));
                self.unresolved_edges.push(unresolved);
            }
        }
    }

    pub fn finalize_unscanned_targets(&mut self) {
        let pending = std::mem::take(&mut self.pending_incoming_edges);
        for edge_ids in pending.into_values() {
            for edge_id in edge_ids {
                if let Some(edge) = self.edges.iter().find(|edge| edge.id == edge_id) {
                    self.unresolved_edges.push(edge.clone());
                }
            }
        }
    }
}

fn set_edge_resolution(edges: &mut [ClosureEdge], edge_id: &str, status: &LexicalResolutionStatus) {
    if let Some(edge) = edges.iter_mut().find(|edge| edge.id == edge_id) {
        edge.lexical_resolution = Some(status.clone());
    }
}
