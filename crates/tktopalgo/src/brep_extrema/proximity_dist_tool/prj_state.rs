// Rust translation of BRepExtrema_ProximityDistTool::ProxPnt_Status and
// BRepExtrema_ProximityDistTool::PrjState (BRepExtrema_ProximityDistTool.hxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The status of a proximity point and the projection state of a point on a triangle.
use rustycad_tkmath::bvh::tools::PrjStateInTriangle;

/// `BRepExtrema_ProximityDistTool::ProxPnt_Status`: a proximity point on the border of
/// its shape, in its middle, or unknown;
/// `assert_eq!(BRepExtremaProxPntStatus::Unknown as i32, 2)`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum BRepExtremaProxPntStatus {
    /// `ProxPnt_Status_BORDER`.
    Border = 0,
    /// `ProxPnt_Status_MIDDLE`.
    Middle = 1,
    /// `ProxPnt_Status_UNKNOWN`.
    #[default]
    Unknown = 2,
}

/// `BRepExtrema_ProximityDistTool::PrjState`: where the projection of a point lies on a
/// triangle of the second set (its index, a vertex, an edge or inside, and the nodes of
/// that vertex or edge, from 0).
///
/// ```
/// use rustycad_tkmath::bvh::tools::PrjStateInTriangle;
/// use rustycad_tktopalgo::brep_extrema::BRepExtremaPrjState;
/// let state = BRepExtremaPrjState::default();
/// assert_eq!(state.get_prj_state(), PrjStateInTriangle::Inner);
/// assert_eq!(state.get_trg_idx(), 0);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BRepExtremaPrjState {
    my_trg_idx: i32,
    my_prj_state: PrjStateInTriangle,
    my_number_of_first_node: i32,
    my_number_of_last_node: i32,
}

impl Default for BRepExtremaPrjState {
    /// `PrjState()`: triangle 0, inner, nodes 0.
    fn default() -> Self {
        Self::new(0, PrjStateInTriangle::Inner, 0, 0)
    }
}

impl BRepExtremaPrjState {
    /// `PrjState(theTrgIdx, thePrjState, theNumberOfFirstNode, theNumberOfLastNode)`;
    /// `BRepExtremaPrjState::new(3, PrjStateInTriangle::Edge, 0, 1)`.
    pub const fn new(
        trg_idx: i32,
        prj_state: PrjStateInTriangle,
        first_node: i32,
        last_node: i32,
    ) -> Self {
        Self {
            my_trg_idx: trg_idx,
            my_prj_state: prj_state,
            my_number_of_first_node: first_node,
            my_number_of_last_node: last_node,
        }
    }

    /// `GetTrgIdx()`: the triangle of the projection; `state.get_trg_idx()`.
    pub const fn get_trg_idx(&self) -> i32 {
        self.my_trg_idx
    }

    /// `GetPrjState()`: vertex, edge or inside; `state.get_prj_state()`.
    pub const fn get_prj_state(&self) -> PrjStateInTriangle {
        self.my_prj_state
    }

    /// `GetNumberOfFirstNode()`: the first node of the vertex or edge (-1 inside);
    /// `state.get_number_of_first_node()`.
    pub const fn get_number_of_first_node(&self) -> i32 {
        self.my_number_of_first_node
    }

    /// `GetNumberOfLastNode()`: the last node of the vertex or edge (-1 inside);
    /// `state.get_number_of_last_node()`.
    pub const fn get_number_of_last_node(&self) -> i32 {
        self.my_number_of_last_node
    }
}
