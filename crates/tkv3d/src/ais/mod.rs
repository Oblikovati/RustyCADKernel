//! The AIS package: interactive objects and the interactive context.
mod selection_scheme;

pub use selection_scheme::AisSelectionScheme;
mod global_status;
mod select_status;
mod selection;
mod selection_edit;
mod selection_members;
pub use global_status::AisGlobalStatus;
pub use select_status::AisSelectStatus;
pub use selection::{
    AisSelectedOwner, AisSelection, AisSelectionBase, AisSelectionFields, AisSelectionSource,
};
pub use selection_edit::{ais_selection_add_select, ais_selection_clear, ais_selection_select};
pub use selection_members::{AisSelectedObjects, AisSelectionMembers, AisSelectionValue};
mod context_styles;
mod interactive_context;
mod interactive_enums;
mod interactive_object;
pub use interactive_context::{AisInteractiveContext, AisInteractiveContextFields};
pub use interactive_enums::{AisDragAction, AisKindOfInteractive};
pub use interactive_object::{
    AisInteractiveObject, AisInteractiveObjectBase, AisInteractiveObjectFields,
};
mod context_members;
pub use context_members::AisContextHandle;

mod context_updates;
mod interactive_object_members;
pub use interactive_object_members::{
    AisApplicationOwner, AisInteractiveObjectMembers, ais_interactive_object_dump_json,
};
