//! Dynamic Markdown Requirements Interpreter.
//!
//! Parses natural language specification narratives written in structured Markdown
//! into canonical `SystemSpec` instances completely offline, without LLM dependencies.

use crate::spec::error::SpecError;
use crate::spec::model::{
    ComponentSpec, EntityField, EntitySpec, OperationParam, OperationSpec, RequirementSource,
    RequirementSpec, SpecMetadata, SystemSpec,
};

/// Contract for interpreting natural language narratives into structured System Specifications.
pub trait RequirementsInterpreter {
    fn interpret_narrative(&self, content: &str, file_name: &str) -> Result<SystemSpec, SpecError>;
}

/// Dynamic Markdown Requirements Interpreter that parses sections, components,
/// entities, rules and WIT contracts agnostically and deterministically.
#[derive(Debug, Default, Clone, Copy)]
pub struct MarkdownRequirementsInterpreter;

impl RequirementsInterpreter for MarkdownRequirementsInterpreter {
    #[allow(clippy::too_many_lines)]
    fn interpret_narrative(&self, content: &str, file_name: &str) -> Result<SystemSpec, SpecError> {
        let lines: Vec<&str> = content.lines().collect();

        // 1. Extract Document Title
        let title = lines.iter().find(|l| l.starts_with("# ")).map_or_else(
            || "System Specification".to_string(),
            |l| l.trim_start_matches("# ").trim().to_string(),
        );

        let mut components: Vec<ComponentSpec> = Vec::new();
        let mut requirements: Vec<RequirementSpec> = Vec::new();
        let mut entities: Vec<EntitySpec> = Vec::new();
        let mut operations: Vec<OperationSpec> = Vec::new();

        // Active parsing context
        let mut current_section = "";
        let mut current_component_id: Option<String> = None;
        let mut current_entity_name: Option<String> = None;
        let mut current_entity_fields: Vec<EntityField> = Vec::new();
        let mut current_entity_desc = String::new();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();

            if line.starts_with("## ") || line.starts_with("### ") {
                let is_h2 = line.starts_with("## ");
                let header_text = if is_h2 {
                    line.trim_start_matches("## ").trim()
                } else {
                    line.trim_start_matches("### ").trim()
                };
                let sec = header_text.to_lowercase();

                // If leaving entities section on an H2 heading, flush pending entity
                if is_h2 && current_section == "entities" {
                    if let Some(ename) = current_entity_name.take() {
                        entities.push(EntitySpec {
                            name: ename,
                            description: current_entity_desc.clone(),
                            fields: current_entity_fields.clone(),
                        });
                        current_entity_fields.clear();
                        current_entity_desc.clear();
                    }
                }

                if is_h2 {
                    if sec.contains("visión") || sec.contains("overview") {
                        current_section = "overview";
                    } else if sec.contains("entidades") || sec.contains("entities") {
                        current_section = "entities";
                    } else if sec.contains("componente") || sec.contains("component") {
                        current_section = "component";
                    } else if sec.contains("requisitos") || sec.contains("requirements") {
                        current_section = "requirements";
                    } else if sec.contains("contratos")
                        || sec.contains("wit")
                        || sec.contains("interfaces")
                    {
                        current_section = "wit";
                    } else if sec.contains("criterios") || sec.contains("acceptance") {
                        current_section = "acceptance";
                    } else {
                        current_section = "other";
                    }
                } else {
                    // H3 level heading handling
                    if sec.contains("requisitos de ") || sec.contains("requirements for ") {
                        current_section = "requirements";
                        let cid = header_text
                            .split(':')
                            .last()
                            .unwrap_or(header_text)
                            .split_whitespace()
                            .last()
                            .unwrap_or("")
                            .replace('`', "")
                            .trim()
                            .to_string();
                        if !cid.is_empty() {
                            current_component_id = Some(cid);
                        }
                    } else if current_section == "wit" {
                        if let (Some(s), Some(e)) = (header_text.find('`'), header_text.rfind('`'))
                        {
                            if s < e {
                                let candidate = header_text[s + 1..e].trim();
                                if components.iter().any(|c| c.id == candidate) {
                                    current_component_id = Some(candidate.to_string());
                                }
                            }
                        }
                    } else if current_section == "entities" {
                        // Flush previous entity if any
                        if let Some(ename) = current_entity_name.take() {
                            entities.push(EntitySpec {
                                name: ename,
                                description: current_entity_desc.clone(),
                                fields: current_entity_fields.clone(),
                            });
                            current_entity_fields.clear();
                            current_entity_desc.clear();
                        }

                        let ename = if let (Some(s), Some(e)) = (line.find('`'), line.rfind('`')) {
                            line[s + 1..e].trim().to_string()
                        } else {
                            header_text.to_string()
                        };
                        current_entity_name = Some(ename);
                    }
                }

                if sec.contains("componente") || sec.contains("component") {
                    // Extract component ID from backticks or from after colon
                    let cid_opt = if let (Some(start), Some(end)) =
                        (header_text.find('`'), header_text.rfind('`'))
                    {
                        if start < end {
                            Some(header_text[start + 1..end].trim().to_string())
                        } else {
                            None
                        }
                    } else if let Some(colon_pos) = header_text.rfind(':') {
                        let candidate = header_text[colon_pos + 1..].trim();
                        if !candidate.is_empty() && !candidate.contains(' ') {
                            Some(candidate.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if let Some(cid) = cid_opt {
                        current_component_id = Some(cid.clone());

                        let comp_name = header_text.split(':').nth(1).map_or_else(
                            || cid.clone(),
                            |s| s.split('(').next().unwrap_or(s).trim().to_string(),
                        );

                        if !components.iter().any(|c| c.id == cid) {
                            components.push(ComponentSpec {
                                id: cid.clone(),
                                name: comp_name,
                                description: format!("Componente {cid}"),
                                wit_file: format!("wit/{cid}.wit"),
                                world: cid.clone(),
                                export_interfaces: Vec::new(),
                                import_interfaces: Vec::new(),
                                capabilities: Vec::new(),
                                dependencies: Vec::new(),
                            });
                        }
                    }
                }
                i += 1;
                continue;
            }

            // In overview or component section, also discover components if listed e.g. "1. **Nombre (`id`):** Descripción"
            if (current_section == "overview" || current_section == "other")
                && (line.contains("(`") && line.contains("`):"))
            {
                if let (Some(start), Some(end)) = (line.find("(`"), line.find("`):")) {
                    let cid = line[start + 2..end].trim().to_string();
                    let name = line.split("**").nth(1).map_or_else(
                        || cid.clone(),
                        |s| s.split('(').next().unwrap_or(s).trim().to_string(),
                    );
                    let desc = line.split("`):").nth(1).unwrap_or("").trim().to_string();

                    if !components.iter().any(|c| c.id == cid) {
                        components.push(ComponentSpec {
                            id: cid.clone(),
                            name,
                            description: desc,
                            wit_file: format!("wit/{cid}.wit"),
                            world: cid.clone(),
                            export_interfaces: Vec::new(),
                            import_interfaces: Vec::new(),
                            capabilities: Vec::new(),
                            dependencies: Vec::new(),
                        });
                    }
                }
            }

            // In entities section, extract record definitions
            if current_section == "entities" && (line.starts_with("- `") || line.starts_with("* `"))
            {
                if let Some(end_tick) = line[3..].find('`') {
                    let fname = line[3..3 + end_tick].trim().to_string();
                    let fdesc = line[3 + end_tick + 1..]
                        .trim_start_matches(':')
                        .trim()
                        .to_string();
                    let ftype = if fname.ends_with("_code")
                        || fname.ends_with("_cents")
                        || fname.ends_with("_count")
                        || fname.ends_with("_grams")
                        || fname.ends_with("_mb")
                        || fname.ends_with("_id")
                        || fname.ends_with("_tier")
                        || fname.ends_with("_km")
                        || fname.ends_with("_quality")
                    {
                        "i32".to_string()
                    } else if fname.starts_with("is_") || fname.starts_with("has_") {
                        "bool".to_string()
                    } else {
                        "i32".to_string()
                    };
                    current_entity_fields.push(EntityField {
                        name: fname,
                        field_type: ftype,
                        description: fdesc,
                    });
                }
            }

            // Extract requirements (e.g. - **`SHIP-001` (Título):** Statement OR - **SHIP-001**: Statement)
            if line.starts_with("- **") || line.starts_with("* **") {
                if let Some(end_bold) = line[4..].find("**") {
                    let raw_id_chunk = &line[4..4 + end_bold];
                    let clean_chunk = raw_id_chunk.replace('`', "");
                    let req_id = clean_chunk
                        .split([' ', '(', ':'])
                        .next()
                        .unwrap_or(&clean_chunk)
                        .trim()
                        .to_string();

                    // Check if it's a valid requirement identifier format e.g. SHIP-001, TEL-001
                    if req_id.contains('-')
                        && req_id
                            .chars()
                            .next()
                            .is_some_and(|c| c.is_ascii_uppercase())
                    {
                        let rest = line[4 + end_bold + 2..].trim();
                        let initial_statement = if let Some(stripped) = rest.strip_prefix(':') {
                            stripped.trim()
                        } else {
                            rest
                        };

                        let mut full_statement = initial_statement.to_string();
                        let mut sub_lines = Vec::new();
                        let mut j = i + 1;
                        while j < lines.len() {
                            let next_line = lines[j].trim();
                            if next_line.starts_with("- **")
                                || next_line.starts_with("* **")
                                || next_line.starts_with('#')
                                || next_line.starts_with("---")
                            {
                                break;
                            }
                            if !next_line.is_empty() {
                                sub_lines.push(next_line);
                            }
                            j += 1;
                        }
                        if !sub_lines.is_empty() {
                            if !full_statement.is_empty() {
                                full_statement.push(' ');
                            }
                            full_statement.push_str(&sub_lines.join(" "));
                        }

                        // Determine component association
                        let comp_id = if let Some(ref cid) = current_component_id {
                            cid.clone()
                        } else if !components.is_empty() {
                            components[0].id.clone()
                        } else {
                            "default-component".to_string()
                        };

                        let title = raw_id_chunk.replace('`', "").trim().to_string();

                        requirements.push(RequirementSpec {
                            id: req_id.clone(),
                            component: comp_id,
                            title,
                            statement: full_statement.clone(),
                            status: "active".to_string(),
                            version: 1,
                            source: RequirementSource {
                                source_type: "narrative".to_string(),
                                ref_target: format!("{file_name}#{}", req_id.to_lowercase()),
                            },
                            acceptance_criteria: vec![full_statement],
                        });

                        i = j;
                        continue;
                    }
                }
            }

            // Extract WIT Interface blocks
            if line.starts_with("interface ") && line.ends_with('{') {
                let iface_name = line
                    .trim_start_matches("interface ")
                    .trim_end_matches('{')
                    .trim()
                    .to_string();
                let mut wit_body = String::new();
                let mut j = i + 1;
                while j < lines.len() {
                    let wline = lines[j].trim();
                    if wline == "}" {
                        break;
                    }
                    if !wline.starts_with("//") {
                        wit_body.push_str(wline);
                        wit_body.push(' ');
                    }
                    j += 1;
                }

                if iface_name == "host-services" {
                    if let Some(ref cid) = current_component_id {
                        if let Some(comp) = components.iter_mut().find(|c| &c.id == cid) {
                            if !comp.import_interfaces.contains(&iface_name) {
                                comp.import_interfaces.push(iface_name.clone());
                            }
                        }
                    }
                } else if iface_name != "types" {
                    // Parse operations from semicolon-separated statements in interface body
                    for stmt in wit_body.split(';') {
                        let wstmt = stmt.trim();
                        if let Some(colon_pos) = wstmt.find(": func(") {
                            let op_name = wstmt[..colon_pos].trim().replace('-', "_");
                            let after_func = &wstmt[colon_pos + 7..];
                            let mut params = Vec::new();
                            let mut returns = "i32".to_string();

                            if let Some(r_paren) = after_func.find(')') {
                                let params_chunk = &after_func[..r_paren];
                                for p in params_chunk.split(',') {
                                    let p_trim = p.trim();
                                    if let Some((pname, ptype)) = p_trim.split_once(':') {
                                        let clean_name = pname.trim().replace('-', "_");
                                        let clean_type = ptype.trim().replace("s32", "i32");
                                        if !clean_name.is_empty() {
                                            params.push(OperationParam {
                                                name: clean_name,
                                                param_type: clean_type,
                                            });
                                        }
                                    }
                                }
                                let ret_chunk = after_func[r_paren + 1..].trim();
                                if let Some(arrow) = ret_chunk.find("->") {
                                    let raw_ret = ret_chunk[arrow + 2..]
                                        .trim()
                                        .trim_end_matches(';')
                                        .trim()
                                        .replace("s32", "i32");
                                    let norm_cmp = raw_ret.replace('-', "").to_lowercase();
                                    if let Some(ent) = entities.iter().find(|e| {
                                        e.name.replace('_', "").to_lowercase() == norm_cmp
                                    }) {
                                        returns.clone_from(&ent.name);
                                    } else {
                                        returns = raw_ret;
                                    }
                                }
                            }

                            // Associate with matching component
                            let target_comp = if let Some(ref cid) = current_component_id {
                                cid.clone()
                            } else if !components.is_empty() {
                                components[0].id.clone()
                            } else {
                                "main-component".to_string()
                            };

                            operations.push(OperationSpec {
                                name: op_name,
                                component: target_comp,
                                wit_interface: iface_name.clone(),
                                params,
                                returns,
                                rules_applied: Vec::new(),
                                effects: Vec::new(),
                            });
                        }
                    }
                }
                i = j;
            }

            i += 1;
        }

        // Flush last entity if any
        if let Some(ename) = current_entity_name.take() {
            entities.push(EntitySpec {
                name: ename,
                description: current_entity_desc,
                fields: current_entity_fields,
            });
        }

        // Normalize operation return types against flushed entities
        for op in &mut operations {
            let norm_cmp = op.returns.replace('-', "").to_lowercase();
            if let Some(ent) = entities
                .iter()
                .find(|e| e.name.replace('_', "").to_lowercase() == norm_cmp)
            {
                op.returns.clone_from(&ent.name);
            }
        }

        // Link dependencies and operations rules_applied
        for op in &mut operations {
            for req in &requirements {
                if req.component == op.component {
                    op.rules_applied.push(req.id.clone());
                }
            }
        }

        // Update component export_interfaces based on operations and WIT
        for comp in &mut components {
            for op in &operations {
                if op.component == comp.id && !comp.export_interfaces.contains(&op.wit_interface) {
                    comp.export_interfaces.push(op.wit_interface.clone());
                }
            }
        }

        // Detect inter-component dependencies and import_interfaces
        let comp_snapshot: Vec<(String, Vec<String>)> = components
            .iter()
            .map(|c| (c.id.clone(), c.export_interfaces.clone()))
            .collect();
        for comp in &mut components {
            for (other_id, other_exports) in &comp_snapshot {
                if other_id != &comp.id {
                    let has_dep = requirements
                        .iter()
                        .any(|r| r.component == comp.id && r.statement.contains(other_id));
                    if has_dep {
                        if !comp.dependencies.contains(other_id) {
                            comp.dependencies.push(other_id.clone());
                        }
                        for exp_iface in other_exports {
                            if !comp.import_interfaces.contains(exp_iface) {
                                comp.import_interfaces.push(exp_iface.clone());
                            }
                        }
                    }
                }
            }
            let uses_host = requirements.iter().any(|r| {
                r.component == comp.id
                    && (r.statement.contains("kv_get")
                        || r.statement.contains("emit_audit")
                        || r.statement.contains("check_credit")
                        || r.statement.contains("tenant_status"))
            });
            if uses_host
                && !comp
                    .import_interfaces
                    .contains(&"host-services".to_string())
            {
                comp.import_interfaces
                    .insert(0, "host-services".to_string());
            }
        }

        // Validation: Fail-explicit when document lacks recognizable components or requirements
        if components.is_empty() {
            return Err(SpecError::MissingComponents);
        }

        if requirements.is_empty() {
            return Err(SpecError::MissingRequirements);
        }

        Ok(SystemSpec {
            schema: Some("./system-spec.schema.json".to_string()),
            spec_version: "1.0.0".to_string(),
            metadata: SpecMetadata {
                title,
                description: format!(
                    "System specification synthesized by Agent-Native RequirementsInterpreter from {file_name}"
                ),
                source_type: "narrative".to_string(),
                source_files: vec![file_name.to_string()],
                last_updated: "2026-10-19".to_string(),
            },
            components,
            requirements,
            entities,
            operations,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interprets_unseen_custom_narrative() {
        let md = r"# Sistema de Telemetría Satelital

## 1. Visión general del producto
Plataforma para adquisición y retransmisión de paquetes orbitales.
1. **Colector de Paquetes (`packet-collector`):** Ingesta tramas telemétricas.
2. **Distribuidor de Datos (`data-distributor`):** Valida órbitas y retransmite.

## 2. Entidades de Dominio
### Muestra de Sensor (`TelemetrySample`)
- `status_code`: Estado del muestreo.
- `altitude_km`: Altitud orbital en kilómetros.
- `signal_quality`: Calidad de enlace del 0 al 100.

## 3. Componente A: Colector (`packet-collector`)
### Reglas
- **`TEL-001` (Integridad de paquete):** Si signal_quality < 10, descarta el paquete retornando -1.
- **`TEL-002` (Filtro atmosférico):** Paquetes por debajo de 160 km son marcados como reentrada.

## 4. Componente B: Distribuidor (`data-distributor`)
### Reglas
- **`DIST-001` (Retransmisión a estación terrestre):** Envía telemetría a centro de control si status_code == 0.

## 5. Contratos de Interfaz (WIT)
### Interfaz Exportada por `packet-collector`
```wit
interface collector-service {
    process-sample: func(altitude-km: s32, signal-quality: s32) -> s32;
}
```
";

        let interpreter = MarkdownRequirementsInterpreter;
        let spec = interpreter
            .interpret_narrative(md, "telemetry.md")
            .expect("valid interpretation");

        assert_eq!(spec.metadata.title, "Sistema de Telemetría Satelital");
        assert_eq!(spec.components.len(), 2);
        assert_eq!(spec.components[0].id, "packet-collector");
        assert_eq!(spec.components[1].id, "data-distributor");
        assert_eq!(spec.requirements.len(), 3);
        assert!(spec.requirements.iter().any(|r| r.id == "TEL-001"));
        assert!(spec.requirements.iter().any(|r| r.id == "TEL-002"));
        assert!(spec.requirements.iter().any(|r| r.id == "DIST-001"));
        assert_eq!(spec.entities.len(), 1);
        assert_eq!(spec.entities[0].name, "TelemetrySample");
        assert_eq!(spec.entities[0].fields.len(), 3);
        assert_eq!(spec.operations.len(), 1);
        assert_eq!(spec.operations[0].name, "process_sample");
    }

    #[test]
    fn test_rejects_incompatible_narrative_without_inventing_components() {
        let md = r"# Poema sobre el software
Un día el código compiló y todos fueron felices.
Fin.
";
        let interpreter = MarkdownRequirementsInterpreter;
        let res = interpreter.interpret_narrative(md, "poem.md");
        assert_eq!(res, Err(SpecError::MissingComponents));
    }

    #[test]
    fn test_rejects_narrative_without_requirements() {
        let md = r"# Sistema Sin Requisitos
## Componente Principal (`main-service`)
Este componente no tiene ninguna regla.
";
        let interpreter = MarkdownRequirementsInterpreter;
        let res = interpreter.interpret_narrative(md, "no_reqs.md");
        assert_eq!(res, Err(SpecError::MissingRequirements));
    }
}
