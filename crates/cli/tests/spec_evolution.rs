//! Integration tests for incremental specification evolution, reconciliation, and diffing.
//!
//! Validates AC-01 through AC-04 of AN-025 completely offline.

use wasma_cli::spec::RequirementsInterpreter;
use wasma_cli::spec::diff::compute_spec_diff;
use wasma_cli::spec::error::SpecError;
use wasma_cli::spec::interpreter::MarkdownRequirementsInterpreter;
use wasma_cli::spec::model::{
    ComponentSpec, EntityField, EntitySpec, OperationParam, OperationSpec, RequirementSource,
    RequirementSpec, SpecMetadata, SystemSpec,
};
use wasma_cli::spec::reconciler::reconcile_spec;

#[allow(clippy::too_many_lines)]
fn create_base_spec() -> SystemSpec {
    SystemSpec {
        schema: None,
        spec_version: "1.0.0".to_string(),
        metadata: SpecMetadata {
            title: "Resource Reservation System".to_string(),
            description: "Quota and reservation coordination".to_string(),
            source_type: "narrative".to_string(),
            source_files: vec!["narrative.md".to_string()],
            last_updated: "2026-10-19".to_string(),
        },
        components: vec![
            ComponentSpec {
                id: "quota-engine".to_string(),
                name: "Quota Engine".to_string(),
                description: "Evaluates resource limits".to_string(),
                wit_file: "wit/quota.wit".to_string(),
                world: "quota-engine".to_string(),
                export_interfaces: vec!["quota-service".to_string()],
                import_interfaces: vec![],
                capabilities: vec![],
                dependencies: vec![],
            },
            ComponentSpec {
                id: "reservation-coordinator".to_string(),
                name: "Reservation Coordinator".to_string(),
                description: "Coordinates tenant reservations".to_string(),
                wit_file: "wit/reservation.wit".to_string(),
                world: "reservation-coordinator".to_string(),
                export_interfaces: vec!["reservation-service".to_string()],
                import_interfaces: vec!["quota-service".to_string()],
                capabilities: vec!["host_services".to_string()],
                dependencies: vec!["quota-engine".to_string()],
            },
        ],
        requirements: vec![
            RequirementSpec {
                id: "QUOTA-001".to_string(),
                component: "quota-engine".to_string(),
                title: "Argument validation".to_string(),
                statement: "If requested <= 0 return 0".to_string(),
                status: "active".to_string(),
                version: 1,
                source: RequirementSource {
                    source_type: "narrative".to_string(),
                    ref_target: "narrative.md#quota-001".to_string(),
                },
                acceptance_criteria: vec!["If requested <= 0 return 0".to_string()],
            },
            RequirementSpec {
                id: "QUOTA-002".to_string(),
                component: "quota-engine".to_string(),
                title: "Quota limit".to_string(),
                statement: "If current + requested > 1000 return 0, else 1".to_string(),
                status: "active".to_string(),
                version: 1,
                source: RequirementSource {
                    source_type: "narrative".to_string(),
                    ref_target: "narrative.md#quota-002".to_string(),
                },
                acceptance_criteria: vec![
                    "If current + requested > 1000 return 0, else 1".to_string(),
                ],
            },
            RequirementSpec {
                id: "RES-001".to_string(),
                component: "reservation-coordinator".to_string(),
                title: "Tenant validation".to_string(),
                statement: "If tenant_id <= 0 return status 1".to_string(),
                status: "active".to_string(),
                version: 1,
                source: RequirementSource {
                    source_type: "narrative".to_string(),
                    ref_target: "narrative.md#res-001".to_string(),
                },
                acceptance_criteria: vec!["If tenant_id <= 0 return status 1".to_string()],
            },
        ],
        entities: vec![EntitySpec {
            name: "ReservationResult".to_string(),
            description: "Reservation outcome".to_string(),
            fields: vec![
                EntityField {
                    name: "status_code".to_string(),
                    field_type: "i32".to_string(),
                    description: "Result code".to_string(),
                },
                EntityField {
                    name: "approved_units".to_string(),
                    field_type: "i32".to_string(),
                    description: "Units approved".to_string(),
                },
            ],
        }],
        operations: vec![
            OperationSpec {
                name: "check_quota".to_string(),
                component: "quota-engine".to_string(),
                wit_interface: "quota-service".to_string(),
                params: vec![
                    OperationParam {
                        name: "requested".to_string(),
                        param_type: "i32".to_string(),
                    },
                    OperationParam {
                        name: "current".to_string(),
                        param_type: "i32".to_string(),
                    },
                ],
                returns: "i32".to_string(),
                rules_applied: vec!["QUOTA-001".to_string(), "QUOTA-002".to_string()],
                effects: vec![],
            },
            OperationSpec {
                name: "reserve".to_string(),
                component: "reservation-coordinator".to_string(),
                wit_interface: "reservation-service".to_string(),
                params: vec![
                    OperationParam {
                        name: "tenant_id".to_string(),
                        param_type: "i32".to_string(),
                    },
                    OperationParam {
                        name: "units".to_string(),
                        param_type: "i32".to_string(),
                    },
                ],
                returns: "ReservationResult".to_string(),
                rules_applied: vec!["RES-001".to_string()],
                effects: vec![],
            },
        ],
    }
}

#[test]
fn test_reconciliation_evolution_v1_to_v2() {
    let base = create_base_spec();

    let wi1 = r"---
id: WI-RES-01
title: Expansión de cuota máxima por tenant
status: active
version: 2
component: quota-engine
replaces: QUOTA-002
---

# WI-RES-01 — Expansión de cuota máxima por tenant

## Requisitos Modificados

- **QUOTA-002** (v2): Si current + requested > 2500 retorna 0, en caso contrario retorna 1.
";

    let evolved =
        reconcile_spec(&base, &[("WI-RES-01.md", wi1)]).expect("reconciliation should succeed");

    assert_eq!(evolved.requirements.len(), 3);
    let q2 = evolved
        .find_requirement("QUOTA-002")
        .expect("QUOTA-002 exists");
    assert_eq!(q2.version, 2);
    assert!(q2.statement.contains("2500"));

    let diff = compute_spec_diff(&base, &evolved);
    assert_eq!(diff.modified_requirements, vec!["QUOTA-002".to_string()]);
    assert_eq!(diff.affected_components, vec!["quota-engine".to_string()]);
    assert!(diff.added_requirements.is_empty());
    assert!(diff.deprecated_requirements.is_empty());
}

#[test]
fn test_reconciliation_additive_requirement() {
    let base = create_base_spec();

    let wi2 = r"---
id: WI-RES-02
title: Incorporación de penalización
status: active
version: 1
component: reservation-coordinator
---

# WI-RES-02 — Incorporación de penalización

## Nuevos Requisitos

- **RES-002**: Si tenant tiene atrasos asigna penalización de 50 puntos.
";

    let evolved =
        reconcile_spec(&base, &[("WI-RES-02.md", wi2)]).expect("reconciliation should succeed");

    assert_eq!(evolved.requirements.len(), 4);
    assert!(evolved.find_requirement("RES-002").is_some());

    // New requirement must be auto-linked to the component's operations
    let op_reserve = evolved
        .operations
        .iter()
        .find(|o| o.name == "reserve")
        .expect("reserve operation");
    assert!(op_reserve.rules_applied.contains(&"RES-002".to_string()));

    let diff = compute_spec_diff(&base, &evolved);
    assert_eq!(diff.added_requirements, vec!["RES-002".to_string()]);
    assert_eq!(
        diff.affected_components,
        vec!["reservation-coordinator".to_string()]
    );
}

#[test]
fn test_reconciliation_rejects_version_downgrade_or_collision() {
    let base = create_base_spec();

    // Base QUOTA-002 is version 1. Proposing a delta with version 1 but different statement must fail.
    let wi_conflict = r"---
id: WI-RES-CONFLICT
title: Conflicto de versión
status: active
version: 1
component: quota-engine
replaces: QUOTA-002
---

- **QUOTA-002** (v1): Modificación conflictiva sin incrementar versión.
";

    let err = reconcile_spec(&base, &[("conflict.md", wi_conflict)]).unwrap_err();
    assert_eq!(
        err,
        SpecError::ReconciliationConflict("conflict.md".to_string(), "QUOTA-002".to_string(), 1, 1)
    );
}

#[test]
fn test_reconciliation_rejects_broken_reference() {
    let base = create_base_spec();

    let wi_broken = r"---
id: WI-BROKEN
title: Referencia rota
status: active
version: 2
component: quota-engine
replaces: NON_EXISTENT_REQ_999
---

- **NEW-REQ**: Statement
";

    let err = reconcile_spec(&base, &[("broken.md", wi_broken)]).unwrap_err();
    assert_eq!(
        err,
        SpecError::BrokenReference("broken.md".to_string(), "NON_EXISTENT_REQ_999".to_string())
    );
}

#[test]
fn test_diff_detects_contract_and_entity_modifications() {
    let base = create_base_spec();
    let mut modified = base.clone();

    // Modify a contract interface
    modified.components[0]
        .export_interfaces
        .push("v2-ops".to_string());

    // Modify an entity
    modified.entities[0].fields.push(EntityField {
        name: "penalty_points".to_string(),
        field_type: "i32".to_string(),
        description: "Penalty points".to_string(),
    });

    let diff = compute_spec_diff(&base, &modified);
    assert_eq!(diff.modified_contracts, vec!["quota-engine".to_string()]);
    assert_eq!(
        diff.modified_entities,
        vec!["ReservationResult".to_string()]
    );
    assert_eq!(diff.affected_components, vec!["quota-engine".to_string()]);
    assert!(!diff.is_empty());
}

#[test]
fn test_multi_step_evolution_chain_v1_to_v2_to_v3() {
    let narrative = r"# Sistema de Telemetría Orbital

## 1. Visión general del producto
Plataforma para adquisición y retransmisión de paquetes orbitales.
1. **Colector de Paquetes (`packet-collector`):** Ingesta tramas telemétricas.

## 2. Entidades de Dominio
### Muestra de Sensor (`TelemetrySample`)
- `status_code`: Estado del muestreo.
- `altitude_km`: Altitud orbital en kilómetros.

## 3. Componente A: Colector (`packet-collector`)
### Reglas
- **`TEL-001` (Integridad de paquete):** Si signal_quality < 10, descarta el paquete retornando -1.

## 4. Contratos de Interfaz (WIT)
### Interfaz Exportada por `packet-collector`
```wit
interface ops {
    process-sample: func(altitude-km: s32) -> s32;
}
```
";

    let interpreter = MarkdownRequirementsInterpreter;
    let spec_v1 = interpreter
        .interpret_narrative(narrative, "telemetry.md")
        .unwrap();
    assert_eq!(spec_v1.requirements.len(), 1);

    // Step 1: Evolve to v2 (add new requirement)
    let wi_step1 = r"---
id: WI-TEL-02
title: Filtro atmosférico
status: active
version: 1
component: packet-collector
---

- **TEL-002**: Paquetes por debajo de 160 km son marcados como reentrada.
";
    let spec_v2 = reconcile_spec(&spec_v1, &[("WI-TEL-02.md", wi_step1)]).unwrap();
    assert_eq!(spec_v2.requirements.len(), 2);
    let diff_1_2 = compute_spec_diff(&spec_v1, &spec_v2);
    assert_eq!(diff_1_2.added_requirements, vec!["TEL-002".to_string()]);

    // Step 2: Evolve to v3 (update TEL-001 version to 2)
    let wi_step2 = r"---
id: WI-TEL-03
title: Umbral de integridad reforzado
status: active
version: 2
component: packet-collector
replaces: TEL-001
---

- **TEL-001** (v2): Si signal_quality < 20, descarta el paquete retornando -2.
";
    let spec_v3 = reconcile_spec(&spec_v2, &[("WI-TEL-03.md", wi_step2)]).unwrap();
    assert_eq!(spec_v3.requirements.len(), 2);
    let tel1 = spec_v3.find_requirement("TEL-001").unwrap();
    assert_eq!(tel1.version, 2);
    assert!(tel1.statement.contains("signal_quality < 20"));

    let diff_2_3 = compute_spec_diff(&spec_v2, &spec_v3);
    assert_eq!(diff_2_3.modified_requirements, vec!["TEL-001".to_string()]);
    assert_eq!(
        diff_2_3.affected_components,
        vec!["packet-collector".to_string()]
    );
}
