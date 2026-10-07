// Rust port of DataExchange/TKDE/DE/DE_ShapeFixParameters.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DE_ShapeFixParameters`: the shape healing parameters of the exchange formats. The
//! header belongs to TKDE, which TKXSBase includes without linking it (TKDE does not
//! depend on TKXSBase); TKDE is not ported, so the struct lives here and its parity rows
//! stay with TKDE.
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `DE_ShapeFixParameters::FixMode`: whether a fix runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum DEShapeFixMode {
    /// `FixOrNot`: depending on the situation.
    FixOrNot = -1,
    /// `NotFix`.
    NotFix = 0,
    /// `Fix`: anyway.
    Fix = 1,
}

/// Declares the parameters with their native defaults.
macro_rules! shape_fix_parameters {
    ($($field:ident: $ty:ty = $default:expr,)*) => {
        /// `DE_ShapeFixParameters`, with the native default values.
        ///
        /// ```
        /// use rustycad_tkxsbase::xs_algo::{DEShapeFixMode, DEShapeFixParameters};
        /// let params = DEShapeFixParameters::default();
        /// assert_eq!(params.create_open_solid_mode, DEShapeFixMode::NotFix);
        /// assert_eq!(params.tolerance3d, 1.0e-6);
        /// ```
        #[derive(Clone, Debug, PartialEq)]
        pub struct DEShapeFixParameters {
            $(#[doc = concat!("`", stringify!($field), "`.")] pub $field: $ty,)*
        }
        impl Default for DEShapeFixParameters {
            fn default() -> Self {
                Self { $($field: $default,)* }
            }
        }
    };
}

use DEShapeFixMode::{Fix, FixOrNot, NotFix};

shape_fix_parameters! {
    tolerance3d: f64 = 1.0e-6,
    max_tolerance3d: f64 = 1.0,
    min_tolerance3d: f64 = 1.0e-7,
    detalization_level: ShapeEnum = ShapeEnum::Vertex,
    non_manifold: bool = false,
    fix_free_shell_mode: DEShapeFixMode = FixOrNot,
    fix_free_face_mode: DEShapeFixMode = FixOrNot,
    fix_free_wire_mode: DEShapeFixMode = FixOrNot,
    fix_same_parameter_mode: DEShapeFixMode = FixOrNot,
    fix_solid_mode: DEShapeFixMode = FixOrNot,
    fix_shell_orientation_mode: DEShapeFixMode = FixOrNot,
    create_open_solid_mode: DEShapeFixMode = NotFix,
    fix_shell_mode: DEShapeFixMode = FixOrNot,
    fix_face_orientation_mode: DEShapeFixMode = FixOrNot,
    fix_face_mode: DEShapeFixMode = FixOrNot,
    fix_wire_mode: DEShapeFixMode = FixOrNot,
    fix_orientation_mode: DEShapeFixMode = FixOrNot,
    fix_add_natural_bound_mode: DEShapeFixMode = FixOrNot,
    fix_missing_seam_mode: DEShapeFixMode = FixOrNot,
    fix_small_area_wire_mode: DEShapeFixMode = FixOrNot,
    remove_small_area_face_mode: DEShapeFixMode = FixOrNot,
    fix_intersecting_wires_mode: DEShapeFixMode = FixOrNot,
    fix_loop_wires_mode: DEShapeFixMode = FixOrNot,
    fix_split_face_mode: DEShapeFixMode = FixOrNot,
    auto_correct_precision_mode: DEShapeFixMode = Fix,
    modify_topology_mode: DEShapeFixMode = NotFix,
    modify_geometry_mode: DEShapeFixMode = Fix,
    closed_wire_mode: DEShapeFixMode = Fix,
    preference_pcurve_mode: DEShapeFixMode = Fix,
    fix_reorder_mode: DEShapeFixMode = FixOrNot,
    fix_small_mode: DEShapeFixMode = FixOrNot,
    fix_connected_mode: DEShapeFixMode = FixOrNot,
    fix_edge_curves_mode: DEShapeFixMode = FixOrNot,
    fix_degenerated_mode: DEShapeFixMode = FixOrNot,
    fix_lacking_mode: DEShapeFixMode = FixOrNot,
    fix_self_intersection_mode: DEShapeFixMode = FixOrNot,
    remove_loop_mode: DEShapeFixMode = FixOrNot,
    fix_reversed2d_mode: DEShapeFixMode = FixOrNot,
    fix_remove_pcurve_mode: DEShapeFixMode = FixOrNot,
    fix_remove_curve3d_mode: DEShapeFixMode = FixOrNot,
    fix_add_pcurve_mode: DEShapeFixMode = FixOrNot,
    fix_add_curve3d_mode: DEShapeFixMode = FixOrNot,
    fix_seam_mode: DEShapeFixMode = FixOrNot,
    fix_shifted_mode: DEShapeFixMode = FixOrNot,
    fix_edge_same_parameter_mode: DEShapeFixMode = NotFix,
    fix_notched_edges_mode: DEShapeFixMode = FixOrNot,
    fix_tail_mode: DEShapeFixMode = NotFix,
    max_tail_angle: DEShapeFixMode = NotFix,
    max_tail_width: DEShapeFixMode = FixOrNot,
    fix_self_intersecting_edge_mode: DEShapeFixMode = FixOrNot,
    fix_intersecting_edges_mode: DEShapeFixMode = FixOrNot,
    fix_non_adjacent_intersecting_edges_mode: DEShapeFixMode = FixOrNot,
    fix_vertex_position_mode: DEShapeFixMode = NotFix,
    fix_vertex_tolerance_mode: DEShapeFixMode = FixOrNot,
}

impl DEShapeFixParameters {
    /// The fix modes with their `FixShape.` parameter names, in the order of
    /// `XSAlgo_ShapeProcessor::FillParameterMap` (after the tolerances and levels).
    /// Native quirk kept: `MaxTailAngle` and `MaxTailWidth` are fix modes.
    pub(crate) fn fix_modes(&self) -> [(&'static str, DEShapeFixMode); 49] {
        [
            ("FixFreeShellMode", self.fix_free_shell_mode),
            ("FixFreeFaceMode", self.fix_free_face_mode),
            ("FixFreeWireMode", self.fix_free_wire_mode),
            ("FixSameParameterMode", self.fix_same_parameter_mode),
            ("FixSolidMode", self.fix_solid_mode),
            ("FixShellOrientationMode", self.fix_shell_orientation_mode),
            ("CreateOpenSolidMode", self.create_open_solid_mode),
            ("FixShellMode", self.fix_shell_mode),
            ("FixFaceOrientationMode", self.fix_face_orientation_mode),
            ("FixFaceMode", self.fix_face_mode),
            ("FixWireMode", self.fix_wire_mode),
            ("FixOrientationMode", self.fix_orientation_mode),
            ("FixAddNaturalBoundMode", self.fix_add_natural_bound_mode),
            ("FixMissingSeamMode", self.fix_missing_seam_mode),
            ("FixSmallAreaWireMode", self.fix_small_area_wire_mode),
            ("RemoveSmallAreaFaceMode", self.remove_small_area_face_mode),
            ("FixIntersectingWiresMode", self.fix_intersecting_wires_mode),
            ("FixLoopWiresMode", self.fix_loop_wires_mode),
            ("FixSplitFaceMode", self.fix_split_face_mode),
            ("AutoCorrectPrecisionMode", self.auto_correct_precision_mode),
            ("ModifyTopologyMode", self.modify_topology_mode),
            ("ModifyGeometryMode", self.modify_geometry_mode),
            ("ClosedWireMode", self.closed_wire_mode),
            ("PreferencePCurveMode", self.preference_pcurve_mode),
            ("FixReorderMode", self.fix_reorder_mode),
            ("FixSmallMode", self.fix_small_mode),
            ("FixConnectedMode", self.fix_connected_mode),
            ("FixEdgeCurvesMode", self.fix_edge_curves_mode),
            ("FixDegeneratedMode", self.fix_degenerated_mode),
            ("FixLackingMode", self.fix_lacking_mode),
            ("FixSelfIntersectionMode", self.fix_self_intersection_mode),
            ("RemoveLoopMode", self.remove_loop_mode),
            ("FixReversed2dMode", self.fix_reversed2d_mode),
            ("FixRemovePCurveMode", self.fix_remove_pcurve_mode),
            ("FixRemoveCurve3dMode", self.fix_remove_curve3d_mode),
            ("FixAddPCurveMode", self.fix_add_pcurve_mode),
            ("FixAddCurve3dMode", self.fix_add_curve3d_mode),
            ("FixSeamMode", self.fix_seam_mode),
            ("FixShiftedMode", self.fix_shifted_mode),
            (
                "FixEdgeSameParameterMode",
                self.fix_edge_same_parameter_mode,
            ),
            ("FixNotchedEdgesMode", self.fix_notched_edges_mode),
            ("FixTailMode", self.fix_tail_mode),
            ("MaxTailAngle", self.max_tail_angle),
            ("MaxTailWidth", self.max_tail_width),
            (
                "FixSelfIntersectingEdgeMode",
                self.fix_self_intersecting_edge_mode,
            ),
            ("FixIntersectingEdgesMode", self.fix_intersecting_edges_mode),
            (
                "FixNonAdjacentIntersectingEdgesMode",
                self.fix_non_adjacent_intersecting_edges_mode,
            ),
            ("FixVertexPositionMode", self.fix_vertex_position_mode),
            ("FixVertexToleranceMode", self.fix_vertex_tolerance_mode),
        ]
    }
}
