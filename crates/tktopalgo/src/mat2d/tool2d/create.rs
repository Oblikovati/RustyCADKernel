// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT2d/MAT2d_Tool2d.cxx (CreateBisector, TrimBisec and the static SetTrim and Domain) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The creation of the geometric bisector of two items of the circuit, restricted
//! by the half lines of the connexions of circles.
use super::{MAT2D_TOLCONF, Mat2dTool2d, are_neighbours, as_curve, is_point};
use crate::bisector::{Bisec, BisecAna};
use crate::mat::MatBisector;
use crate::mat2d::circuit::trimmed_item;
use crate::mat2d::mat_edge_number as edge_number;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dCircle, Geom2dConic, Geom2dCurve, Geom2dGeometry, Geom2dHyperbola, Geom2dLine,
    Geom2dParabola, Geom2dPoint, Geom2dTrimmedCurve,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkgeomalgo::int_res2d::Domain;
use rustycad_tkmath::gp::{Direction2, Point2, Vector2};
use std::f64::consts::PI;

/// The point of passage and the vectors of a new bisector.
struct Passage {
    point: Point2,
    first: Vector2,
    second: Vector2,
}

impl Mat2dTool2d {
    /// `CreateBisector(abisector)`: the geometric bisector of the two items that the
    /// MAT bisector separates, from its issue point; it takes the next number and the
    /// sense 1; `tool.create_bisector(&bisector)?`.
    pub fn create_bisector(&mut self, abisector: &Handle<MatBisector>) -> Result<(), Failure> {
        let edge1number: i32 = edge_number(&abisector.first_edge())?;
        let edge2number: i32 = edge_number(&abisector.second_edge())?;
        let circuit = self.circuit_ref()?;
        let mut ontheline: bool =
            are_neighbours(edge1number, edge2number, circuit.number_of_items());
        let initial_neighbour: bool = ontheline;
        if circuit.connexion_on(edge2number) {
            ontheline = false;
        }
        let elt1: Handle<dyn Geom2dGeometry> = circuit.value(edge1number)?;
        let elt2: Handle<dyn Geom2dGeometry> = circuit.value(edge2number)?;
        let passage = Passage {
            point: self.geom_pnt(abisector.issue_point())?,
            first: self.geom_vec(abisector.first_vector())?,
            second: self.geom_vec(abisector.second_vector())?,
        };
        let mut bisector = Bisec::new();
        self.perform_bisector(&mut bisector, (&elt1, &elt2), &passage, ontheline)?;
        self.trim_bisec(&mut bisector, edge1number, initial_neighbour, 1)?;
        self.trim_bisec(&mut bisector, edge2number, initial_neighbour, 2)?;
        self.the_number_of_bisectors += 1;
        self.the_geom_bisectors
            .insert(self.the_number_of_bisectors, bisector);
        abisector.set_bisector_number(self.the_number_of_bisectors);
        abisector.set_sense(1.);
        Ok(())
    }

    // The Perform of the bisector that suits the two items (curves or points).
    fn perform_bisector(
        &self,
        bisector: &mut Bisec,
        (elt1, elt2): (&Handle<dyn Geom2dGeometry>, &Handle<dyn Geom2dGeometry>),
        passage: &Passage,
        ontheline: bool,
    ) -> Result<(), Failure> {
        let (p, v1, v2) = (passage.point, passage.first, passage.second);
        let (dir, tol) = (self.the_direction, MAT2D_TOLCONF);
        match (is_point(elt1), is_point(elt2)) {
            (false, false) => {
                let (item1, item2) = (as_curve(elt1)?, as_curve(elt2)?);
                bisector.perform_curves(
                    &item1,
                    &item2,
                    p,
                    v1,
                    v2,
                    dir,
                    self.the_join_type,
                    tol,
                    ontheline,
                )
            }
            (true, true) => {
                let (point1, point2): (Handle<dyn Geom2dPoint>, Handle<dyn Geom2dPoint>) =
                    (elt1.cast(), elt2.cast());
                bisector.perform_points(&point1, &point2, p, v1, v2, dir, tol, ontheline)
            }
            (true, false) => {
                let point1: Handle<dyn Geom2dPoint> = elt1.cast();
                bisector.perform_point_curve(
                    &point1,
                    &as_curve(elt2)?,
                    p,
                    v1,
                    v2,
                    dir,
                    tol,
                    ontheline,
                )
            }
            (false, true) => {
                let point2: Handle<dyn Geom2dPoint> = elt2.cast();
                bisector.perform_curve_point(
                    &as_curve(elt1)?,
                    &point2,
                    p,
                    v1,
                    v2,
                    dir,
                    tol,
                    ontheline,
                )
            }
        }
    }

    /// `TrimBisec(Bis, IndexEdge, InitialNeighbour, StartOrEnd)`: the bisector of a
    /// circle linked to its neighbour by a connexion is cut by the half line from the
    /// centre through the end of the connexion (necessary to the logic of the
    /// algorithm).
    fn trim_bisec(
        &self,
        b1: &mut Bisec,
        index_edge: i32,
        initial_neighbour: bool,
        start_or_end: i32,
    ) -> Result<(), Failure> {
        let circuit = self.circuit_ref()?;
        let i_next: i32 = if index_edge == circuit.number_of_items() {
            1
        } else {
            index_edge + 1
        };
        let item: Handle<dyn Geom2dGeometry> = circuit.value(index_edge)?;
        if is_point(&item) || initial_neighbour {
            return Ok(());
        }
        let curve: Handle<dyn Geom2dCurve> = trimmed_item(&item)?.basis_curve();
        let circle: Handle<Geom2dCircle> = curve.cast();
        let mut limits: Vec<Handle<dyn Geom2dCurve>> = Vec::new();
        if let Some(circle) = circle.get() {
            let ori: Point2 = circle.location();
            if circuit.connexion_on(index_edge) && start_or_end == 1 {
                limits.push(half_line(
                    ori,
                    circuit.connexion(index_edge)?.point_on_first(),
                )?);
            }
            if circuit.connexion_on(i_next) && start_or_end == 2 {
                limits.push(half_line(
                    ori,
                    circuit.connexion(i_next)?.point_on_second(),
                )?);
            }
        }
        for line in limits {
            set_trim(b1, &line)?;
        }
        Ok(())
    }
}

// The half line from `ori` through `p2`, trimmed to `[0, Infinite]`.
fn half_line(ori: Point2, p2: Point2) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let dir = Direction2::new(p2.x() - ori.x(), p2.y() - ori.y())?;
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(ori, dir)).cast();
    Ok(Handle::new(Geom2dTrimmedCurve::new(
        &line,
        0.,
        precision::infinite(),
        true,
        true,
    )?)
    .cast())
}

/// `SetTrim(Bis, Line1)`: the bisector is cut at its first intersection with `line1`
/// farther than `10 MAT2d_TOLCONF` from its start.
fn set_trim(bis: &mut Bisec, line1: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
    let tolerance: f64 = MAT2D_TOLCONF;
    let bisector: Handle<Geom2dTrimmedCurve> = bis.change_value();
    let domain1: Domain = domain(&bisector, tolerance)?;
    let (ub1, ub2) = (bisector.first_parameter(), bisector.last_parameter());
    let first_point_bisector: Point2 = bisector.value(ub1)?;
    let mut u_trim: f64 = precision::infinite();
    let adap_bisector = Geom2dAdaptorCurve::with_curve(&bisector.cast::<dyn Geom2dCurve>())?;
    let adap_line1 = Geom2dAdaptorCurve::with_curve(line1)?;
    let mut intersect = GInter::new();
    intersect.perform_first_domain(&adap_bisector, &domain1, &adap_line1, tolerance, tolerance)?;
    if intersect.is_done() && !intersect.is_empty()? {
        for i in 1..=intersect.nb_points()? {
            let point = intersect.point(i)?;
            let distance: f64 = first_point_bisector.distance(point.value());
            if distance > 10. * tolerance && point.param_on_first() < u_trim {
                u_trim = point.param_on_first();
            }
        }
    }
    if u_trim < ub2 && u_trim > ub1 {
        bisector.set_trim(ub1, u_trim, true, true)?;
    }
    Ok(())
}

/// `Domain(Bisector1, Tolerance)`: the domain of a bisector, its end capped at
/// 10000, or on a parabola or a hyperbola at the parameter of the 50000 limit.
pub(super) fn domain(
    bisector1: &Handle<Geom2dTrimmedCurve>,
    tolerance: f64,
) -> Result<Domain, Failure> {
    let param1: f64 = bisector1.first_parameter();
    let mut param2: f64 = bisector1.last_parameter();
    if param2 > 10000. {
        param2 = capped_last(&bisector1.basis_curve());
    }
    let mut domain1 = Domain::bounded(
        bisector1.value(param1)?,
        param1,
        tolerance,
        bisector1.value(param2)?,
        param2,
        tolerance,
    );
    if bisector1.basis_curve().is_periodic() {
        domain1.set_equivalent_parameters(0., 2. * PI)?;
    }
    Ok(domain1)
}

// The capped end of a long bisector: the parameter of the 50000 limit of the conic of
// an analytic bisector (parabola or hyperbola), 10000 otherwise.
fn capped_last(basis: &Handle<dyn Geom2dCurve>) -> f64 {
    let limit: f64 = 50000.;
    let conic: Handle<dyn Geom2dCurve> = match basis.cast::<BisecAna>().get() {
        Some(ana) => ana.geom2d_curve(),
        None => return 10000.,
    };
    if let Some(parabola) = conic.cast::<Geom2dParabola>().get() {
        let focus: f64 = parabola.focal();
        let val1: f64 = (limit * focus).sqrt();
        let val2: f64 = (limit * limit).sqrt();
        return if val1 <= val2 { val1 } else { val2 };
    }
    if let Some(hyperbola) = conic.cast::<Geom2dHyperbola>().get() {
        let (majr, minr) = (hyperbola.major_radius(), hyperbola.minor_radius());
        let (valu1, valu2) = (limit / majr, limit / minr);
        let val1: f64 = (valu1 + (valu1 * valu1 - 1.).sqrt()).ln();
        let val2: f64 = (valu2 + (valu2 * valu2 + 1.).sqrt()).ln();
        return if val1 <= val2 { val1 } else { val2 };
    }
    10000.
}
