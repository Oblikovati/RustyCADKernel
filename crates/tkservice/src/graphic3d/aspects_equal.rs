//! `Graphic3d_Aspects::IsEqual`: field-by-field equality of two aspects (handles by identity).
use super::{Graphic3dAspectsFields, Graphic3dAspectsSource};

/// Whether the shared resources are the same objects.
fn same_resources(a: &Graphic3dAspectsFields, b: &Graphic3dAspectsFields) -> bool {
    a.program == b.program
        && a.texture_set == b.texture_set
        && a.marker_image == b.marker_image
        && a.hatch_style == b.hatch_style
        && a.text_font == b.text_font
}

/// Whether the colors, materials and interior settings are equal.
fn same_surface(a: &Graphic3dAspectsFields, b: &Graphic3dAspectsFields) -> bool {
    a.interior_color == b.interior_color
        && a.back_interior_color == b.back_interior_color
        && a.front_material == b.front_material
        && a.back_material == b.back_material
        && a.interior_style == b.interior_style
        && a.shading_model == b.shading_model
        && a.face_culling == b.face_culling
        && a.alpha_mode == b.alpha_mode
        && a.alpha_cutoff == b.alpha_cutoff
        && a.polygon_offset == b.polygon_offset
}

/// Whether the line, marker and text settings are equal.
fn same_styles(a: &Graphic3dAspectsFields, b: &Graphic3dAspectsFields) -> bool {
    a.line_type == b.line_type
        && a.edge_color == b.edge_color
        && a.line_width == b.line_width
        && a.line_factor == b.line_factor
        && a.line_pattern == b.line_pattern
        && a.marker_type == b.marker_type
        && a.marker_scale == b.marker_scale
        && a.text_style == b.text_style
        && a.text_display_type == b.text_display_type
        && a.text_font_aspect == b.text_font_aspect
        && a.text_angle == b.text_angle
}

/// Whether the flags are equal.
fn same_flags(a: &Graphic3dAspectsFields, b: &Graphic3dAspectsFields) -> bool {
    a.to_skip_first_edge == b.to_skip_first_edge
        && a.to_distinguish_materials == b.to_distinguish_materials
        && a.to_use_vertex_color_for_back_faces == b.to_use_vertex_color_for_back_faces
        && a.to_draw_edges == b.to_draw_edges
        && a.to_draw_silhouette == b.to_draw_silhouette
        && a.to_map_texture == b.to_map_texture
        && a.is_text_zoomable == b.is_text_zoomable
}

/// The `IsEqual` member of `Graphic3d_Aspects`.
pub trait Graphic3dAspectsEqual: Graphic3dAspectsSource {
    /// Whether `other` is this object or has the same fields (resources by identity, floats
    /// by value, colors with the color tolerance); `aspects.is_equal(&*other)`.
    fn is_equal(&self, other: &dyn Graphic3dAspectsSource) -> bool {
        let (mine, theirs) = (self.aspects_base(), other.aspects_base());
        if std::ptr::eq(mine, theirs) {
            return true;
        }
        let (a, b) = (mine.fields.read().clone(), theirs.fields.read().clone());
        same_resources(&a, &b) && same_surface(&a, &b) && same_styles(&a, &b) && same_flags(&a, &b)
    }
}
impl<T: Graphic3dAspectsSource + ?Sized> Graphic3dAspectsEqual for T {}
