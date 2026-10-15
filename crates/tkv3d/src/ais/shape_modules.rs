// Shape-family registrations stay in the AIS namespace; leaf and parent interfaces share native RTTI.
mod shape;
mod shape_object;
pub use shape_object::{
    AisShape, AisShapeParameters, ais_shape_compute, ais_shape_compute_hlr,
    ais_shape_compute_hlr_presentation, ais_shape_dump_json, ais_shape_selection_mode,
};
mod textured_shape;
mod textured_shape_file_ref;
mod textured_shape_mapping;
mod textured_shape_source;
mod textured_shape_sources;
pub use shape::{AisShapeBase, AisShapeFields, AisShapeSource};
pub use textured_shape::{AisTexturedShapeBase, AisTexturedShapeFields};
pub use textured_shape_file_ref::AisTexturedShapeFileRef;
pub use textured_shape_mapping::AisTexturedShapeMappingMembers;
pub use textured_shape_source::{AisTexturedShapeMembers, AisTexturedShapeSource};
pub use textured_shape_sources::{
    ais_textured_shape_set_texture_file_name, ais_textured_shape_set_texture_pix_map,
};

mod textured_shape_aspects;
mod textured_shape_factory;
pub use textured_shape_aspects::ais_textured_shape_update_attributes;
pub use textured_shape_factory::{
    AisNativeTexturedTextureFactory, AisTexturedTextureFactory, AisTexturedTextureSource,
};

mod shape_appearance;
mod shape_unset_appearance;
mod shape_unset_shading;
pub use shape_appearance::{
    ais_shape_replace_with_new_own_aspects, ais_shape_set_color, ais_shape_set_material,
};
pub use shape_unset_appearance::ais_shape_unset_color;
pub use shape_unset_shading::ais_shape_unset_material;

mod textured_shape_pixmap_ref;
pub use textured_shape_pixmap_ref::AisTexturedShapePixMapRef;
mod textured_shape_appearance;
pub use textured_shape_appearance::{
    ais_textured_shape_set_color, ais_textured_shape_set_material, ais_textured_shape_unset_color,
    ais_textured_shape_unset_material,
};

mod shape_transparency;
pub use shape_transparency::{ais_shape_set_transparency, ais_shape_unset_transparency};

mod shape_width;
pub use shape_width::{ais_shape_set_width, ais_shape_unset_width};

mod shape_bounds;
mod textured_shape_compute;
mod textured_shape_object;
pub use textured_shape_compute::ais_textured_shape_compute;
pub use textured_shape_object::{AisTexturedShape, AisTexturedShapeParent};
mod shape_appearance_queries;
pub use shape_appearance_queries::{ais_shape_color_into, ais_shape_material, ais_shape_transparency};
