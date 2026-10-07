//! Graphic3d: the 3D presentation data model and the graphic driver interfaces (OCCT package
//! Graphic3d).
//!
//! Each class keeps its package prefix (`Graphic3d_Camera` is `Graphic3dCamera`); abstract
//! driver classes are traits over embedded bases (visualization-design.md section 1).
mod array_classes;
mod array_edges;
mod array_of_primitives;
mod array_validity;
mod array_vertices;
mod aspect_classes;
mod aspects;
mod aspects_equal;
mod aspects_members;
mod aspects_style;
mod attrib_buffer;
mod bound_buffer;
mod bsdf;
mod buffer;
mod buffer_members;
mod cube_map;
mod cube_map_order;
mod cube_map_packed;
mod cube_map_separate;
mod hatch_style;
mod index_buffer;
mod marker_image;
mod marker_standard;
mod marker_tables;
mod material_aspect;
mod material_enums;
mod material_specs;
mod material_table;
mod pbr_material;
mod render_enums;
mod shader_object;
mod shader_program;
mod shader_variable;
mod small_values;
mod stats_enums;
mod texture_1d;
mod texture_2d;
mod texture_3d;
mod texture_enums;
mod texture_env;
mod texture_map;
mod texture_params;
mod texture_root;
mod texture_set;

pub use array_classes::{
    Graphic3dArrayOfPoints, Graphic3dArrayOfPolygons, Graphic3dArrayOfPolylines,
    Graphic3dArrayOfQuadrangleStrips, Graphic3dArrayOfQuadrangles, Graphic3dArrayOfSegments,
    Graphic3dArrayOfTriangleFans, Graphic3dArrayOfTriangleStrips, Graphic3dArrayOfTriangles,
    Graphic3dArrayOptions,
};
pub use array_of_primitives::{Graphic3dArrayOfPrimitives, Graphic3dAttributeSlot};
pub use aspect_classes::{
    Graphic3dAspectFillArea3d, Graphic3dAspectLine3d, Graphic3dAspectMarker3d,
    Graphic3dAspectText3d,
};
pub use aspects::{
    Graphic3dAspects, Graphic3dAspectsBase, Graphic3dAspectsFields, Graphic3dAspectsSource,
    graphic3d_aspects_dump_json,
};
pub use aspects_equal::Graphic3dAspectsEqual;
pub use aspects_members::{Graphic3dAspectsMembers, Graphic3dAspectsRenderMembers};
pub use aspects_style::{
    Graphic3dAspectsEdgeMembers, Graphic3dAspectsLineMembers, Graphic3dAspectsTextMembers,
    graphic3d_default_line_pattern_for_type, graphic3d_default_line_type_for_pattern,
};
pub use attrib_buffer::Graphic3dAttribBuffer;
pub use bound_buffer::{Graphic3dBoundBuffer, Graphic3dBoundBufferState};
pub use bsdf::{Graphic3dBsdf, Graphic3dFresnel, Graphic3dFresnelModel};
pub use buffer::{
    Graphic3dAttribute, Graphic3dBuffer, Graphic3dBufferBase, Graphic3dBufferSource,
    Graphic3dBufferState, Graphic3dTypeOfAttribute, Graphic3dTypeOfData,
    graphic3d_buffer_dump_json,
};
pub use buffer_members::{Graphic3dBufferMembers, Graphic3dBufferValue};
pub use cube_map::{
    Graphic3dCubeMap, Graphic3dCubeMapBase, Graphic3dCubeMapIteration, Graphic3dCubeMapMembers,
};
pub use cube_map_order::{Graphic3dCubeMapOrder, Graphic3dValidatedCubeMapOrder};
pub use cube_map_packed::{Graphic3dCubeMapPacked, graphic3d_cube_map_packed_check_order};
pub use cube_map_separate::Graphic3dCubeMapSeparate;
pub use hatch_style::Graphic3dHatchStyle;
pub use index_buffer::{
    Graphic3dIndexBuffer, Graphic3dIndexBufferMembers, Graphic3dIndexBufferSource,
    Graphic3dMutableIndexBuffer,
};
pub use marker_image::Graphic3dMarkerImage;
pub use material_aspect::Graphic3dMaterialAspect;
pub use material_enums::*;
pub use pbr_material::{Graphic3dPbrMaterial, material_definition_error};
pub use render_enums::*;
pub use shader_object::{
    Graphic3dShaderObject, Graphic3dShaderObjectVariable, Graphic3dShaderVariableList,
};
pub use shader_program::Graphic3dShaderProgram;
pub use shader_variable::{
    Graphic3dShaderAttribute, Graphic3dShaderVariable, Graphic3dUniformFloat, Graphic3dUniformInt,
    Graphic3dUniformMat3, Graphic3dUniformMat4, Graphic3dUniformValue, Graphic3dUniformValueTypeId,
    Graphic3dUniformVec2, Graphic3dUniformVec2i, Graphic3dUniformVec3, Graphic3dUniformVec3i,
    Graphic3dUniformVec4, Graphic3dUniformVec4i, Graphic3dValueInterface,
};
pub use small_values::{Graphic3dBufferRange, Graphic3dPolygonOffset, Graphic3dVertex};
pub use stats_enums::*;
pub use texture_1d::{
    Graphic3dTexture1D, Graphic3dTexture1DBase, Graphic3dTexture1DMembers,
    Graphic3dTexture1Dmanual, Graphic3dTexture1Dsegment,
};
pub use texture_2d::{
    Graphic3dTexture2D, Graphic3dTexture2DBase, Graphic3dTexture2DMembers,
    Graphic3dTexture2DSource, Graphic3dTexture2Dplane,
};
pub use texture_3d::Graphic3dTexture3D;
pub use texture_enums::*;
pub use texture_env::Graphic3dTextureEnv;
pub use texture_map::{Graphic3dTextureMap, Graphic3dTextureMapMembers};
pub use texture_params::Graphic3dTextureParams;
pub use texture_root::{
    Graphic3dTextureRoot, Graphic3dTextureRootBase, Graphic3dTextureRootFields,
    Graphic3dTextureRootMembers, graphic3d_convert_to_compatible, graphic3d_texture_root_get_image,
    graphic3d_textures_folder,
};
pub use texture_set::Graphic3dTextureSet;
