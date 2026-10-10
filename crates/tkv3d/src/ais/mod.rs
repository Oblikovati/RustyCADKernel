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

mod context_detection_cursor;
mod context_detection_cycle;
mod context_highlight;
mod context_move_to;
mod detection_status;
pub use context_highlight::AisHighlightStyle;
pub use detection_status::AisStatusOfDetection;

mod context_highlight_owners;
mod context_owner_walk;
mod context_selected_members;
mod context_selection_actions;
mod context_unhighlight_owners;
mod pick_status;
pub use pick_status::AisStatusOfPick;

mod context_global_highlight;
mod context_object_highlight;
mod context_select_pick;

mod context_select_legacy;
mod context_set_selected;
mod context_toggle_selected;
