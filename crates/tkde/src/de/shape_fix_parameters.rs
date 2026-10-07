// Rust port of DataExchange/TKDE/DE/DE_ShapeFixParameters.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DE_ShapeFixParameters`: the shape-healing settings a format node passes to ShapeFix.
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `DE_ShapeFixParameters::FixMode`: whether a healing procedure runs.
///
/// The native enum class has an `int` underlying type and `DE_ShapeFixConfigurationNode::Load`
/// casts any resource integer to it, so the port is a newtype over `i32` that keeps every
/// value; the three named values are the constants (de-design.md section 5).
///
/// ```
/// use rustycad_tkde::de::DeShapeFixParametersFixMode;
/// assert_eq!(DeShapeFixParametersFixMode::FIX_OR_NOT.0, -1);
/// assert_eq!(DeShapeFixParametersFixMode(5).0, 5);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DeShapeFixParametersFixMode(pub i32);
impl DeShapeFixParametersFixMode {
    /// `FixOrNot`: the procedure runs or not, depending on the situation.
    pub const FIX_OR_NOT: Self = Self(-1);
    /// `NotFix`: the procedure does not run.
    pub const NOT_FIX: Self = Self(0);
    /// `Fix`: the procedure runs anyway.
    pub const FIX: Self = Self(1);
}

/// `DE_ShapeFixParameters`, with the native default of every field.
///
/// ```
/// use rustycad_tkde::de::{DeShapeFixParameters, DeShapeFixParametersFixMode};
/// let parameters = DeShapeFixParameters::default();
/// assert_eq!(parameters.tolerance3d, 1.0e-6);
/// assert_eq!(parameters.create_open_solid_mode, DeShapeFixParametersFixMode::NOT_FIX);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeShapeFixParameters {
    /// `Tolerance3d`.
    pub tolerance3d: f64,
    /// `MaxTolerance3d`.
    pub max_tolerance3d: f64,
    /// `MinTolerance3d`.
    pub min_tolerance3d: f64,
    /// `DetalizationLevel`.
    pub detalization_level: ShapeEnum,
    /// `NonManifold`.
    pub non_manifold: bool,
    /// `FixFreeShellMode`.
    pub fix_free_shell_mode: DeShapeFixParametersFixMode,
    /// `FixFreeFaceMode`.
    pub fix_free_face_mode: DeShapeFixParametersFixMode,
    /// `FixFreeWireMode`.
    pub fix_free_wire_mode: DeShapeFixParametersFixMode,
    /// `FixSameParameterMode`.
    pub fix_same_parameter_mode: DeShapeFixParametersFixMode,
    /// `FixSolidMode`.
    pub fix_solid_mode: DeShapeFixParametersFixMode,
    /// `FixShellOrientationMode`.
    pub fix_shell_orientation_mode: DeShapeFixParametersFixMode,
    /// `CreateOpenSolidMode`.
    pub create_open_solid_mode: DeShapeFixParametersFixMode,
    /// `FixShellMode`.
    pub fix_shell_mode: DeShapeFixParametersFixMode,
    /// `FixFaceOrientationMode`.
    pub fix_face_orientation_mode: DeShapeFixParametersFixMode,
    /// `FixFaceMode`.
    pub fix_face_mode: DeShapeFixParametersFixMode,
    /// `FixWireMode`.
    pub fix_wire_mode: DeShapeFixParametersFixMode,
    /// `FixOrientationMode`.
    pub fix_orientation_mode: DeShapeFixParametersFixMode,
    /// `FixAddNaturalBoundMode`.
    pub fix_add_natural_bound_mode: DeShapeFixParametersFixMode,
    /// `FixMissingSeamMode`.
    pub fix_missing_seam_mode: DeShapeFixParametersFixMode,
    /// `FixSmallAreaWireMode`.
    pub fix_small_area_wire_mode: DeShapeFixParametersFixMode,
    /// `RemoveSmallAreaFaceMode`.
    pub remove_small_area_face_mode: DeShapeFixParametersFixMode,
    /// `FixIntersectingWiresMode`.
    pub fix_intersecting_wires_mode: DeShapeFixParametersFixMode,
    /// `FixLoopWiresMode`.
    pub fix_loop_wires_mode: DeShapeFixParametersFixMode,
    /// `FixSplitFaceMode`.
    pub fix_split_face_mode: DeShapeFixParametersFixMode,
    /// `AutoCorrectPrecisionMode`.
    pub auto_correct_precision_mode: DeShapeFixParametersFixMode,
    /// `ModifyTopologyMode`.
    pub modify_topology_mode: DeShapeFixParametersFixMode,
    /// `ModifyGeometryMode`.
    pub modify_geometry_mode: DeShapeFixParametersFixMode,
    /// `ClosedWireMode`.
    pub closed_wire_mode: DeShapeFixParametersFixMode,
    /// `PreferencePCurveMode`.
    pub preference_pcurve_mode: DeShapeFixParametersFixMode,
    /// `FixReorderMode`.
    pub fix_reorder_mode: DeShapeFixParametersFixMode,
    /// `FixSmallMode`.
    pub fix_small_mode: DeShapeFixParametersFixMode,
    /// `FixConnectedMode`.
    pub fix_connected_mode: DeShapeFixParametersFixMode,
    /// `FixEdgeCurvesMode`.
    pub fix_edge_curves_mode: DeShapeFixParametersFixMode,
    /// `FixDegeneratedMode`.
    pub fix_degenerated_mode: DeShapeFixParametersFixMode,
    /// `FixLackingMode`.
    pub fix_lacking_mode: DeShapeFixParametersFixMode,
    /// `FixSelfIntersectionMode`.
    pub fix_self_intersection_mode: DeShapeFixParametersFixMode,
    /// `RemoveLoopMode`.
    pub remove_loop_mode: DeShapeFixParametersFixMode,
    /// `FixReversed2dMode`.
    pub fix_reversed2d_mode: DeShapeFixParametersFixMode,
    /// `FixRemovePCurveMode`.
    pub fix_remove_pcurve_mode: DeShapeFixParametersFixMode,
    /// `FixRemoveCurve3dMode`.
    pub fix_remove_curve3d_mode: DeShapeFixParametersFixMode,
    /// `FixAddPCurveMode`.
    pub fix_add_pcurve_mode: DeShapeFixParametersFixMode,
    /// `FixAddCurve3dMode`.
    pub fix_add_curve3d_mode: DeShapeFixParametersFixMode,
    /// `FixSeamMode`.
    pub fix_seam_mode: DeShapeFixParametersFixMode,
    /// `FixShiftedMode`.
    pub fix_shifted_mode: DeShapeFixParametersFixMode,
    /// `FixEdgeSameParameterMode`.
    pub fix_edge_same_parameter_mode: DeShapeFixParametersFixMode,
    /// `FixNotchedEdgesMode`.
    pub fix_notched_edges_mode: DeShapeFixParametersFixMode,
    /// `FixTailMode`.
    pub fix_tail_mode: DeShapeFixParametersFixMode,
    /// `MaxTailAngle` (a mode, as natively, not an angle).
    pub max_tail_angle: DeShapeFixParametersFixMode,
    /// `MaxTailWidth` (a mode, as natively, not a width).
    pub max_tail_width: DeShapeFixParametersFixMode,
    /// `FixSelfIntersectingEdgeMode`.
    pub fix_self_intersecting_edge_mode: DeShapeFixParametersFixMode,
    /// `FixIntersectingEdgesMode`.
    pub fix_intersecting_edges_mode: DeShapeFixParametersFixMode,
    /// `FixNonAdjacentIntersectingEdgesMode`.
    pub fix_non_adjacent_intersecting_edges_mode: DeShapeFixParametersFixMode,
    /// `FixVertexPositionMode`.
    pub fix_vertex_position_mode: DeShapeFixParametersFixMode,
    /// `FixVertexToleranceMode`.
    pub fix_vertex_tolerance_mode: DeShapeFixParametersFixMode,
}

impl Default for DeShapeFixParameters {
    fn default() -> Self {
        const FIX_OR_NOT: DeShapeFixParametersFixMode = DeShapeFixParametersFixMode::FIX_OR_NOT;
        const NOT_FIX: DeShapeFixParametersFixMode = DeShapeFixParametersFixMode::NOT_FIX;
        const FIX: DeShapeFixParametersFixMode = DeShapeFixParametersFixMode::FIX;
        Self {
            tolerance3d: 1.0e-6,
            max_tolerance3d: 1.0,
            min_tolerance3d: 1.0e-7,
            detalization_level: ShapeEnum::Vertex,
            non_manifold: false,
            fix_free_shell_mode: FIX_OR_NOT,
            fix_free_face_mode: FIX_OR_NOT,
            fix_free_wire_mode: FIX_OR_NOT,
            fix_same_parameter_mode: FIX_OR_NOT,
            fix_solid_mode: FIX_OR_NOT,
            fix_shell_orientation_mode: FIX_OR_NOT,
            create_open_solid_mode: NOT_FIX,
            fix_shell_mode: FIX_OR_NOT,
            fix_face_orientation_mode: FIX_OR_NOT,
            fix_face_mode: FIX_OR_NOT,
            fix_wire_mode: FIX_OR_NOT,
            fix_orientation_mode: FIX_OR_NOT,
            fix_add_natural_bound_mode: FIX_OR_NOT,
            fix_missing_seam_mode: FIX_OR_NOT,
            fix_small_area_wire_mode: FIX_OR_NOT,
            remove_small_area_face_mode: FIX_OR_NOT,
            fix_intersecting_wires_mode: FIX_OR_NOT,
            fix_loop_wires_mode: FIX_OR_NOT,
            fix_split_face_mode: FIX_OR_NOT,
            auto_correct_precision_mode: FIX,
            modify_topology_mode: NOT_FIX,
            modify_geometry_mode: FIX,
            closed_wire_mode: FIX,
            preference_pcurve_mode: FIX,
            fix_reorder_mode: FIX_OR_NOT,
            fix_small_mode: FIX_OR_NOT,
            fix_connected_mode: FIX_OR_NOT,
            fix_edge_curves_mode: FIX_OR_NOT,
            fix_degenerated_mode: FIX_OR_NOT,
            fix_lacking_mode: FIX_OR_NOT,
            fix_self_intersection_mode: FIX_OR_NOT,
            remove_loop_mode: FIX_OR_NOT,
            fix_reversed2d_mode: FIX_OR_NOT,
            fix_remove_pcurve_mode: FIX_OR_NOT,
            fix_remove_curve3d_mode: FIX_OR_NOT,
            fix_add_pcurve_mode: FIX_OR_NOT,
            fix_add_curve3d_mode: FIX_OR_NOT,
            fix_seam_mode: FIX_OR_NOT,
            fix_shifted_mode: FIX_OR_NOT,
            fix_edge_same_parameter_mode: NOT_FIX,
            fix_notched_edges_mode: FIX_OR_NOT,
            fix_tail_mode: NOT_FIX,
            max_tail_angle: NOT_FIX,
            max_tail_width: FIX_OR_NOT,
            fix_self_intersecting_edge_mode: FIX_OR_NOT,
            fix_intersecting_edges_mode: FIX_OR_NOT,
            fix_non_adjacent_intersecting_edges_mode: FIX_OR_NOT,
            fix_vertex_position_mode: NOT_FIX,
            fix_vertex_tolerance_mode: FIX_OR_NOT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_the_native_member_initializers() {
        let parameters = DeShapeFixParameters::default();
        assert_eq!(
            (parameters.max_tolerance3d, parameters.min_tolerance3d),
            (1.0, 1.0e-7)
        );
        assert_eq!(parameters.detalization_level, ShapeEnum::Vertex);
        assert!(!parameters.non_manifold);
        assert_eq!(
            parameters.auto_correct_precision_mode,
            DeShapeFixParametersFixMode::FIX
        );
        assert_eq!(
            parameters.fix_tail_mode,
            DeShapeFixParametersFixMode::NOT_FIX
        );
        assert_eq!(
            parameters.max_tail_width,
            DeShapeFixParametersFixMode::FIX_OR_NOT
        );
    }

    #[test]
    fn fix_mode_keeps_values_outside_the_named_ones() {
        assert_eq!(
            DeShapeFixParametersFixMode::NOT_FIX,
            DeShapeFixParametersFixMode(0)
        );
        assert_ne!(
            DeShapeFixParametersFixMode(7),
            DeShapeFixParametersFixMode::FIX
        );
    }
}
