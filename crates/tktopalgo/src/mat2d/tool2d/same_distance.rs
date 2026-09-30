// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT2d/MAT2d_Tool2d.cxx (Projection, IsSameDistance and the static CheckEnds) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The test that a point is equidistant from the items two bisectors separate.
use super::{MAT2D_TOLCONF, Mat2dTool2d, is_point};
use crate::mat::MatBisector;
use crate::mat2d::circuit::trimmed_item;
use crate::mat2d::mat_edge_number as edge_number;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg2d::geom2d::{
    Geom2dBoundedCurve, Geom2dCurve, Geom2dGeometry, Geom2dPoint, Geom2dTrimmedCurve,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeombase::extrema::ExtPC2d;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::{CurveType, JoinType};
use rustycad_tkmath::gp::Point2;
use std::f64::consts::PI;

/// `CheckEnds(Elt, PCom, Distance, Tol)`: true when an end of the curve item is at
/// `distance` (within `tol`) from `pcom`; false for a point item.
fn check_ends(
    elt: &Handle<dyn Geom2dGeometry>,
    pcom: Point2,
    distance: f64,
    tol: f64,
) -> Result<bool, Failure> {
    if is_point(elt) {
        return Ok(false);
    }
    let curve: Handle<Geom2dTrimmedCurve> = trimmed_item(elt)?;
    let (a_pf, a_pl) = (curve.start_point()?, curve.end_point()?);
    let (df, dl) = (pcom.distance(a_pf), pcom.distance(a_pl));
    Ok((df - distance).abs() <= tol || (dl - distance).abs() <= tol)
}

impl Mat2dTool2d {
    /// `Projection(IndexElt, Point, Distance)`: the distance from `pcom` to the item
    /// `i_edge` (to its extrema between its connexions); false when the extrema are
    /// done but there is none.
    pub(super) fn projection(&self, i_edge: i32, pcom: Point2) -> Result<(bool, f64), Failure> {
        let circuit = self.circuit_ref()?;
        let elt: Handle<dyn Geom2dGeometry> = circuit.value(i_edge)?;
        if is_point(&elt) {
            return Ok((true, pcom.distance(elt.cast::<dyn Geom2dPoint>().pnt2d())));
        }
        let curve: Handle<Geom2dTrimmedCurve> = trimmed_item(&elt)?;
        let (param_min, param_max) = self.projection_range(i_edge, &curve)?;
        let c1 = Geom2dAdaptorCurve::with_curve(&curve.cast::<dyn Geom2dCurve>())?;
        let circle: bool = c1.get_type()? == CurveType::Circle;
        let (param_min, param_max) = widened_range(&c1, circle, param_min, param_max)?;
        let extremas = ExtPC2d::with_range(pcom, &c1, param_min, param_max, 1.0e-10)?;
        if !extremas.is_done() {
            let distance: f64 = if circle {
                c1.circle()?.radius()
            } else {
                precision::infinite()
            };
            return Ok((true, distance));
        }
        if extremas.nb_ext()? < 1 {
            return Ok((false, precision::infinite()));
        }
        let mut distance: f64 = precision::infinite();
        for i in 1..=extremas.nb_ext()? {
            let square: f64 = extremas.square_distance(i)?;
            if square < distance {
                distance = square;
            }
        }
        Ok((true, distance.sqrt()))
    }

    // The range of the item limited by its connexions, adjusted on a periodic curve.
    fn projection_range(
        &self,
        i_edge: i32,
        curve: &Handle<Geom2dTrimmedCurve>,
    ) -> Result<(f64, f64), Failure> {
        let circuit = self.circuit_ref()?;
        let mut param_min: f64 = curve.first_parameter();
        let mut param_max: f64 = curve.last_parameter();
        if circuit.connexion_on(i_edge) {
            param_min = circuit.connexion(i_edge)?.parameter_on_second();
        }
        let i_next: i32 = if i_edge == circuit.number_of_items() {
            1
        } else {
            i_edge + 1
        };
        if circuit.connexion_on(i_next) {
            param_max = circuit.connexion(i_next)?.parameter_on_first();
            if curve.basis_curve().is_periodic() {
                (param_min, param_max) =
                    elclib::adjust_periodic(0., 2. * PI, MAT2D_TOLCONF, param_min, param_max);
            }
        }
        Ok((param_min, param_max))
    }

    /// `IsSameDistance(BisectorOne, BisectorTwo, Point, Distance)`: true when `pcom` is
    /// at the same distance (within `300 MAT2d_TOLCONF`) from the four items the two
    /// bisectors separate, with that distance; an item without projection takes the
    /// distance of its partner when one of its ends is at it. The joins by
    /// intersection ignore the infinite distances.
    pub(super) fn is_same_distance(
        &self,
        bisector_one: &Handle<MatBisector>,
        bisector_two: &Handle<MatBisector>,
        pcom: Point2,
    ) -> Result<(bool, f64), Failure> {
        let edges: [i32; 4] = [
            edge_number(&bisector_one.first_edge())?,
            edge_number(&bisector_one.second_edge())?,
            edge_number(&bisector_two.first_edge())?,
            edge_number(&bisector_two.second_edge())?,
        ];
        let mut dist: [f64; 4] = [0.; 4];
        let mut done: [bool; 4] = [true; 4];
        (done[0], dist[0]) = self.projection(edges[0], pcom)?;
        (done[1], dist[1]) = self.projection(edges[1], pcom)?;
        self.share_distance(&edges, &mut dist, &done, (0, 1), pcom)?;
        for k in [2, 3] {
            if edges[k] == edges[0] {
                dist[k] = dist[0];
            } else if edges[k] == edges[1] {
                dist[k] = dist[1];
            } else {
                (done[k], dist[k]) = self.projection(edges[k], pcom)?;
            }
        }
        self.share_distance(&edges, &mut dist, &done, (2, 3), pcom)?;
        Ok(self.common_distance(&dist))
    }

    // The item of a pair without projection takes the distance of the other when one
    // of its ends is at it.
    fn share_distance(
        &self,
        edges: &[i32; 4],
        dist: &mut [f64; 4],
        done: &[bool; 4],
        (a, b): (usize, usize),
        pcom: Point2,
    ) -> Result<(), Failure> {
        let eps: f64 = 1.0e-7;
        let (from, to) = match (done[a], done[b]) {
            (true, false) => (a, b),
            (false, true) => (b, a),
            _ => return Ok(()),
        };
        let elt: Handle<dyn Geom2dGeometry> = self.circuit_ref()?.value(edges[to])?;
        let tol: f64 = std_max(precision::confusion(), eps * dist[from]);
        if check_ends(&elt, pcom, dist[from], tol)? {
            dist[to] = dist[from];
        }
        Ok(())
    }

    // The common distance of the four items, infinite when they differ.
    fn common_distance(&self, dist: &[f64; 4]) -> (bool, f64) {
        let eps_dist: f64 = MAT2D_TOLCONF * 300.;
        let intersection: bool = self.the_join_type == JoinType::Intersection;
        let mut distance: f64 = dist[0];
        if intersection
            && precision::is_infinite(distance)
            && let Some(finite) = dist[1..].iter().find(|d| !precision::is_infinite(**d))
        {
            distance = *finite;
        }
        for d in dist {
            if intersection && precision::is_infinite(*d) {
                continue;
            }
            if (d - distance).abs() > eps_dist {
                return (false, precision::infinite());
            }
        }
        (true, distance)
    }
}

// The range widened by the tolerance: by `100 Eps` (or `Eps / R` for a small radius)
// on a circle whose range stays below a turn, by `Eps` otherwise.
fn widened_range(
    c1: &Geom2dAdaptorCurve,
    circle: bool,
    param_min: f64,
    param_max: f64,
) -> Result<(f64, f64), Failure> {
    let eps: f64 = MAT2D_TOLCONF;
    if !circle {
        return Ok((param_min - eps, param_max + eps));
    }
    let r: f64 = c1.circle()?.radius();
    let eps_circ: f64 = if r < 1. { eps / r } else { 100. * eps };
    if (param_max - param_min + 2. * eps_circ) < 2. * PI {
        return Ok((param_min - eps_circ, param_max + eps_circ));
    }
    Ok((param_min, param_max))
}
