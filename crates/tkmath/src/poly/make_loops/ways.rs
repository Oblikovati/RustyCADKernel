// Rust translation of Poly_MakeLoops3D and Poly_MakeLoops2D in
// FoundationClasses/TKMath/Poly/Poly_MakeLoops.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The native `chooseLeftWay` overrides: the smallest signed turn from the incoming
//! tangent, measured around the surface normal (3D) or in the plane (2D).
use super::{Link, LinkTable, LoopWay, MakeLoops, MakeLoopsHelper, get_link};
use crate::gp::{Direction2, Direction3, Xyz};
use rustycad_tkernel::standard::Failure;
use std::f64::consts::PI;

fn first_candidate(candidates: &[i32]) -> Result<i32, Failure> {
    candidates
        .first()
        .copied()
        .ok_or_else(|| Failure::with_message("Standard_NoSuchObject", "NCollection_List::First"))
}
/// `Poly_MakeLoops3D::Helper`: tangents and normals; `None` when undefined.
pub trait MakeLoops3DHelper: MakeLoopsHelper {
    /// Tangent at the first node of a link; `helper.get_first_tangent(&link)`.
    fn get_first_tangent(&self, link: &Link) -> Option<Direction3>;
    /// Tangent at the last node of a link; `helper.get_last_tangent(&link)`.
    fn get_last_tangent(&self, link: &Link) -> Option<Direction3>;
    /// Surface normal at a node; `helper.get_normal(3)`.
    fn get_normal(&self, node: i32) -> Option<Direction3>;
}
/// `Poly_MakeLoops2D::Helper`: planar tangents; `None` when undefined.
pub trait MakeLoops2DHelper: MakeLoopsHelper {
    /// Tangent at the first node of a link; `helper.get_first_tangent(&link)`.
    fn get_first_tangent(&self, link: &Link) -> Option<Direction2>;
    /// Tangent at the last node of a link; `helper.get_last_tangent(&link)`.
    fn get_last_tangent(&self, link: &Link) -> Option<Direction2>;
}
/// The `Poly_MakeLoops3D` strategy; `MakeLoops::new(LeftWay3D::new(&helper))`.
pub struct LeftWay3D<'h> {
    helper: &'h dyn MakeLoops3DHelper,
}
impl<'h> LeftWay3D<'h> {
    /// Wraps a 3D helper; `LeftWay3D::new(&helper)`.
    pub fn new(helper: &'h dyn MakeLoops3DHelper) -> Self {
        Self { helper }
    }
}
/// `Poly_MakeLoops3D`; `MakeLoops3D::new(LeftWay3D::new(&helper))`.
pub type MakeLoops3D<'h> = MakeLoops<LeftWay3D<'h>>;
/// `Poly_MakeLoops2D`; `MakeLoops2D::new(LeftWay2D::new(true, &helper))`.
pub type MakeLoops2D<'h> = MakeLoops<LeftWay2D<'h>>;
// Tangent projected to the plane orthogonal to the normal, when long enough.
fn projected(normal: Direction3, tangent: Direction3) -> Option<Direction3> {
    let normal = Xyz::from(normal);
    let projection = normal.cross_crossed(Xyz::from(tangent), normal);
    if projection.square_modulus() < 1e-14 {
        return None;
    }
    Direction3::from_xyz(projection).ok()
}
// Keeps the candidate with the smallest angle; an angle near -PI counts as PI.
fn smallest_turn(
    candidates: &[i32],
    mut angle_of: impl FnMut(i32) -> Result<Option<f64>, Failure>,
) -> Result<i32, Failure> {
    let mut best = (PI * 2.0, 0);
    for &candidate in candidates {
        let Some(mut angle) = angle_of(candidate)? else {
            continue;
        };
        if angle < 1e-4 - PI {
            angle = PI;
        }
        if angle < best.0 {
            best = (angle, candidate);
        }
    }
    if best.1 == 0 {
        return first_candidate(candidates);
    }
    Ok(best.1)
}
fn turn_angle_3d(
    helper: &dyn MakeLoops3DHelper,
    links: &LinkTable,
    normal: Direction3,
    reference: Direction3,
    candidate: i32,
) -> Result<Option<f64>, Failure> {
    let candidate_link = get_link(links, candidate)?;
    let Some(tangent) = helper.get_first_tangent(&candidate_link) else {
        return Ok(None);
    };
    let Some(projected_tangent) = projected(normal, tangent) else {
        return Ok(None);
    };
    Ok(Some(projected_tangent.angle_with_ref(reference, normal)))
}
impl LoopWay for LeftWay3D<'_> {
    fn helper(&self) -> &dyn MakeLoopsHelper {
        self.helper
    }
    fn choose_left_way(
        &self,
        links: &LinkTable,
        node: i32,
        segment: i32,
        candidates: &[i32],
    ) -> Result<i32, Failure> {
        let Some(normal) = self.helper.get_normal(node) else {
            return first_candidate(candidates);
        };
        let reference = self
            .helper
            .get_last_tangent(&get_link(links, segment)?)
            .and_then(|tangent| projected(normal, tangent));
        let Some(reference) = reference else {
            return first_candidate(candidates);
        };
        smallest_turn(candidates, |candidate| {
            turn_angle_3d(self.helper, links, normal, reference, candidate)
        })
    }
}
/// The `Poly_MakeLoops2D` strategy, turning left or (when `left_way` is false) right;
/// `LeftWay2D::new(true, &helper)`.
pub struct LeftWay2D<'h> {
    right_way: bool,
    helper: &'h dyn MakeLoops2DHelper,
}
fn turn_angle_2d(
    helper: &dyn MakeLoops2DHelper,
    links: &LinkTable,
    reference: Direction2,
    right_way: bool,
    candidate: i32,
) -> Result<Option<f64>, Failure> {
    let candidate_link = get_link(links, candidate)?;
    let Some(tangent) = helper.get_first_tangent(&candidate_link) else {
        return Ok(None);
    };
    let angle = tangent.angle(reference);
    Ok(Some(if right_way { -angle } else { angle }))
}
impl<'h> LeftWay2D<'h> {
    /// Wraps a 2D helper; `LeftWay2D::new(true, &helper)`.
    pub fn new(left_way: bool, helper: &'h dyn MakeLoops2DHelper) -> Self {
        Self {
            right_way: !left_way,
            helper,
        }
    }
}
impl LoopWay for LeftWay2D<'_> {
    fn helper(&self) -> &dyn MakeLoopsHelper {
        self.helper
    }
    fn choose_left_way(
        &self,
        links: &LinkTable,
        _node: i32,
        segment: i32,
        candidates: &[i32],
    ) -> Result<i32, Failure> {
        let segment_link = get_link(links, segment)?;
        let Some(reference) = self.helper.get_last_tangent(&segment_link) else {
            return first_candidate(candidates);
        };
        smallest_turn(candidates, |candidate| {
            turn_angle_2d(self.helper, links, reference, self.right_way, candidate)
        })
    }
}
