// Keep the native AIS namespace flat while grouping the ColoredShape family registrations.
mod colored_drawer;
pub use colored_drawer::{AisColoredDrawer, AisColoredDrawerFlags, AisColoredDrawerParent};

#[cfg(test)]
mod colored_drawer_tests;

#[cfg(test)]
mod colored_drawer_runtime_tests;

mod shape_drawer_map;
mod shape_drawer_node;
pub use shape_drawer_map::AisShapeDrawerMap;

mod shape_drawer_container;
mod shape_drawer_references;
pub use shape_drawer_container::{AisShapeDrawerMapMut, AisShapeDrawerMapRef};
pub use shape_drawer_references::{AisShapeDrawerNodeRef, AisShapeDrawerValueMut};

mod shape_drawer_lifecycle;

#[cfg(test)]
mod shape_drawer_map_tests;
#[cfg(test)]
mod shape_drawer_oracle_inputs;

#[cfg(test)]
mod shape_drawer_lifecycle_tests;

#[cfg(test)]
mod shape_drawer_oracle_rows;

#[cfg(test)]
mod shape_drawer_reference_tests;

mod colored_shape;
mod colored_shape_members;
pub use colored_shape::{
    AisColoredShapeBase, AisColoredShapeSource, ais_colored_shape_clear_custom_aspects,
    ais_colored_shape_custom_aspects,
};
pub use colored_shape_members::AisColoredShapeMembers;
mod colored_shape_object;
pub use colored_shape_object::AisColoredShape;
#[cfg(test)]
mod colored_shape_owner_tests;
mod shape_reference;
pub use shape_reference::{AisShapeMembers, AisShapeRef};
#[cfg(test)]
mod colored_shape_dispatch_fixture;
#[cfg(test)]
mod colored_shape_dispatch_tests;
mod colored_shape_expansion;
#[cfg(test)]
mod colored_shape_parent_drop;
mod shape_drawer_cursor;
pub use colored_shape_expansion::AisColoredShapeExpansion;

mod colored_shape_expansion_fill;
pub use colored_shape_expansion_fill::ais_colored_shape_fill_subshape_drawer_map;
#[cfg(test)]
mod colored_shape_alias_inputs;
#[cfg(test)]
mod colored_shape_alias_tests;
mod colored_shape_custom_appearance;
#[cfg(test)]
mod colored_shape_expansion_inputs;
#[cfg(test)]
mod colored_shape_expansion_tests;
#[cfg(test)]
mod shape_drawer_cursor_tests;
pub use colored_shape_custom_appearance::AisColoredShapeCustomAppearance;
mod colored_shape_global_appearance;
pub use colored_shape_global_appearance::{
    ais_colored_shape_set_color, ais_colored_shape_set_material,
    ais_colored_shape_set_transparency, ais_colored_shape_set_width,
};
mod colored_compound_map;
mod colored_compound_node;
mod colored_compound_references;
mod colored_compound_value;
#[cfg(test)]
mod colored_shape_appearance_snapshot;
#[cfg(test)]
mod colored_shape_appearance_tests;
mod colored_shape_copy_appearance;
#[cfg(test)]
mod colored_shape_copy_tests;
#[cfg(test)]
mod colored_shape_custom_dispatch_tests;
pub use colored_compound_map::AisColoredDrawerCompounds;
pub use colored_compound_references::AisColoredCompoundRef;
#[cfg(test)]
mod colored_compound_drop_probe;
#[cfg(test)]
mod colored_compound_map_tests;
mod colored_shape_dispatch;
mod colored_shape_dispatch_shell;
pub use colored_shape_dispatch::{AisColoredOpenedCompounds, ais_colored_shape_dispatch_colors};
mod colored_compound_cursor;
#[cfg(test)]
mod colored_compound_reference_graphics;
#[cfg(test)]
mod colored_compound_reference_group;
#[cfg(test)]
mod colored_compound_value_drop_probe;
#[cfg(test)]
mod colored_compound_wf_live_tests;
#[cfg(test)]
mod colored_compound_wf_reference_tests;
#[cfg(test)]
mod colored_shape_closed_dispatch_tests;
mod colored_shape_emission;
mod colored_shape_emission_shading;
#[cfg(test)]
mod colored_shape_grouping_tests;
pub use colored_shape_emission::ais_colored_shape_add_custom_props;
mod colored_shape_compute;
mod shape_compute_bounds;
pub use colored_shape_compute::ais_colored_shape_compute;

mod colored_shape_selection;
mod shape_selection;
pub use colored_shape_selection::{
    ais_colored_shape_compute_selection, ais_colored_shape_compute_subshape_selection,
};
pub use shape_selection::{
    ais_shape_compute_selection, ais_shape_compute_selection_with_messenger,
    ais_shape_selection_type,
};
#[cfg(test)]
mod colored_compound_cursor_tests;
