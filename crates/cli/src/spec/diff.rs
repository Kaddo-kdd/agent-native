//! Semantic specification diffing and impact analysis.
//!
//! Provides [`SpecDiff`] and [`compute_spec_diff`] to detect added, modified,
//! and deprecated requirements, affected components, modified operations,
//! entities, and contract interface modifications across specification versions.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::spec::model::{RequirementSpec, SystemSpec};

/// Semantic diff between two `SystemSpec` versions (e.g. `v_n` -> `v_{n+1}`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SpecDiff {
    /// Requirements present in `after` but not in `before`.
    pub added_requirements: Vec<String>,
    /// Requirements present in both whose statement or version changed.
    pub modified_requirements: Vec<String>,
    /// Requirements present in `before` but missing in `after`.
    pub deprecated_requirements: Vec<String>,
    /// IDs of components affected by requirement or contract changes.
    pub affected_components: Vec<String>,
    /// Operation names whose signature or referenced rules changed.
    pub modified_operations: Vec<String>,
    /// Entity names whose fields or definitions changed.
    pub modified_entities: Vec<String>,
    /// Component IDs whose WIT exported or imported interfaces changed.
    pub modified_contracts: Vec<String>,
}

impl SpecDiff {
    /// Returns `true` if there are no semantic differences between the specifications.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.added_requirements.is_empty()
            && self.modified_requirements.is_empty()
            && self.deprecated_requirements.is_empty()
            && self.affected_components.is_empty()
            && self.modified_operations.is_empty()
            && self.modified_entities.is_empty()
            && self.modified_contracts.is_empty()
    }
}

/// Computes the semantic difference between two specification versions.
#[must_use]
pub fn compute_spec_diff(before: &SystemSpec, after: &SystemSpec) -> SpecDiff {
    let mut diff = SpecDiff::default();

    let before_map: HashMap<String, &RequirementSpec> = before
        .requirements
        .iter()
        .map(|r| (r.id.clone(), r))
        .collect();
    let after_map: HashMap<String, &RequirementSpec> = after
        .requirements
        .iter()
        .map(|r| (r.id.clone(), r))
        .collect();

    // Check added and modified requirements
    for (id, req) in &after_map {
        match before_map.get(id) {
            None => {
                diff.added_requirements.push(id.clone());
                if !diff.affected_components.contains(&req.component) {
                    diff.affected_components.push(req.component.clone());
                }
            }
            Some(old_req)
                if old_req.statement != req.statement || old_req.version != req.version =>
            {
                diff.modified_requirements.push(id.clone());
                if !diff.affected_components.contains(&req.component) {
                    diff.affected_components.push(req.component.clone());
                }
            }
            _ => {}
        }
    }

    // Check deprecated requirements (present in before, missing in after)
    for (id, req) in &before_map {
        if !after_map.contains_key(id) {
            diff.deprecated_requirements.push(id.clone());
            if !diff.affected_components.contains(&req.component) {
                diff.affected_components.push(req.component.clone());
            }
        }
    }

    // Check operation changes
    for op in &after.operations {
        if !before.operations.iter().any(|b| {
            b.name == op.name
                && b.component == op.component
                && b.params == op.params
                && b.returns == op.returns
                && b.rules_applied == op.rules_applied
        }) {
            diff.modified_operations.push(op.name.clone());
            if !diff.affected_components.contains(&op.component) {
                diff.affected_components.push(op.component.clone());
            }
        }
    }

    // Check entity changes
    for ent in &after.entities {
        if !before
            .entities
            .iter()
            .any(|b| b.name == ent.name && b.fields == ent.fields)
        {
            diff.modified_entities.push(ent.name.clone());
        }
    }

    // Check contract/WIT changes
    for comp in &after.components {
        if let Some(b) = before.components.iter().find(|b| b.id == comp.id) {
            if b.export_interfaces != comp.export_interfaces
                || b.import_interfaces != comp.import_interfaces
                || b.wit_file != comp.wit_file
            {
                diff.modified_contracts.push(comp.id.clone());
                if !diff.affected_components.contains(&comp.id) {
                    diff.affected_components.push(comp.id.clone());
                }
            }
        } else if !diff.affected_components.contains(&comp.id) {
            diff.affected_components.push(comp.id.clone());
        }
    }

    diff.added_requirements.sort();
    diff.modified_requirements.sort();
    diff.deprecated_requirements.sort();
    diff.affected_components.sort();
    diff.modified_operations.sort();
    diff.modified_entities.sort();
    diff.modified_contracts.sort();

    diff
}
