// Rust port of the resource keys and comments of DataExchange/TKDE/DE/
// DE_ShapeFixConfigurationNode.cxx (Load and Save) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The keys of the shape-healing resources, in the native Load and Save order, with the
//! comment Save writes before each value (native typos and misplaced `!` kept).
use super::{DeShapeFixParameters, DeShapeFixParametersFixMode};

/// The tail of every mode comment.
pub(super) const AVAILABLE_MODES: &str =
    "Available values: \"FixOrNot\"(-1), \"NotFix\"(0), \"Fix\"(1)\n";

/// A tolerance resource: its key, its comment and its field.
pub(super) struct ToleranceKey {
    /// The key after `provider.FORMAT.VENDOR.healing.`.
    pub(super) key: &'static str,
    /// The comment lines before the value.
    pub(super) comment: &'static str,
    /// The field the value is read into and written from.
    pub(super) field: fn(&mut DeShapeFixParameters) -> &mut f64,
}

/// A mode resource: its key, the first comment lines, the default named in the comment and
/// its field.
pub(super) struct FixModeKey {
    /// The key after `provider.FORMAT.VENDOR.healing.`.
    pub(super) key: &'static str,
    /// The comment lines before the `!Default value:` line.
    pub(super) description: &'static str,
    /// The default the comment names (`"FixOrNot"(-1)`).
    pub(super) default_label: &'static str,
    /// The field the value is read into and written from.
    pub(super) field: fn(&mut DeShapeFixParameters) -> &mut DeShapeFixParametersFixMode,
}

/// The three tolerances, first in Load and Save.
pub(super) const TOLERANCE_KEYS: [ToleranceKey; 3] = [
    ToleranceKey {
        key: "tolerance3d",
        comment: "!Defines the maximum allowable tolerance\n!Default value: 1.e-6. Available values: any real positive (non null) value\n",
        field: |parameters| &mut parameters.tolerance3d,
    },
    ToleranceKey {
        key: "max.tolerance3d",
        comment: "!Defines the maximum allowable tolerance\n!Default value: 1.0. Available values: any real positive (non null) value\n",
        field: |parameters| &mut parameters.max_tolerance3d,
    },
    ToleranceKey {
        key: "min.tolerance3d",
        comment: "!Defines the minimum allowable tolerance\n!Default value: 1.e-7. Available values: any real positive (non null) value\n",
        field: |parameters| &mut parameters.min_tolerance3d,
    },
];

/// The modes, after the tolerances, in the native order.
pub(super) const FIX_MODE_KEYS: [FixModeKey; 49] = [
    FixModeKey {
        key: "free.shell",
        description: "!Defines the mode for applying fixes of ShapeFix_Shell for ShapeFix_Shape\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_free_shell_mode,
    },
    FixModeKey {
        key: "free.face",
        description: "!Defines the mode for applying fixes of ShapeFix_Face for ShapeFix_Shape\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_free_face_mode,
    },
    FixModeKey {
        key: "free.wire",
        description: "!Defines the mode for applying fixes of ShapeFix_Wire for ShapeFix_Shape\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_free_wire_mode,
    },
    FixModeKey {
        key: "same.parameter",
        description: "!Defines the mode for applying ShapeFix::SameParameter after all fixes\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_same_parameter_mode,
    },
    FixModeKey {
        key: "solid",
        description: "!Defines the mode for applying fixes of ShapeFix_Solid\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_solid_mode,
    },
    FixModeKey {
        key: "shell.orientation",
        description: "!Defines the mode for applying analysis and fixes of orientation of shells in the solid\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_shell_orientation_mode,
    },
    FixModeKey {
        key: "create.open.solid",
        description: "!Defines the mode for creation of solids. If operation is executed then solids are created from open shells else solids are created from closed shells only\n",
        default_label: "\"NotFix\"(0)",
        field: |parameters| &mut parameters.create_open_solid_mode,
    },
    FixModeKey {
        key: "shell",
        description: "!Defines the mode for applying fixes of ShapeFix_Shell for ShapeFix_Solid\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_shell_mode,
    },
    FixModeKey {
        key: "face.orientation",
        description: "!Defines the mode for applying analysis and fixes of orientation of faces in the shell\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_face_orientation_mode,
    },
    FixModeKey {
        key: "face",
        description: "!Defines the mode for applying fixes of ShapeFix_Face\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_face_mode,
    },
    FixModeKey {
        key: "wire",
        description: "!Defines the mode for applying fixes of ShapeFix_Wire\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_wire_mode,
    },
    FixModeKey {
        key: "orientation",
        description: "!Defines the mode for applying a fix for the orientation of faces in the shell\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_orientation_mode,
    },
    FixModeKey {
        key: "add.natural.bound",
        description: "!Defines the add natural bound mode. If operation is executed then natural boundary is added on faces that miss them\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_add_natural_bound_mode,
    },
    FixModeKey {
        key: "missing.seam",
        description: "!Defines the fix missing seam mode (tries to insert seam is missed)\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_missing_seam_mode,
    },
    FixModeKey {
        key: "small.area.wire",
        description: "!Defines the fix small area wire mode (drops small wires)\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_small_area_wire_mode,
    },
    FixModeKey {
        key: "remove.small.area.face",
        description: "!Defines the remove face with small area (drops faces with small outer wires)\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.remove_small_area_face_mode,
    },
    FixModeKey {
        key: "intersecting.wires",
        description: "!Defines the fix intersecting wires mode in ShapeFix_Face\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_intersecting_wires_mode,
    },
    FixModeKey {
        key: "loop.wires",
        description: "!Defines the fix loop wires mode in ShapeFix_Face\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_loop_wires_mode,
    },
    FixModeKey {
        key: "split.face",
        description: "!Defines the fix split face mode in ShapeFix_Face\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_split_face_mode,
    },
    FixModeKey {
        key: "auto.correct.precision",
        description: "!Defines the auto-correct precision mode in ShapeFix_Face\n",
        default_label: "\"Fix\"(1)",
        field: |parameters| &mut parameters.auto_correct_precision_mode,
    },
    FixModeKey {
        key: "modify.topology",
        description: "!Defines the mode allowed to modify topology of the wire during fixing (adding/removing edges etc.)\n",
        default_label: "\"NotFix\"(0)",
        field: |parameters| &mut parameters.modify_topology_mode,
    },
    FixModeKey {
        key: "modify.geometry",
        description: "!Defines the mode allowed to modify geometry of the edges and vertices\n",
        default_label: "\"Fix\"(1)",
        field: |parameters| &mut parameters.modify_geometry_mode,
    },
    FixModeKey {
        key: "closed.wire",
        description: "!Defines the mode allowed to defines whether the wire is to be closed (by calling methods like FixDegenerated() !and FixConnected() for lastand first edges\n",
        default_label: "\"Fix\"(1)",
        field: |parameters| &mut parameters.closed_wire_mode,
    },
    FixModeKey {
        key: "preference.pcurve",
        description: "!Defines the mode allowed to defines the 2d representation of the wire is preferable over 3d one (in the case of ambiguity in FixEdgeCurves)\n!and FixConnected() for lastand first edges\n",
        default_label: "\"Fix\"(1)",
        field: |parameters| &mut parameters.preference_pcurve_mode,
    },
    FixModeKey {
        key: "reorder.edges",
        description: "!Defines the mode allowed to reorder edges in the wire\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_reorder_mode,
    },
    FixModeKey {
        key: "remove.small.edges",
        description: "!Defines the mode allowed to remove small edges\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_small_mode,
    },
    FixModeKey {
        key: "connected.edges",
        description: "!Defines the mode for fix connecting edges in the wire\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_connected_mode,
    },
    FixModeKey {
        key: "edge.curves",
        description: "!Defines the mode for fix edges (3Dcurves and 2D curves)\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_edge_curves_mode,
    },
    FixModeKey {
        key: "add.degenerated.edges",
        description: "!Defines the mode for add degenerated edges\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_degenerated_mode,
    },
    FixModeKey {
        key: "add.lacking.edges",
        description: "!Defines the mode for add lacking edges\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_lacking_mode,
    },
    FixModeKey {
        key: "selfintersection",
        description: "!Defines the mode for fix selfintersection edges\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_self_intersection_mode,
    },
    FixModeKey {
        key: "remove.loop",
        description: "!Defines the mode allowed to remove loop\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.remove_loop_mode,
    },
    FixModeKey {
        key: "reversed2d",
        description: "!Defines the mode allowed to fix edge if pcurve is directed opposite to 3d curve\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_reversed2d_mode,
    },
    FixModeKey {
        key: "remove.pcurve",
        description: "!Defines the mode allowed to remove the pcurve(s) of the edge if it does not match the vertices\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_remove_pcurve_mode,
    },
    FixModeKey {
        key: "remove.curve3d",
        description: "!Defines the mode allowed to remove 3d curve of the edge if it does not match the vertices\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_remove_curve3d_mode,
    },
    FixModeKey {
        key: "add.pcurve",
        description: "!Defines the mode allowed to add pcurve(s) of the edge if missing (by projecting 3d curve)\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_add_pcurve_mode,
    },
    FixModeKey {
        key: "add.curve3d",
        description: "!Defines the mode allowed to build 3d curve of the edge if missing\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_add_curve3d_mode,
    },
    FixModeKey {
        key: "correct.order.in.seam",
        description: "!Defines the mode allowed to correct order of pcurves in the seam edge depends on its orientation\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_seam_mode,
    },
    FixModeKey {
        key: "shifted",
        description: "!Defines the mode allowed to shifts wrong 2D curves back, ensuring that the 2D curves of the edges in the wire are connected\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_shifted_mode,
    },
    FixModeKey {
        key: "edge.same.parameter",
        description: "!Defines the mode for applying EdgeSameParameter\n",
        default_label: "\"NotFix\"(0)",
        field: |parameters| &mut parameters.fix_edge_same_parameter_mode,
    },
    FixModeKey {
        key: "notched.edges",
        description: "!Defines the mode for fix notched edges\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_notched_edges_mode,
    },
    FixModeKey {
        key: "tail",
        description: "!Defines the mode for fix tail in wire\n",
        default_label: "\"NotFix\"(0)",
        field: |parameters| &mut parameters.fix_tail_mode,
    },
    FixModeKey {
        key: "max.tail.angle",
        description: "!Defines the mode for max angle of the tails\n",
        default_label: "\"NotFix\"(0)",
        field: |parameters| &mut parameters.max_tail_angle,
    },
    FixModeKey {
        key: "max.tail.width",
        description: "!Defines the mode for max tail width\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.max_tail_width,
    },
    FixModeKey {
        key: "selfintersecting.edge",
        description: "!Defines the mode for fix selfintersecting of edge\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_self_intersecting_edge_mode,
    },
    FixModeKey {
        key: "intersecting.edges",
        description: "!Defines the mode for fix intersecting edges\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_intersecting_edges_mode,
    },
    FixModeKey {
        key: "nonadjacent.intersecting.edges",
        description: "!Defines the mode for fix non adjacent intersecting edges\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_non_adjacent_intersecting_edges_mode,
    },
    FixModeKey {
        key: "vertex.position",
        description: "!Defines the mode for applying ShapeFix::FixVertexPosition before all fixes\n",
        default_label: "\"NotFix\"(0)",
        field: |parameters| &mut parameters.fix_vertex_position_mode,
    },
    FixModeKey {
        key: "vertex.tolerance",
        description: "!Defines the mode for increases the tolerances of the edge vertices to comprise !the ends of 3d curve and pcurve on the given face (first method) or all pcurves stored in an edge (second one)\n",
        default_label: "\"FixOrNot\"(-1)",
        field: |parameters| &mut parameters.fix_vertex_tolerance_mode,
    },
];
