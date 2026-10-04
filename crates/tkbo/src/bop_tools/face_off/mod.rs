use super::{CoupleOfShape, algo_tools2d};
use crate::int_tools::Context;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Edge, Face},
};
use rustycad_tkernel::{
    ncollection::List,
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkgeomalgo::geom_api::ProjectPointOnSurf;
use rustycad_tkgeombase::extrema::ExtAlgo;
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};
mod direction;
mod step;
struct FaceFrame {
    point: Point3,
    parameter: f64,
    tangent: Direction3,
    small: bool,
    step: f64,
    plane: ProjectPointOnSurf,
}
/// Select the closing face by OCCT's angular ordering; `get_face_off(&edge,&face,&pairs,&mut selected,&context)?`.
pub fn get_face_off(
    edge: &Edge,
    face: &Face,
    candidates: &List<CoupleOfShape>,
    selected: &mut Face,
    context: &Context,
) -> Result<bool, Failure> {
    let mut frame = face_frame(edge, face, candidates, context)?;
    let (_, normal, binormal) = direction::face_direction(edge, face, &mut frame, context)?;
    let mut choice = AngularChoice {
        minimum: 100.,
        done: true,
        reference: normal.crossed(binormal)?,
    };
    for pair in candidates.iter() {
        let candidate_face = Face::unchecked(pair.shape2().clone());
        let (computed, candidate_binormal) = candidate_direction(edge, pair, &mut frame, context)?;
        let angle = angle_with_reference(binormal, candidate_binormal, choice.reference);
        choice.consider(
            identity_angle(angle, face, &candidate_face, computed),
            candidate_face,
            selected,
        );
    }
    Ok(choice.done)
}
struct AngularChoice {
    minimum: f64,
    done: bool,
    reference: Direction3,
}
impl AngularChoice {
    fn consider(&mut self, mut angle: f64, face: Face, selected: &mut Face) {
        if angle.abs() < precision::confusion()
            || (angle - self.minimum).abs() < precision::confusion()
        {
            self.done = false;
        }
        if angle < 0. {
            angle += std::f64::consts::TAU;
        }
        if angle < self.minimum {
            self.minimum = angle;
            *selected = face;
        }
    }
}
fn candidate_direction(
    edge: &Edge,
    pair: &CoupleOfShape,
    frame: &mut FaceFrame,
    context: &Context,
) -> Result<(bool, Direction3), Failure> {
    let candidate_edge = Edge::unchecked(pair.shape1().clone());
    let candidate_face = Face::unchecked(pair.shape2().clone());
    let tangent = frame.tangent;
    if candidate_edge.orientation() != edge.orientation() {
        frame.tangent.reverse();
    }
    let (computed, _, binormal) =
        direction::face_direction(&candidate_edge, &candidate_face, frame, context)?;
    frame.tangent = tangent;
    Ok((computed, binormal))
}

fn identity_angle(angle: f64, face: &Face, candidate: &Face, computed: bool) -> f64 {
    if angle.abs() >= precision::angular() {
        return angle;
    }
    if candidate.is_equal(face) {
        return std::f64::consts::PI;
    }
    if candidate.is_same(face) || !computed {
        return std::f64::consts::TAU;
    }
    angle
}
fn angle_with_reference(first: Direction3, second: Direction3, reference: Direction3) -> f64 {
    let cross = first.xyz().crossed(second.xyz());
    let cosine = first.dot(second);
    let half = std::f64::consts::FRAC_PI_2;
    let mut angle = if cross.modulus() >= 0. {
        half * (1. - cosine)
    } else {
        std::f64::consts::TAU - half * (3. + cosine)
    };
    if cross.dot(reference.xyz()) < 0. {
        angle = -angle;
    }
    angle
}
fn face_frame(
    edge: &Edge,
    face: &Face,
    candidates: &List<CoupleOfShape>,
    context: &Context,
) -> Result<FaceFrame, Failure> {
    let (curve, first, last) = BRepTool::curve(edge)?;
    let parameter = algo_tools2d::intermediate_point(first, last);
    let point = curve.value(parameter)?;
    let mut derivative = Vector3::default();
    algo_tools2d::edge_tangent(edge, parameter, &mut derivative)?;
    let tangent = Direction3::from_vector(derivative)?;
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomPlane::from_point(point, tangent)?).cast();
    let mut plane = ProjectPointOnSurf::new();
    plane.init_surface(&surface, surface.bounds(), ExtAlgo::Grad)?;
    let (step, small) = step::minimum_step(edge, face, candidates, point, context)?;
    Ok(FaceFrame {
        point,
        parameter,
        tangent,
        small,
        step,
        plane,
    })
}
