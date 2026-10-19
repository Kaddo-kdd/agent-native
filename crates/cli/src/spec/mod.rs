//! System specification interpretation, models and validation.

pub mod error;
pub mod interpreter;
pub mod model;

pub use error::SpecError;
pub use interpreter::{MarkdownRequirementsInterpreter, RequirementsInterpreter};
pub use model::{
    ComponentSpec, EntityField, EntitySpec, OperationParam, OperationSpec, RequirementSource,
    RequirementSpec, SpecMetadata, SystemSpec,
};
