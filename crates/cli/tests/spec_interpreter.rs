//! Integration tests for the `SystemSpec` Markdown Requirements Interpreter.
//!
//! Validates AC-01 through AC-05 of AN-023 completely offline.

use agent_native::spec::{
    MarkdownRequirementsInterpreter, RequirementsInterpreter, SpecError, SystemSpec,
};

const SAMPLE_MULTI_COMPONENT_NARRATIVE: &str = r"# Sistema de Gestión de Envíos Logísticos

## 1. Visión general del producto
Plataforma descentralizada para cálculo de tarifas y auditoría aduanera.
1. **Calculador de Tarifas (`shipping-calculator`):** Calcula costos de envío y recargos por peso.
2. **Auditor Aduanero (`customs-auditor`):** Aplica regulaciones arancelarias e impuestos transfronterizos.

## 2. Entidades de Dominio
### Cotización de Envío (`ShippingQuote`)
- `subtotal_cents`: Monto neto base del flete.
- `tariff_cents`: Recargo aduanero aplicado.
- `total_cents`: Monto total consolidado.
- `is_express`: Indicador de despacho prioritario.

### Declaración Aduanera (`CustomsDeclaration`)
- `hs_code`: Código arancelario armonizado.
- `duty_cents`: Arancel liquidado.

## 3. Componente A: Calculador (`shipping-calculator`)
### Reglas
- **`SHIP-001` (Tarifa base):** Todo flete con peso menor a 1000 gramos tiene un cargo fijo de 500 centavos.
- **`SHIP-002` (Dependencia aduanera):** Invoca a `customs-auditor` para calcular los aranceles antes de emitir la cotización final.
- **`SHIP-003` (Auditoría de estado):** Registra el resultado mediante `emit_audit` en el registro del host.

## 4. Componente B: Auditor (`customs-auditor`)
### Reglas
- **`CUST-001` (Exención fiscal):** Si el código arancelario es 9999, la tasa es cero.
- **`CUST-002` (Consulta de estado):** Consulta `kv_get` para verificar listas negras de exportación.

## 5. Contratos de Interfaz (WIT)
### Interfaz Exportada por `shipping-calculator`
```wit
interface quote-service {
    calculate-quote: func(weight-grams: s32, is-express: bool) -> shipping-quote;
}
```

### Interfaz Exportada por `customs-auditor`
```wit
interface customs-service {
    assess-duty: func(hs-code: s32, declared-value-cents: s32) -> s32;
}
```
";

#[test]
fn test_interpret_multi_component_system_spec() {
    let interpreter = MarkdownRequirementsInterpreter;
    let spec = interpreter
        .interpret_narrative(SAMPLE_MULTI_COMPONENT_NARRATIVE, "logistics.md")
        .expect("must parse successfully");

    // Metadata
    assert_eq!(
        spec.metadata.title,
        "Sistema de Gestión de Envíos Logísticos"
    );
    assert_eq!(spec.metadata.source_type, "narrative");
    assert_eq!(spec.metadata.source_files, vec!["logistics.md".to_string()]);

    // Components
    assert_eq!(spec.components.len(), 2);
    let ship = spec
        .find_component("shipping-calculator")
        .expect("shipping-calculator must exist");
    assert_eq!(ship.name, "Calculador de Tarifas");
    assert!(
        ship.export_interfaces
            .contains(&"quote-service".to_string())
    );
    assert!(ship.dependencies.contains(&"customs-auditor".to_string()));
    assert!(
        ship.import_interfaces
            .contains(&"customs-service".to_string())
    );
    assert!(
        ship.import_interfaces
            .contains(&"host-services".to_string())
    );

    let customs = spec
        .find_component("customs-auditor")
        .expect("customs-auditor must exist");
    assert_eq!(customs.name, "Auditor Aduanero");
    assert!(
        customs
            .export_interfaces
            .contains(&"customs-service".to_string())
    );
    assert!(
        customs
            .import_interfaces
            .contains(&"host-services".to_string())
    );

    // Requirements
    assert_eq!(spec.requirements.len(), 5);
    let r1 = spec
        .find_requirement("SHIP-001")
        .expect("SHIP-001 must exist");
    assert_eq!(r1.component, "shipping-calculator");
    assert!(r1.statement.contains("500 centavos"));

    let r2 = spec
        .find_requirement("SHIP-002")
        .expect("SHIP-002 must exist");
    assert_eq!(r2.component, "shipping-calculator");

    let c1 = spec
        .find_requirement("CUST-001")
        .expect("CUST-001 must exist");
    assert_eq!(c1.component, "customs-auditor");

    // Entities
    assert_eq!(spec.entities.len(), 2);
    let quote_entity = spec
        .entities
        .iter()
        .find(|e| e.name == "ShippingQuote")
        .expect("ShippingQuote");
    assert_eq!(quote_entity.fields.len(), 4);
    assert_eq!(quote_entity.fields[0].name, "subtotal_cents");
    assert_eq!(quote_entity.fields[0].field_type, "i32");
    assert_eq!(quote_entity.fields[3].name, "is_express");
    assert_eq!(quote_entity.fields[3].field_type, "bool");

    // Operations
    assert_eq!(spec.operations.len(), 2);
    let op_quote = spec
        .operations
        .iter()
        .find(|o| o.name == "calculate_quote")
        .expect("calculate_quote");
    assert_eq!(op_quote.component, "shipping-calculator");
    assert_eq!(op_quote.wit_interface, "quote-service");
    assert_eq!(op_quote.params.len(), 2);
    assert_eq!(op_quote.params[0].name, "weight_grams");
    assert_eq!(op_quote.params[0].param_type, "i32");
    assert_eq!(op_quote.params[1].name, "is_express");
    assert_eq!(op_quote.params[1].param_type, "bool");
    assert_eq!(op_quote.returns, "ShippingQuote");
    assert!(op_quote.rules_applied.contains(&"SHIP-001".to_string()));
    assert!(op_quote.rules_applied.contains(&"SHIP-002".to_string()));

    let op_duty = spec
        .operations
        .iter()
        .find(|o| o.name == "assess_duty")
        .expect("assess_duty");
    assert_eq!(op_duty.component, "customs-auditor");
    assert_eq!(op_duty.wit_interface, "customs-service");
    assert_eq!(op_duty.returns, "i32");
}

#[test]
fn test_system_spec_json_serialization_roundtrip() {
    let interpreter = MarkdownRequirementsInterpreter;
    let spec = interpreter
        .interpret_narrative(SAMPLE_MULTI_COMPONENT_NARRATIVE, "logistics.md")
        .expect("must parse successfully");

    // Serialize to pretty JSON
    let json_str = spec.to_json_pretty().expect("serialization must succeed");
    assert!(json_str.contains("\"spec_version\": \"1.0.0\""));
    assert!(json_str.contains("\"title\": \"Sistema de Gestión de Envíos Logísticos\""));

    // Deserialize back from JSON
    let roundtripped = SystemSpec::from_json_str(&json_str).expect("deserialization must succeed");
    assert_eq!(spec, roundtripped);
}

#[test]
fn test_fail_explicit_on_empty_document() {
    let interpreter = MarkdownRequirementsInterpreter;
    let res = interpreter.interpret_narrative("", "empty.md");
    assert_eq!(res, Err(SpecError::MissingComponents));
}

#[test]
fn test_fail_explicit_on_narrative_with_only_components_but_no_requirements() {
    let md = r"# Sistema
## Componente A (`comp-a`)
Descripción de comp-a.
## Componente B (`comp-b`)
Descripción de comp-b.
";
    let interpreter = MarkdownRequirementsInterpreter;
    let res = interpreter.interpret_narrative(md, "no_reqs.md");
    assert_eq!(res, Err(SpecError::MissingRequirements));
}
