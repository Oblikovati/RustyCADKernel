// Rust translation of ShapeExtend_WireData::ComputeSeams, IsSeam, Reverse(face) and the
// static SwapSeam of ShapeExtend_WireData.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The seam edges of a wire data: an edge present twice, FORWARD and REVERSED.
use super::{ShapeExtendWireData, WireDataFields, position};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;

impl WireDataFields {
    /// `ComputeSeams(enforce)` on the locked fields; see [`ShapeExtendWireData::compute_seams`].
    fn compute_seams(&mut self, enforce: bool) -> Result<(), Failure> {
        if self.seam_f >= 0 && !enforce {
            return Ok(());
        }
        let mut seams = Sequence::default();
        self.seam_f = 0;
        self.seam_r = 0;
        let (reversed_map, rank_of) = self.map_reversed_edges()?;
        for (i, edge) in self.edges.iter().enumerate() {
            if edge.orientation() == Orientation::Reversed {
                continue;
            }
            let num = reversed_map.find_index(edge);
            if num == 0 {
                continue;
            }
            let rank = i as i32 + 1;
            if self.seam_f == 0 {
                self.seam_f = rank;
                self.seam_r = rank_of[num];
            } else {
                seams.append(rank);
                seams.append(rank_of[num]);
            }
        }
        self.seams_cache.clear();
        for &rank in seams.iter() {
            self.seams_cache.add(rank);
        }
        self.seams = Some(seams);
        Ok(())
    }

    /// The first pass of `ComputeSeams`: the REVERSED edges in a map (`TopTools_ShapeMapHasher`
    /// identity) and, for each map index, the rank of the last REVERSED edge mapped to it
    /// (`SE[num] = i`).
    fn map_reversed_edges(&self) -> Result<(IndexedMapOfShape, Vec<i32>), Failure> {
        let mut map = IndexedMapOfShape::default();
        let mut rank_of = vec![0; self.edges.len() + 1];
        for (i, edge) in self.edges.iter().enumerate() {
            if edge.orientation() == Orientation::Reversed {
                let num = map.add(edge.clone());
                rank_of[num] = i as i32 + 1;
            }
        }
        Ok((map, rank_of))
    }

    /// `IsSeam(num)` on the locked fields.
    fn is_seam(&mut self, num: i32) -> Result<bool, Failure> {
        if self.seam_f < 0 {
            self.compute_seams(true)?;
        }
        if self.seam_f == 0 {
            return Ok(false);
        }
        if num == self.seam_f || num == self.seam_r {
            return Ok(true);
        }
        Ok(self.seams_cache.contains(num))
    }
}

impl ShapeExtendWireData {
    /// `ComputeSeams(enforce)`: finds the seams (an edge FORWARD whose REVERSED twin is in
    /// the list): the first pair is kept apart, the others in the seam list; without
    /// `enforce`, nothing when the seams are known; `data.compute_seams(true)?`.
    pub fn compute_seams(&self, enforce: bool) -> Result<(), Failure> {
        self.fields.write().compute_seams(enforce)
    }

    /// `IsSeam(num)`: whether the edge of rank `num` is one of a seam pair (the seams are
    /// computed first when needed); `data.is_seam(1)`.
    pub fn is_seam(&self, num: i32) -> bool {
        // ComputeSeams cannot fail on the edges of the list: they all are edges.
        self.fields.write().is_seam(num).unwrap_or(false)
    }

    /// `Reverse(face)`: [`ShapeExtendWireData::reverse`], then, for a non-null face, the
    /// two p-curves of every seam edge on the face are swapped (a seam reversed changes
    /// its role); `data.reverse_on_face(&face)?`.
    pub fn reverse_on_face(&self, face: &Face) -> Result<(), Failure> {
        self.reverse()?;
        if face.is_null() {
            return Ok(());
        }
        let ranks = self.seam_ranks()?;
        for rank in ranks {
            let edge = {
                let fields = self.fields.read();
                let index = position(rank, "ShapeExtend_WireData::Reverse")?;
                fields.edges.value(index)?.clone()
            };
            swap_seam(&edge, face)?;
        }
        self.fields.write().seam_f = -1;
        Ok(())
    }

    /// The ranks `Reverse(face)` visits after `ComputeSeams(true)`: the first pair (when
    /// found), then the seam list.
    fn seam_ranks(&self) -> Result<Vec<i32>, Failure> {
        let mut fields = self.fields.write();
        fields.compute_seams(true)?;
        let mut ranks: Vec<i32> = [fields.seam_f, fields.seam_r]
            .into_iter()
            .filter(|&rank| rank > 0)
            .collect();
        if let Some(seams) = &fields.seams {
            ranks.extend(seams.iter().copied());
        }
        Ok(ranks)
    }
}

/// The static `SwapSeam(S, F)`: for a FORWARD (not REVERSED) seam edge with both p-curves
/// on the face (taken FORWARD), stores them swapped and sets the range on the face to
/// the one of the former first p-curve.
fn swap_seam(shape: &Shape, face: &Face) -> Result<(), Failure> {
    let mut edge = Edge::unchecked(shape.clone());
    if edge.is_null() || face.is_null() || edge.orientation() == Orientation::Reversed {
        return Ok(());
    }
    let the_face = Face::unchecked(face.oriented(Orientation::Forward));
    let forward = BRepTool::curve_on_face(&edge, &the_face)?;
    edge.set_orientation(Orientation::Reversed);
    let reversed = BRepTool::curve_on_face(&edge, &the_face)?;
    if forward.pcurve.is_null() || reversed.pcurve.is_null() {
        return Ok(());
    }
    edge.set_orientation(Orientation::Forward);
    let builder = BRepBuilder::default();
    builder.update_edge_pcurves_on_face(
        &edge,
        (&reversed.pcurve, &forward.pcurve),
        &the_face,
        0.0,
    )?;
    builder.range_on_face(&edge, &the_face, forward.first, forward.last)
}
