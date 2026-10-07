// Rust translation of the pair choice of ShapeFix_Wireframe::MergeSmallEdges
// (ShapeFix_Wireframe.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871).

//! The choice of the pair of edges to join around a small middle edge: the tangent
//! angles at both ends, the faces shared with each neighbour and the angle limit.
use super::{MergeRun, WireScope};
use crate::shape_analysis::ShapeAnalysisEdge;
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Vector3;

/// The positions around the small edge: `index`, `prev` and `next`.
#[derive(Clone, Copy)]
pub(super) struct Positions {
    pub index: i32,
    pub prev: i32,
    pub next: i32,
}

/// The result of the choice: `edge1`/`edge2` (shifted to the middle and next edges when
/// `take_next` was set by a shared-face test), `edge3` (the next edge), `same_set`,
/// `isLimAngle` and `theList2` (the faces of the middle edge).
pub(super) struct Choice {
    pub edge1: Edge,
    pub edge2: Edge,
    pub edge3: Edge,
    pub take_next: bool,
    pub same_set: bool,
    pub lim_angle: bool,
    pub faces: List<Shape>,
}

impl MergeRun<'_, '_> {
    /// One step of the loop over the wire edges: the index of the next step.
    pub(super) fn merge_at(&mut self, index: i32, scope: &WireScope) -> Result<i32, Failure> {
        let data = self.wire_data()?;
        let count = data.nb_edges();
        let prev = if index == 1 { count } else { index - 1 };
        let next = if index == count { 1 } else { index + 1 };
        let at = Positions { index, prev, next };
        let edges = [data.edge(prev)?, data.edge(index)?, data.edge(next)?];
        // gka protection against joining seem edge
        if edges[1].is_same(&edges[0]) || edges[1].is_same(&edges[2]) {
            return Ok(index + 1);
        }
        let seams = (data.is_seam(index), data.is_seam(prev), data.is_seam(next));
        if !self.maps.small.contains(&edges[1]) {
            return Ok(index + 1);
        }
        let angles = angles(&edges[0], &edges[1], &edges[2])?;
        if self.maps.multiple.contains(&edges[0]) || self.maps.multiple.contains(&edges[1]) {
            return Ok(index + 1);
        }
        self.apply_choice(at, self.choose(edges, seams, angles), scope)
    }

    /// The join of the chosen pair, or the drop of the small edge.
    fn apply_choice(
        &mut self,
        at: Positions,
        choice: Choice,
        scope: &WireScope,
    ) -> Result<i32, Failure> {
        if choice.same_set && !choice.lim_angle {
            return self.join_pair(at, choice, scope);
        }
        if self.drop_mode {
            return self.drop_middle(at.index, &choice);
        }
        self.remove_closed(at.index, &choice)
    }

    /// The faces of the edge in `theEdgeToFaces` (empty when unbound).
    pub(super) fn faces_of(&self, edge: &Edge) -> List<Shape> {
        self.maps.edge_faces.find(edge).cloned().unwrap_or_default()
    }

    /// Whether the faces of a neighbour are those of the middle edge (`same_set1`,
    /// `same_set2`, gka protection against joining seem edges with other edges).
    fn same_set(middle: &List<Shape>, neighbour: &List<Shape>, seams: (bool, bool)) -> bool {
        let mut set = MapOfShape::default();
        middle.iter().for_each(|face| {
            set.add(face.clone());
        });
        neighbour.extent() == middle.extent()
            && seams.0 == seams.1
            && neighbour.iter().all(|f| set.contains(f))
    }

    /// The shared-face tests and the resulting pair.
    fn choose(
        &self,
        [edge1, edge2, edge3]: [Edge; 3],
        (seam, seam1, seam2): (bool, bool, bool),
        angles: (f64, f64),
    ) -> Choice {
        let faces = self.faces_of(&edge2);
        let same_set1 = Self::same_set(&faces, &self.faces_of(&edge1), (seam, seam1));
        let same_set2 = Self::same_set(&faces, &self.faces_of(&edge3), (seam, seam2));
        let take_next = edge1.is_same(&edge3);
        let mut choice = Choice {
            edge1,
            edge2,
            edge3,
            take_next,
            same_set: false,
            lim_angle: false,
            faces,
        };
        self.apply_same_sets(&mut choice, (same_set1, same_set2), angles);
        choice
    }

    fn apply_same_sets(&self, choice: &mut Choice, sets: (bool, bool), (ang1, ang2): (f64, f64)) {
        let exceeds = |angle: f64| self.limit_angle.is_some_and(|limit| angle > limit);
        choice.same_set = sets.0 || sets.1;
        match sets {
            (true, true) => {
                if (ang2 - ang1).abs() > precision::angular() {
                    choice.take_next = ang2 < ang1;
                }
                if choice.take_next {
                    shift(choice);
                }
                choice.lim_angle = exceeds(ang1.min(ang2));
            }
            (true, false) => choice.lim_angle = exceeds(ang1),
            (false, true) => {
                choice.lim_angle = exceeds(ang2);
                shift(choice);
                choice.take_next = true;
            }
            (false, false) => {}
        }
    }
}

fn shift(choice: &mut Choice) {
    choice.edge1 = choice.edge2.clone();
    choice.edge2 = choice.edge3.clone();
}

/// `Ang1` (between the first and the middle edge) and `Ang2` (between the middle and the
/// next edge); both 0 when an edge has no 3D curve.
fn angles(edge1: &Edge, edge2: &Edge, edge3: &Edge) -> Result<(f64, f64), Failure> {
    let sae = ShapeAnalysisEdge::new();
    let (c1, _, last1) = sae.curve_3d(edge1, true)?;
    if c1.is_null() {
        return Ok((0., 0.));
    }
    let (c2, first2, last2) = sae.curve_3d(edge2, true)?;
    if c2.is_null() {
        return Ok((0., 0.));
    }
    let (c3, first3, _) = sae.curve_3d(edge3, true)?;
    if c3.is_null() {
        return Ok((0., 0.));
    }
    let ang1 = angle(
        oriented(c1.d1(last1)?.1, edge1),
        oriented(c2.d1(first2)?.1, edge2),
    )?;
    let ang2 = angle(
        oriented(c2.d1(last2)?.1, edge2),
        oriented(c3.d1(first3)?.1, edge3),
    )?;
    Ok((ang1, ang2))
}

fn oriented(vector: Vector3, edge: &Edge) -> Vector3 {
    if edge.orientation() == Orientation::Reversed {
        -vector
    } else {
        vector
    }
}

fn angle(vec1: Vector3, vec2: Vector3) -> Result<f64, Failure> {
    let tol2 = precision::square_confusion();
    if vec1.square_magnitude() < tol2 || vec2.square_magnitude() < tol2 {
        return Ok(std::f64::consts::FRAC_PI_2);
    }
    Ok(vec1.angle(vec2)?.abs())
}
