//! The aspect members of `Prs3d_Drawer`: each aspect is own when set, else the link's.
use super::{
    Prs3dArrowAspect, Prs3dDatumAspect, Prs3dDimensionAspect, Prs3dDrawer, Prs3dIsoAspect,
    Prs3dLineAspect, Prs3dPlaneAspect, Prs3dPointAspect, Prs3dShadingAspect, Prs3dTextAspect,
};
use rustycad_tkernel::standard::Handle;

/// Declares the getter, setter and `HasOwn` members of an aspect handle.
macro_rules! drawer_aspect_members {
    ($get:ident, $set:ident, $has:ident, $field:ident, $ty:ty, $doc:literal) => {
        #[doc = concat!("`", $doc, "`: the own aspect, else the link's; `drawer.", stringify!($get), "()`.")]
        pub fn $get(&self) -> Handle<$ty> {
            self.aspect_or_link(|f| f.$field.clone(), Prs3dDrawer::$get)
        }
        #[doc = concat!("Sets the own `", $doc, "` (null to inherit); `drawer.", stringify!($set), "(aspect)`.")]
        pub fn $set(&self, aspect: Handle<$ty>) {
            self.edit(|f| f.$field = aspect);
        }
        #[doc = concat!("Whether `", $doc, "` is own; `drawer.", stringify!($has), "()`.")]
        pub fn $has(&self) -> bool {
            !self.fields.read().$field.is_null()
        }
    };
}

impl Prs3dDrawer {
    drawer_aspect_members!(
        u_iso_aspect,
        set_u_iso_aspect,
        has_own_u_iso_aspect,
        u_iso_aspect,
        Prs3dIsoAspect,
        "UIsoAspect"
    );
    drawer_aspect_members!(
        v_iso_aspect,
        set_v_iso_aspect,
        has_own_v_iso_aspect,
        v_iso_aspect,
        Prs3dIsoAspect,
        "VIsoAspect"
    );
    drawer_aspect_members!(
        wire_aspect,
        set_wire_aspect,
        has_own_wire_aspect,
        wire_aspect,
        Prs3dLineAspect,
        "WireAspect"
    );
    drawer_aspect_members!(
        point_aspect,
        set_point_aspect,
        has_own_point_aspect,
        point_aspect,
        Prs3dPointAspect,
        "PointAspect"
    );
    drawer_aspect_members!(
        line_aspect,
        set_line_aspect,
        has_own_line_aspect,
        line_aspect,
        Prs3dLineAspect,
        "LineAspect"
    );
    drawer_aspect_members!(
        text_aspect,
        set_text_aspect,
        has_own_text_aspect,
        text_aspect,
        Prs3dTextAspect,
        "TextAspect"
    );
    drawer_aspect_members!(
        shading_aspect,
        set_shading_aspect,
        has_own_shading_aspect,
        shading_aspect,
        Prs3dShadingAspect,
        "ShadingAspect"
    );
    drawer_aspect_members!(
        seen_line_aspect,
        set_seen_line_aspect,
        has_own_seen_line_aspect,
        seen_line_aspect,
        Prs3dLineAspect,
        "SeenLineAspect"
    );
    drawer_aspect_members!(
        plane_aspect,
        set_plane_aspect,
        has_own_plane_aspect,
        plane_aspect,
        Prs3dPlaneAspect,
        "PlaneAspect"
    );
    drawer_aspect_members!(
        arrow_aspect,
        set_arrow_aspect,
        has_own_arrow_aspect,
        arrow_aspect,
        Prs3dArrowAspect,
        "ArrowAspect"
    );
    drawer_aspect_members!(
        hidden_line_aspect,
        set_hidden_line_aspect,
        has_own_hidden_line_aspect,
        hidden_line_aspect,
        Prs3dLineAspect,
        "HiddenLineAspect"
    );
    drawer_aspect_members!(
        vector_aspect,
        set_vector_aspect,
        has_own_vector_aspect,
        vector_aspect,
        Prs3dLineAspect,
        "VectorAspect"
    );
    drawer_aspect_members!(
        datum_aspect,
        set_datum_aspect,
        has_own_datum_aspect,
        datum_aspect,
        Prs3dDatumAspect,
        "DatumAspect"
    );
    drawer_aspect_members!(
        section_aspect,
        set_section_aspect,
        has_own_section_aspect,
        section_aspect,
        Prs3dLineAspect,
        "SectionAspect"
    );
    drawer_aspect_members!(
        free_boundary_aspect,
        set_free_boundary_aspect,
        has_own_free_boundary_aspect,
        free_boundary_aspect,
        Prs3dLineAspect,
        "FreeBoundaryAspect"
    );
    drawer_aspect_members!(
        un_free_boundary_aspect,
        set_un_free_boundary_aspect,
        has_own_un_free_boundary_aspect,
        un_free_boundary_aspect,
        Prs3dLineAspect,
        "UnFreeBoundaryAspect"
    );
    drawer_aspect_members!(
        face_boundary_aspect,
        set_face_boundary_aspect,
        has_own_face_boundary_aspect,
        face_boundary_aspect,
        Prs3dLineAspect,
        "FaceBoundaryAspect"
    );
    drawer_aspect_members!(
        dimension_aspect,
        set_dimension_aspect,
        has_own_dimension_aspect,
        dimension_aspect,
        Prs3dDimensionAspect,
        "DimensionAspect"
    );
}
