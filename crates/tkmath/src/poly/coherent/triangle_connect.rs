// Rust translation of the SetConnection members of FoundationClasses/TKMath/Poly/Poly_CoherentTriangle.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Connections exist only between coherently oriented triangles: the shared side runs in
//! opposite directions in the two triangles.
use super::CoherentTriangle;
use rustycad_tkernel::standard::Failure;
/// One native branch: this side, the other's side and the other's node reported opposite.
type Pairing = (usize, usize, usize);
impl CoherentTriangle {
    /// Connects `other` across side `conn` of `this` when the side matches; returns whether it
    /// did; `CoherentTriangle::set_connection(&mut triangles, 0, 1, 1)?`.
    pub fn set_connection(
        triangles: &mut [Self],
        this: usize,
        conn: i32,
        other: usize,
    ) -> Result<bool, Failure> {
        const II: [usize; 5] = [2, 0, 1, 2, 0];
        let side = conn as usize;
        let (start, end) = (
            triangles[this].nodes[II[side + 2]],
            triangles[this].nodes[II[side]],
        );
        let theirs = triangles[other].nodes;
        // The first node of `other` equal to `start` decides; `end` must then precede it.
        let Some(at) = (0..3).find(|k| theirs[*k] == start) else {
            return Ok(false);
        };
        let before = (at + 2) % 3;
        if theirs[before] != end {
            return Ok(false);
        }
        let other_side = (at + 1) % 3;
        Self::link_pair(
            triangles,
            (this, side),
            (other, other_side),
            theirs[other_side],
        )?;
        Ok(true)
    }
    /// Finds the shared side of two triangles and connects them, unless they are already
    /// connected on it; `CoherentTriangle::set_connection_any(&mut triangles, 0, 1)?`.
    pub fn set_connection_any(
        triangles: &mut [Self],
        this: usize,
        other: usize,
    ) -> Result<bool, Failure> {
        let Some((own, theirs, opposite)) = Self::shared_side(triangles, this, other) else {
            return Ok(false);
        };
        let opposite_node = triangles[other].nodes[opposite];
        Self::link_pair(triangles, (this, own), (other, theirs), opposite_node)?;
        Ok(true)
    }
    // The native decision tree of SetConnection(Poly_CoherentTriangle&): the first node of
    // `other` equal to this triangle's node 0 selects two candidate sides, otherwise side 0
    // is tried against the three rotations of `other`.
    fn shared_side(triangles: &[Self], this: usize, other: usize) -> Option<Pairing> {
        let mine = triangles[this].nodes;
        let t = triangles[other].nodes;
        let free = |side: usize| triangles[this].connected[side] != Some(other);
        if let Some(at) = (0..3).find(|k| mine[0] == t[*k]) {
            let (next, previous) = ((at + 1) % 3, (at + 2) % 3);
            let first = (mine[1] == t[previous] && free(2)).then_some((2, next, next));
            let second = (mine[2] == t[next] && free(1)).then_some((1, previous, previous));
            return first.or(second);
        }
        if !free(0) {
            return None;
        }
        [0, 2, 1].into_iter().find_map(|k| {
            let side = (k + 1) % 3;
            (mine[1] == t[k] && mine[2] == t[(k + 2) % 3]).then_some((0, side, side))
        })
    }
}
