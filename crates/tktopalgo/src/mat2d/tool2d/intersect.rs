// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT2d/MAT2d_Tool2d.cxx (IntersectBisector) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The intersection of two neighbour bisectors: the point equidistant from the
//! items they separate with the smallest parameters on both.
use super::create::domain;
use super::{MAT2D_TOLCONF, Mat2dTool2d, are_neighbours};
use crate::bisector::{BisectorCurve, BisectorInter};
use crate::mat::MatBisector;
use crate::mat2d::mat_edge_number as edge_number;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBoundedCurve, Geom2dTrimmedCurve, errors};
use rustycad_tkgeomalgo::int_res2d::{Domain, IntersectionSegment};
use rustycad_tkmath::gp::{Direction2, Point2};

/// The solution the search keeps: the parameters on both bisectors, the distance
/// and the point.
struct Solution {
    param1: f64,
    param2: f64,
    distance_mini: f64,
    point: Point2,
    valid: bool,
}

// The parameter of a point on the bisector basis of a trimmed bisector.
fn parameter_on(bisector: &Handle<Geom2dTrimmedCurve>, p: Point2) -> Result<f64, Failure> {
    let curve: Handle<dyn BisectorCurve> = bisector.basis_curve().cast();
    let curve: &dyn BisectorCurve = curve
        .get()
        .ok_or_else(|| errors::null_object("MAT2d_Tool2d::IntersectBisector: the basis is not a Bisector_Curve; expected a bisector"))?;
    curve.parameter(p)
}

impl Mat2dTool2d {
    /// `IntersectBisector(bisectorone, bisectortwo, intpnt)`: the distance of the
    /// point where the two bisectors meet at the same distance from the items they
    /// separate; `int_pnt` becomes the number of the new point, and the parameters of
    /// the MAT bisectors are updated. Infinite (and `int_pnt` untouched) when there is
    /// no valid point; `tool.intersect_bisector(&one, &two, &mut point)?`.
    pub fn intersect_bisector(
        &mut self,
        bisector_one: &Handle<MatBisector>,
        bisector_two: &Handle<MatBisector>,
        int_pnt: &mut i32,
    ) -> Result<f64, Failure> {
        let tolerance: f64 = MAT2D_TOLCONF;
        let bisector1: Handle<Geom2dTrimmedCurve> = self
            .change_geom_bis(bisector_one.bisector_number())?
            .change_value();
        let bisector2: Handle<Geom2dTrimmedCurve> = self
            .change_geom_bis(bisector_two.bisector_number())?
            .change_value();
        if bisector1.is_null() || bisector2.is_null() {
            return Ok(precision::infinite());
        }
        let edges: [i32; 4] = self.edges_of(bisector_one, bisector_two)?;
        if self.issued_from_reverse_connexions(&edges)? {
            return Ok(precision::infinite());
        }
        let domain1: Domain = domain(&bisector1, tolerance)?;
        let domain2: Domain = domain(&bisector2, tolerance)?;
        if domain1.last_parameter()? - domain1.first_parameter()? < tolerance
            || domain2.last_parameter()? - domain2.first_parameter()? < tolerance
        {
            return Ok(precision::infinite());
        }
        let mut intersect = BisectorInter::new();
        let (bis1, bis2) = (
            self.geom_bis(bisector_one.bisector_number())?,
            self.geom_bis(bisector_two.bisector_number())?,
        );
        intersect.perform(bis1, &domain1, bis2, &domain2, tolerance, tolerance, true)?;
        if !intersect.is_done() || intersect.is_empty()? {
            return Ok(precision::infinite());
        }
        let mut solution = Solution {
            param1: precision::infinite(),
            param2: precision::infinite(),
            distance_mini: precision::infinite(),
            point: Point2::default(),
            valid: false,
        };
        self.segment_solutions(
            &intersect,
            (bisector_one, bisector_two),
            (&bisector1, &bisector2),
            &mut solution,
        )?;
        self.point_solutions(&intersect, (bisector_one, bisector_two), &mut solution)?;
        if !solution.valid {
            return Ok(precision::infinite());
        }
        self.the_number_of_pnts += 1;
        self.the_geom_pnts
            .insert(self.the_number_of_pnts, solution.point);
        *int_pnt = self.the_number_of_pnts;
        self.accept_solution(
            (bisector_one, bisector_two),
            (&bisector1, &bisector2),
            &edges,
            solution,
        )
    }

    // The four items the two bisectors separate.
    fn edges_of(
        &self,
        one: &Handle<MatBisector>,
        two: &Handle<MatBisector>,
    ) -> Result<[i32; 4], Failure> {
        Ok([
            edge_number(&one.first_edge())?,
            edge_number(&one.second_edge())?,
            edge_number(&two.first_edge())?,
            edge_number(&two.second_edge())?,
        ])
    }

    // Bisectors of consecutive items issued from two connexions that are the reverse
    // of each other start at the same point: their intersection is not valid.
    fn issued_from_reverse_connexions(&self, edges: &[i32; 4]) -> Result<bool, Failure> {
        let [if1, is1, if2, is2] = *edges;
        let n: i32 = self.number_of_items()?;
        let circuit = self.circuit_ref()?;
        if !(are_neighbours(if1, is1, n)
            && are_neighbours(if2, is2, n)
            && circuit.connexion_on(is2)
            && circuit.connexion_on(is1))
        {
            return Ok(false);
        }
        let (c1, c2) = (circuit.connexion(is1)?, circuit.connexion(is2)?);
        Ok(c2.index_first_line() == c1.index_second_line()
            && c1.index_first_line() == c2.index_second_line())
    }

    // The small segments of the intersection: their first point equidistant from the
    // items (searched along a segment of up to 10 tolerances), with the smallest
    // parameters.
    fn segment_solutions(
        &self,
        intersect: &BisectorInter,
        mat: (&Handle<MatBisector>, &Handle<MatBisector>),
        geom: (&Handle<Geom2dTrimmedCurve>, &Handle<Geom2dTrimmedCurve>),
        solution: &mut Solution,
    ) -> Result<(), Failure> {
        for i in 1..=intersect.nb_segments()? {
            let segment: IntersectionSegment = intersect.segment(i)?;
            let Some((point, distance)) = self.retained_point(&segment, mat)? else {
                continue;
            };
            let parama: f64 = parameter_on(geom.0, point)?;
            let paramb: f64 = parameter_on(geom.1, point)?;
            if parama < solution.param1 && paramb < solution.param2 {
                *solution = Solution {
                    param1: parama,
                    param2: paramb,
                    distance_mini: distance,
                    point,
                    valid: true,
                };
            }
        }
        Ok(())
    }

    // The point of a bounded segment equidistant from the items, and its distance.
    fn retained_point(
        &self,
        segment: &IntersectionSegment,
        mat: (&Handle<MatBisector>, &Handle<MatBisector>),
    ) -> Result<Option<(Point2, f64)>, Failure> {
        let tolerance: f64 = MAT2D_TOLCONF;
        if !(segment.has_first_point() && segment.has_last_point()) {
            return Ok(None);
        }
        let p1: Point2 = segment.first_point()?.value();
        let p2: Point2 = segment.last_point()?.value();
        let segment_length: f64 = p1.distance(p2);
        if segment_length <= tolerance {
            let (same, distance) = self.is_same_distance(mat.0, mat.1, p1)?;
            return Ok(same.then_some((p1, distance)));
        }
        if segment_length > 10. * tolerance {
            return Ok(None);
        }
        let dir = Direction2::new(p2.x() - p1.x(), p2.y() - p1.y())?;
        let mut dist: f64 = 0.;
        while dist <= segment_length + tolerance {
            let point: Point2 = p1.translated(dist * rustycad_tkmath::gp::Vector2::from(dir));
            let (same, distance) = self.is_same_distance(mat.0, mat.1, point)?;
            if same {
                return Ok(Some((point, distance)));
            }
            dist += tolerance;
        }
        Ok(None)
    }

    // The points of the intersection: a single point is taken as it is (valid when
    // equidistant); several are taken when equidistant and farther than the
    // tolerance, with the smallest parameters.
    fn point_solutions(
        &self,
        intersect: &BisectorInter,
        mat: (&Handle<MatBisector>, &Handle<MatBisector>),
        solution: &mut Solution,
    ) -> Result<(), Failure> {
        let nb_points: i32 = intersect.nb_points()?;
        if nb_points == 1 {
            let point = intersect.point(1)?;
            let (same, distance) = self.is_same_distance(mat.0, mat.1, point.value())?;
            *solution = Solution {
                param1: point.param_on_first(),
                param2: point.param_on_second(),
                distance_mini: distance,
                point: point.value(),
                valid: same,
            };
            return Ok(());
        }
        for i in 1..=nb_points {
            let point = intersect.point(i)?;
            let (same, distance) = self.is_same_distance(mat.0, mat.1, point.value())?;
            if !(same && distance > MAT2D_TOLCONF) {
                continue;
            }
            let (parama, paramb) = (point.param_on_first(), point.param_on_second());
            if parama < solution.param1 && paramb < solution.param2 {
                *solution = Solution {
                    param1: parama,
                    param2: paramb,
                    distance_mini: distance,
                    point: point.value(),
                    valid: true,
                };
            }
        }
        Ok(())
    }

    // The checks of a valid point: not at the start of a bisector unless it comes
    // from a connexion between neighbours (the middle of the connexion may be on the
    // other bisector), and not beyond the parameters already found; the MAT bisectors
    // then take the parameters.
    fn accept_solution(
        &self,
        (one, two): (&Handle<MatBisector>, &Handle<MatBisector>),
        (bisector1, bisector2): (&Handle<Geom2dTrimmedCurve>, &Handle<Geom2dTrimmedCurve>),
        edges: &[i32; 4],
        mut solution: Solution,
    ) -> Result<f64, Failure> {
        let tolerance: f64 = MAT2D_TOLCONF;
        let [ie1, ie2, ie3, ie4] = *edges;
        let n: i32 = self.number_of_items()?;
        let circuit = self.circuit_ref()?;
        let neighbours: bool =
            are_neighbours(ie1, ie2, n) && are_neighbours(ie3, ie4, n) && ie2 == ie3;
        let mut extremite_controle: bool = true;
        if circuit.connexion_on(ie2) && neighbours {
            extremite_controle = false;
            solution.param1 += tolerance;
        }
        if circuit.connexion_on(ie4) && neighbours {
            extremite_controle = false;
            solution.param2 += tolerance;
        }
        if extremite_controle
            && (bisector1.start_point()?.distance(solution.point) < tolerance
                || bisector2.start_point()?.distance(solution.point) < tolerance)
        {
            return Ok(precision::infinite());
        }
        let second: f64 = one.second_parameter();
        if second < precision::infinite() && second < solution.param1 * (1. - tolerance) {
            return Ok(precision::infinite());
        }
        let first: f64 = two.first_parameter();
        if first < precision::infinite() && first < solution.param2 * (1. - tolerance) {
            return Ok(precision::infinite());
        }
        one.set_second_parameter(solution.param1);
        two.set_first_parameter(solution.param2);
        Ok(solution.distance_mini)
    }
}
