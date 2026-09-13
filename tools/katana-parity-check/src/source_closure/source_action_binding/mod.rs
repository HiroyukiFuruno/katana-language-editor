mod build;
mod facts;
mod order;
mod routes;
mod span;
mod types;

pub(crate) use types::{
    SourceActionBindingReport, SourceActionFact, SourceActionRouteCandidate, SourceActionUnresolved,
};

impl SourceActionBindingReport {
    pub(crate) fn binding_id(binding: &super::requirement_binding::RequirementBinding) -> String {
        facts::binding_id(binding)
    }

    pub(crate) fn validate_fact_span(fact: &SourceActionFact) -> Result<(), String> {
        facts::validate_fact_span(fact)
    }

    pub(crate) fn route_construction_is_within_binding(
        route: &SourceActionRouteCandidate,
        binding: &super::requirement_binding::RequirementBinding,
    ) -> Result<bool, String> {
        routes::route_construction_is_within_binding(route, binding)
    }
}

#[cfg(test)]
#[path = "../source_action_binding_tests/mod.rs"]
pub(in crate::source_closure) mod tests;
