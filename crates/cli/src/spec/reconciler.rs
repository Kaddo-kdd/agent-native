//! Incremental specification reconciler.
//!
//! Reconciles deltas, Work Items, and requirement updates against a base
//! [`SystemSpec`], enforcing strictly monotonic versioning, detecting conflicts,
//! maintaining broken reference safety, and preserving cumulative behavior.

use serde::Deserialize;

use crate::spec::error::SpecError;
use crate::spec::model::{RequirementSource, RequirementSpec, SystemSpec};

/// Metadata parsed from the frontmatter of a Work Item delta.
#[derive(Debug, Clone, Deserialize, Default)]
struct WorkItemFrontmatter {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub component: Option<String>,
    #[serde(default)]
    pub replaces: Option<String>,
    #[serde(default = "default_status")]
    pub status: String,
}

fn default_version() -> u32 {
    1
}

fn default_status() -> String {
    "active".to_string()
}

/// Parses a single Work Item or requirement delta into its frontmatter and requirements.
#[allow(clippy::too_many_lines)]
fn parse_work_item_delta(
    filename: &str,
    content: &str,
) -> (WorkItemFrontmatter, Vec<RequirementSpec>) {
    let mut fm = WorkItemFrontmatter::default();
    let mut body_lines = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    if lines.first().is_some_and(|l| l.trim() == "---") {
        let mut in_fm = true;
        let mut fm_lines = Vec::new();
        for &line in lines.iter().skip(1) {
            if in_fm {
                if line.trim() == "---" {
                    in_fm = false;
                } else {
                    fm_lines.push(line);
                }
            } else {
                body_lines.push(line);
            }
        }

        // Lightweight key-value parsing for frontmatter without external yaml parser
        for line in fm_lines {
            let trimmed = line.trim();
            if let Some(colon) = trimmed.find(':') {
                let key = trimmed[..colon].trim();
                let val = trimmed[colon + 1..]
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                match key {
                    "id" => fm.id = val.to_string(),
                    "title" => fm.title = val.to_string(),
                    "version" => {
                        if let Ok(v) = val.parse::<u32>() {
                            fm.version = v;
                        }
                    }
                    "component" => fm.component = Some(val.to_string()),
                    "replaces" => fm.replaces = Some(val.to_string()),
                    "status" => fm.status = val.to_string(),
                    _ => {}
                }
            }
        }
    } else {
        body_lines = lines;
    }

    let mut reqs = Vec::new();
    let mut i = 0;
    while i < body_lines.len() {
        let trimmed = body_lines[i].trim();
        if trimmed.starts_with("- **") || trimmed.starts_with("* **") {
            if let Some(end_bold) = trimmed[4..].find("**") {
                let id_chunk = &trimmed[4..4 + end_bold];
                let rest = trimmed[4 + end_bold + 2..].trim();

                // Extract optional (v2) inside id or after bold
                let mut req_version = fm.version;
                if let (Some(vs), Some(ve)) = (rest.find("(v"), rest.find(')')) {
                    if vs < ve {
                        if let Ok(v) = rest[vs + 2..ve].parse::<u32>() {
                            req_version = v;
                        }
                    }
                }

                let initial_statement = if let Some(stripped) = rest.strip_prefix(':') {
                    stripped.trim()
                } else if let Some(colon_pos) = rest.find(':') {
                    rest[colon_pos + 1..].trim()
                } else {
                    rest
                };

                let mut full_statement = initial_statement.to_string();
                let mut j = i + 1;
                while j < body_lines.len() {
                    let next_trimmed = body_lines[j].trim();
                    if next_trimmed.starts_with("- **")
                        || next_trimmed.starts_with("* **")
                        || next_trimmed.starts_with('#')
                    {
                        break;
                    }
                    if !next_trimmed.is_empty() {
                        if !full_statement.is_empty() {
                            full_statement.push(' ');
                        }
                        full_statement.push_str(next_trimmed);
                    }
                    j += 1;
                }

                let clean_id = id_chunk.split_whitespace().next().unwrap_or(id_chunk);
                let comp = fm.component.clone().unwrap_or_else(|| {
                    if clean_id.starts_with("SHIP") {
                        "shipping-calculator".to_string()
                    } else if clean_id.starts_with("QUOTA") {
                        "quota-engine".to_string()
                    } else if clean_id.starts_with("RES") {
                        "reservation-coordinator".to_string()
                    } else {
                        "service".to_string()
                    }
                });

                reqs.push(RequirementSpec {
                    id: clean_id.to_string(),
                    component: comp,
                    title: format!("Requirement {clean_id}"),
                    statement: full_statement.clone(),
                    status: fm.status.clone(),
                    version: req_version,
                    source: RequirementSource {
                        source_type: "work-item".to_string(),
                        ref_target: format!("{filename}#{}", clean_id.to_lowercase()),
                    },
                    acceptance_criteria: vec![full_statement],
                });
                i = j;
                continue;
            }
        }
        i += 1;
    }

    (fm, reqs)
}

/// Reconciles a collection of requirement deltas or Work Items against a base `SystemSpec`.
///
/// # Strict Monotonic Versioning & Conflicts
///
/// - If a requirement exists and the proposed update has $version \le existing.version$ with a
///   different statement, returns [`SpecError::ReconciliationConflict`].
/// - If a Work Item declares `replaces: TARGET` but `TARGET` does not exist in the specification,
///   returns [`SpecError::BrokenReference`].
/// - If reconciliation succeeds, non-modified requirements and components remain untouched,
///   and newly added requirements are associated with their component operations.
pub fn reconcile_spec(
    base: &SystemSpec,
    updates: &[(&str, &str)],
) -> Result<SystemSpec, SpecError> {
    let mut spec = base.clone();

    // Track updated sources in metadata
    for &(filename, _) in updates {
        if !spec.metadata.source_files.iter().any(|f| f == filename) {
            spec.metadata.source_files.push(filename.to_string());
        }
    }

    for &(filename, content) in updates {
        let (fm, reqs) = parse_work_item_delta(filename, content);

        for req in reqs {
            let target_id = fm.replaces.clone().unwrap_or_else(|| req.id.clone());

            if let Some(existing) = spec.requirements.iter_mut().find(|r| r.id == target_id) {
                // Reject invalid backwards or duplicate version updates with different statements
                if req.version <= existing.version && req.statement != existing.statement {
                    return Err(SpecError::ReconciliationConflict(
                        filename.to_string(),
                        existing.id.clone(),
                        existing.version,
                        req.version,
                    ));
                }

                // If target_id was replaced by a new id, update the requirement ID
                if target_id != req.id {
                    existing.id.clone_from(&req.id);
                }

                // Update existing requirement preserving version tracking
                existing.version = req.version;
                existing.statement.clone_from(&req.statement);
                existing.status.clone_from(&req.status);
                existing.source.clone_from(&req.source);
                existing
                    .acceptance_criteria
                    .clone_from(&req.acceptance_criteria);
            } else {
                // If it claims to replace a non-existent requirement, fail explicit
                if let Some(ref repl) = fm.replaces {
                    return Err(SpecError::BrokenReference(
                        filename.to_string(),
                        repl.clone(),
                    ));
                }

                // Add new requirement and link it to component operations
                for op in &mut spec.operations {
                    if op.component == req.component && !op.rules_applied.contains(&req.id) {
                        op.rules_applied.push(req.id.clone());
                    }
                }
                spec.requirements.push(req);
            }
        }
    }

    Ok(spec)
}
