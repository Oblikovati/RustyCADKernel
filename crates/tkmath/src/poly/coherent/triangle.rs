// Rust translation of FoundationClasses/TKMath/Poly/Poly_CoherentTriangle.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Poly_CoherentTriangle`. Natively a triangle points at its neighbours and links; here
//! it stores their indices in the owning arrays, and the members that change two
//! triangles at once take the triangle array and the two indices. Out-of-range positions
//! (native undefined behaviour) panic on the bounds check.
use rustycad_tkernel::standard::Failure;
/// Three nodes, up to three neighbours with their opposite nodes, and three links;
/// `CoherentTriangle::new(0, 1, 2)`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoherentTriangle {
    pub(super) connections: i32,
    pub(super) nodes: [i32; 3],
    pub(super) nodes_on_connected: [i32; 3],
    pub(super) connected: [Option<usize>; 3],
    pub(super) links: [Option<usize>; 3],
}
impl Default for CoherentTriangle {
    fn default() -> Self {
        Self::new(-1, -1, -1)
    }
}
impl CoherentTriangle {
    /// A triangle on three nodes without neighbours or links; `CoherentTriangle::new(0, 1, 2)`.
    pub const fn new(node0: i32, node1: i32, node2: i32) -> Self {
        Self {
            connections: 0,
            nodes: [node0, node1, node2],
            nodes_on_connected: [-1; 3],
            connected: [None; 3],
            links: [None; 3],
        }
    }
    /// Node at position 0, 1 or 2; `triangle.node(0)`.
    pub const fn node(&self, ind: i32) -> i32 {
        self.nodes[ind as usize]
    }
    /// True when any node is negative (a removed triangle); `triangle.is_empty()`.
    pub const fn is_empty(&self) -> bool {
        self.nodes[0] < 0 || self.nodes[1] < 0 || self.nodes[2] < 0
    }
    /// Number of connected neighbours; `triangle.n_connections()`.
    pub const fn n_connections(&self) -> i32 {
        self.connections
    }
    /// Opposite node of the neighbour across side `conn`, -1 without one; `triangle.get_connected_node(0)`.
    pub const fn get_connected_node(&self, conn: i32) -> i32 {
        self.nodes_on_connected[conn as usize]
    }
    /// Index of the neighbour across side `conn`; `triangle.get_connected_tri(0)`.
    pub const fn get_connected_tri(&self, conn: i32) -> Option<usize> {
        self.connected[conn as usize]
    }
    /// Index of the link on side `link`, `None` when links are not built; `triangle.get_link(0)`.
    pub const fn get_link(&self, link: i32) -> Option<usize> {
        self.links[link as usize]
    }
    /// Side (0..2) on which `other` is connected, or -1; `triangle.find_connection(3)`.
    pub fn find_connection(&self, other: usize) -> i32 {
        self.connected
            .iter()
            .position(|connected| *connected == Some(other))
            .map_or(-1, |side| side as i32)
    }
    /// Disconnects side `conn` in both triangles, as `RemoveConnection(int)`. A neighbour
    /// that does not point back raises `Standard_ProgramError`;
    /// `CoherentTriangle::remove_connection(&mut triangles, 0, 1)?`.
    pub fn remove_connection(
        triangles: &mut [Self],
        this: usize,
        conn: i32,
    ) -> Result<(), Failure> {
        let Some(other) = triangles[this].connected[conn as usize] else {
            return Ok(());
        };
        let back = triangles[other].back_side(this)?;
        let neighbour = &mut triangles[other];
        neighbour.connected[back] = None;
        neighbour.nodes_on_connected[back] = -1;
        neighbour.connections -= 1;
        let own = &mut triangles[this];
        own.connected[conn as usize] = None;
        own.nodes_on_connected[conn as usize] = -1;
        own.connections -= 1;
        Ok(())
    }
    // The native search tests sides 1 and 2 only when side 0 does not point back.
    fn back_side(&self, this: usize) -> Result<usize, Failure> {
        if self.connected[0] == Some(this) {
            return Ok(0);
        }
        (1..3).find(|side| self.connected[*side] == Some(this)).ok_or_else(|| {
            Failure::with_type(
                "Standard_ProgramError",
                &format!("Poly_CoherentTriangle::RemoveConnection: wrong connection between triangles (triangle {this} is not a neighbour of its neighbour)"),
                "",
            )
        })
    }
    /// Disconnects `other` wherever it is connected, as `RemoveConnection(Poly_CoherentTriangle&)`;
    /// `CoherentTriangle::remove_connection_with(&mut triangles, 0, 1)?`.
    pub fn remove_connection_with(
        triangles: &mut [Self],
        this: usize,
        other: usize,
    ) -> Result<bool, Failure> {
        let side = triangles[this].find_connection(other);
        if side >= 0 {
            Self::remove_connection(triangles, this, side)?;
        }
        Ok(side >= 0)
    }
    // Records `other` across side `own_side` and `this` across `other_side` of `other`,
    // after dropping any previous connection on both sides.
    pub(super) fn link_pair(
        triangles: &mut [Self],
        (this, own_side): (usize, usize),
        (other, other_side): (usize, usize),
        opposite_on_other: i32,
    ) -> Result<(), Failure> {
        Self::remove_connection(triangles, this, own_side as i32)?;
        let own = &mut triangles[this];
        own.nodes_on_connected[own_side] = opposite_on_other;
        own.connected[own_side] = Some(other);
        own.connections += 1;
        Self::remove_connection(triangles, other, other_side as i32)?;
        let own_node = triangles[this].nodes[own_side];
        let neighbour = &mut triangles[other];
        neighbour.nodes_on_connected[other_side] = own_node;
        neighbour.connected[other_side] = Some(this);
        neighbour.connections += 1;
        Ok(())
    }
}
