//! Geometry mutation marks existing presentations stale; native selection is refreshed by redisplay.
use super::{AisViewCubeGeometry, AisViewCubePresentationSettings, AisViewCubePresentationSource};
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkernel::{
    precision,
    standard::{Failure, exceptions::out_of_range},
};
macro_rules! cube_dimension_member {
    ($get:ident,$set:ident,$field:ident) => {
        #[doc = concat!("Read native geometry; `cube.",stringify!($get),"()`.")]
        fn $get(&self) -> f64 { self.view_cube_presentation_base().settings.read().geometry.$field }
        #[doc = concat!("Mark geometry stale only beyond Confusion; `cube.",stringify!($set),"(3.0)`.")]
        fn $set(&self, value: f64) { cube_update_dimension(self,value,|g| &mut g.$field); }
    };
}
macro_rules! cube_visibility_member {
    ($get:ident,$set:ident,$field:ident) => {
        #[doc = concat!("Read native visibility; `cube.",stringify!($get),"()`.")]
        fn $get(&self) -> bool {
            self.view_cube_presentation_base().settings.read().$field
        }
        #[doc = concat!("Mark presentation stale on change; `cube.",stringify!($set),"(false)`.")]
        fn $set(&self, value: bool) {
            cube_update_visibility(self, value, |s| &mut s.$field);
        }
    };
}
/// Native geometry operations over retained state; `cube.set_size(100.0,true)`.
pub trait AisViewCubeGeometryMembers: AisViewCubePresentationSource {
    cube_dimension_member!(
        box_facet_extension,
        set_box_facet_extension,
        facet_extension
    );
    cube_dimension_member!(axes_padding, set_axes_padding, axes_padding);
    cube_dimension_member!(box_edge_gap, set_box_edge_gap, edge_gap);
    cube_dimension_member!(box_edge_min_size, set_box_edge_min_size, edge_min_size);
    cube_dimension_member!(
        box_corner_min_size,
        set_box_corner_min_size,
        corner_min_size
    );
    cube_dimension_member!(axes_radius, set_axes_radius, axes_radius);
    cube_dimension_member!(axes_cone_radius, set_axes_cone_radius, axes_cone_radius);
    cube_dimension_member!(
        axes_sphere_radius,
        set_axes_sphere_radius,
        axes_sphere_radius
    );
    cube_visibility_member!(to_draw_axes, set_draw_axes, draw_axes);
    cube_visibility_member!(to_draw_edges, set_draw_edges, draw_edges);
    cube_visibility_member!(to_draw_vertices, set_draw_vertices, draw_corners);
    /// Read retained side width; `cube.size()`.
    fn size(&self) -> f64 {
        self.view_cube_presentation_base()
            .settings
            .read()
            .geometry
            .size
    }
    /// Always assign size; independently adapt positive extension/padding and font; `cube.set_size(70.0,true)`.
    fn set_size(&self, value: f64, adapt: bool) {
        let previous = self.size();
        self.view_cube_presentation_base()
            .settings
            .write()
            .geometry
            .size = value;
        if adapt {
            cube_adapt_dimensions(self, value);
        }
        if (previous - value).abs() > precision::confusion() {
            self.set_to_update_all();
        }
    }
    /// Read retained rounding ratio; `cube.round_radius()`.
    fn round_radius(&self) -> f64 {
        self.view_cube_presentation_base()
            .settings
            .read()
            .geometry
            .round_radius
    }
    /// Validate the native [0,0.5] range before marking geometry stale; `cube.set_round_radius(0.2)?`.
    fn set_round_radius(&self, value: f64) -> Result<(), Failure> {
        if value < 0.0 || value > 0.5 {
            return Err(out_of_range(&format!(
                "AIS_ViewCube round radius={value}; expected [0, 0.5]"
            )));
        }
        cube_update_dimension(self, value, |g| &mut g.round_radius);
        Ok(())
    }
}
impl<T: AisViewCubePresentationSource + ?Sized> AisViewCubeGeometryMembers for T {}
fn cube_update_dimension<C: AisViewCubePresentationSource + ?Sized>(
    cube: &C,
    value: f64,
    project: fn(&mut AisViewCubeGeometry) -> &mut f64,
) {
    let mut settings = cube.view_cube_presentation_base().settings.write();
    let retained = project(&mut settings.geometry);
    let changed = (*retained - value).abs() > precision::confusion();
    if !changed {
        return;
    }
    *retained = value;
    drop(settings);
    cube.set_to_update_all();
}
fn cube_update_visibility<C: AisViewCubePresentationSource + ?Sized>(
    cube: &C,
    value: bool,
    project: fn(&mut AisViewCubePresentationSettings) -> &mut bool,
) {
    let mut settings = cube.view_cube_presentation_base().settings.write();
    let retained = project(&mut settings);
    if *retained == value {
        return;
    }
    *retained = value;
    drop(settings);
    cube.set_to_update_all();
}
fn cube_adapt_dimensions<C: AisViewCubeGeometryMembers + ?Sized>(cube: &C, size: f64) {
    if cube.box_facet_extension() > 0.0 {
        cube.set_box_facet_extension(size * 0.15);
    }
    if cube.axes_padding() > 0.0 {
        cube.set_axes_padding(size * 0.1);
    }
    let text = cube.attributes().text_aspect();
    if (text.height() - size * 0.16).abs() > precision::confusion() {
        text.set_height(size * 0.16);
        cube.set_to_update_all();
    }
}
