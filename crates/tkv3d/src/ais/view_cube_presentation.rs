//! Martensite ViewCube presentation state. Selection and animation remain application hooks.
use super::{AisInteractiveObject, AisInteractiveObjectBase};
use crate::v3d::V3dTypeOfOrientation;
use crate::{prs_mgr::PrsMgrTypeOfPresentation3d, prs3d::Prs3dShadingAspect};
use rustycad_tkernel::{
    standard::{Failure, Handle, SharedState},
    tcollection::AsciiString,
};
use rustycad_tkservice::graphic3d::Graphic3dArrayOfTriangles;
/// Native geometry controls after default SetSize(70); `geometry.size=100.0`.
#[derive(Clone, Debug)]
pub struct AisViewCubeGeometry {
    /// Native retained size; configure before recomputing the presentation.
    pub size: f64,
    /// Native retained facet extension; configure before recomputing the presentation.
    pub facet_extension: f64,
    /// Native retained edge gap; configure before recomputing the presentation.
    pub edge_gap: f64,
    /// Native retained edge min size; configure before recomputing the presentation.
    pub edge_min_size: f64,
    /// Native retained corner min size; configure before recomputing the presentation.
    pub corner_min_size: f64,
    /// Native retained round radius; configure before recomputing the presentation.
    pub round_radius: f64,
    /// Native retained axes padding; configure before recomputing the presentation.
    pub axes_padding: f64,
    /// Native retained axes radius; configure before recomputing the presentation.
    pub axes_radius: f64,
    /// Native retained axes cone radius; configure before recomputing the presentation.
    pub axes_cone_radius: f64,
    /// Native retained axes sphere radius; configure before recomputing the presentation.
    pub axes_sphere_radius: f64,
}
impl Default for AisViewCubeGeometry {
    fn default() -> Self {
        Self {
            size: 70.0,
            facet_extension: 10.5,
            edge_gap: 0.0,
            edge_min_size: 2.0,
            corner_min_size: 2.0,
            round_radius: 0.0,
            axes_padding: 7.0,
            axes_radius: 1.0,
            axes_cone_radius: 3.0,
            axes_sphere_radius: 4.0,
        }
    }
}
/// Retained presentation policy; mutate before redisplaying the owning AIS object.
/// Example: `base.settings.write().draw_axes=false`.
#[derive(Clone, Debug)]
pub struct AisViewCubePresentationSettings {
    /// Native retained geometry; configure before recomputing the presentation.
    pub geometry: AisViewCubeGeometry,
    /// Native retained draw axes; configure before recomputing the presentation.
    pub draw_axes: bool,
    /// Native retained draw edges; configure before recomputing the presentation.
    pub draw_edges: bool,
    /// Native retained draw corners; configure before recomputing the presentation.
    pub draw_corners: bool,
    /// Native retained is y up; configure before recomputing the presentation.
    pub is_y_up: bool,
    /// Native retained side labels; configure before recomputing the presentation.
    pub side_labels: [AsciiString; 6],
    /// Native retained axes labels; configure before recomputing the presentation.
    pub axes_labels: [AsciiString; 3],
}
impl Default for AisViewCubePresentationSettings {
    fn default() -> Self {
        Self {
            geometry: AisViewCubeGeometry::default(),
            draw_axes: true,
            draw_edges: true,
            draw_corners: true,
            is_y_up: false,
            side_labels: ["RIGHT", "BACK", "TOP", "LEFT", "FRONT", "BOTTOM"].map(AsciiString::from),
            axes_labels: ["X", "Y", "Z"].map(AsciiString::from),
        }
    }
}
/// Native geometry sizing pass, including rounded fan overestimates; `counts.vertices`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AisViewCubeTriangleCounts {
    /// Native retained vertices; configure before recomputing the presentation.
    pub vertices: i32,
    /// Native retained triangles; configure before recomputing the presentation.
    pub triangles: i32,
}
/// Embeddable geometry/aspects for an AIS subclass; `AisViewCubePresentationBase::new()?`.
/// This initializes display state, not a concrete AIS_ViewCube or its animation/owners.
/// Bind the owning object to its interactive context before computing axes or outlines;
/// their appearance comes from the linked default drawer unless explicitly supplied.
pub struct AisViewCubePresentationBase {
    /// Native retained interactive; configure before recomputing the presentation.
    pub interactive: AisInteractiveObjectBase,
    /// Native retained settings; configure before recomputing the presentation.
    pub settings: SharedState<AisViewCubePresentationSettings>,
    /// Native retained edge aspect; configure before recomputing the presentation.
    pub edge_aspect: Handle<Prs3dShadingAspect>,
    /// Native retained corner aspect; configure before recomputing the presentation.
    pub corner_aspect: Handle<Prs3dShadingAspect>,
}
impl AisViewCubePresentationBase {
    /// Create native default presentation state; `AisViewCubePresentationBase::new()?`.
    pub fn new() -> Result<Self, Failure> {
        let interactive = AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView);
        let (edge_aspect, corner_aspect) =
            super::view_cube_appearance::initialize_cube_appearance(&interactive)?;
        Ok(Self {
            interactive,
            settings: SharedState::default(),
            edge_aspect,
            corner_aspect,
        })
    }
}
/// Native virtual geometry contract; subclasses retain real selection/animation behavior.
/// Example: implement `view_cube_presentation_base()` and use `ais_view_cube_compute` in Compute.
pub trait AisViewCubePresentationSource: AisInteractiveObject {
    /// Borrow the subclass display state; `cube.view_cube_presentation_base()`.
    fn view_cube_presentation_base(&self) -> &AisViewCubePresentationBase;
    /// Dispatch native side/edge/corner geometry; null arrays perform the sizing pass.
    /// Example: `cube.create_box_part_triangles(&array,&mut counts,orientation)?`.
    fn create_box_part_triangles(
        &self,
        array: &Handle<Graphic3dArrayOfTriangles>,
        counts: &mut AisViewCubeTriangleCounts,
        orientation: V3dTypeOfOrientation,
    ) -> Result<(), Failure> {
        super::view_cube_geometry::ais_view_cube_create_box_part_triangles(
            self,
            array,
            counts,
            orientation,
        )
    }
    /// Append a side or count its capacity; `cube.create_box_side_triangles(&array,&mut counts,orientation)?`.
    fn create_box_side_triangles(
        &self,
        array: &Handle<Graphic3dArrayOfTriangles>,
        counts: &mut AisViewCubeTriangleCounts,
        orientation: V3dTypeOfOrientation,
    ) -> Result<(), Failure> {
        super::view_cube_geometry::ais_view_cube_create_box_side_triangles(
            self,
            array,
            counts,
            orientation,
        )
    }
    /// Append an edge or count its capacity; `cube.create_box_edge_triangles(&array,&mut counts,orientation)?`.
    fn create_box_edge_triangles(
        &self,
        array: &Handle<Graphic3dArrayOfTriangles>,
        counts: &mut AisViewCubeTriangleCounts,
        orientation: V3dTypeOfOrientation,
    ) -> Result<(), Failure> {
        super::view_cube_geometry::ais_view_cube_create_box_edge_triangles(
            self,
            array,
            counts,
            orientation,
        )
    }
    /// Append a corner or count its capacity; `cube.create_box_corner_triangles(&array,&mut counts,orientation)?`.
    fn create_box_corner_triangles(
        &self,
        array: &Handle<Graphic3dArrayOfTriangles>,
        counts: &mut AisViewCubeTriangleCounts,
        orientation: V3dTypeOfOrientation,
    ) -> Result<(), Failure> {
        super::view_cube_corners::ais_view_cube_create_box_corner_triangles(
            self,
            array,
            counts,
            orientation,
        )
    }
}
