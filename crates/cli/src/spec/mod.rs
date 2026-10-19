pub mod diff;
pub mod error;
pub mod interpreter;
pub mod model;
pub mod reconciler;

pub use diff::{SpecDiff, compute_spec_diff};
pub use error::SpecError;
pub use interpreter::{MarkdownRequirementsInterpreter, RequirementsInterpreter};
pub use model::{
    ComponentSpec, EntityField, EntitySpec, OperationParam, OperationSpec, RequirementSource,
    RequirementSpec, SpecMetadata, SystemSpec, generate_wit_for_component,
};
pub use reconciler::reconcile_spec;
