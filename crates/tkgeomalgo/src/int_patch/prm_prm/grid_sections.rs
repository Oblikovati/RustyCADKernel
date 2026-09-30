// Rust translation of the IntPolyh branch of IntPatch_PrmPrmIntersection::Perform(Surf1, D1,
// Surf2, D2, TolTangency, Epsilon, Deflection, Increment, ClearFlag)
// (ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_PrmPrmIntersection.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The marching from the section lines and the tangent zones of the polyhedral
//! intersection of IntPolyh.
use super::IntPatchPrmPrmIntersection;
use super::helpers::{adjust_on_periodic, widen_bounds};
use super::perform_grid::DomainSamples;
use super::start_params;
use super::walk_steps::{AcceptOptions, WalkInputs, is_new_start, keep_rejected};
use crate::int_patch::{IntPatchPointLine, IntPatchWLine};
use crate::int_polyh::IntPolyhIntersection;
use crate::int_walk::IntWalkPWalking;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Direction3, Line3, Point3, Vector3};

/// The walking of the polyhedral branch: the walking, the intersection and the bounds
/// of the current line.
struct PolyhWalk<'a> {
    inputs: &'a WalkInputs<'a>,
    pw: IntWalkPWalking,
    interference: IntPolyhIntersection,
    periods: [f64; 4],
}

/// The line-duplication test of the section lines: the ends of the walking match the
/// ends of a line of at least 3 points (in either order) and the middle point of that
/// line is within `2 Epsilon` of a segment of the walking.
fn duplicated_section(
    walk: &PolyhWalk<'_>,
    lines: &[Handle<dyn crate::int_patch::IntPatchLine>],
    nb_lig_calculee: i32,
) -> Result<Option<i32>, Failure> {
    let pw: &IntWalkPWalking = &walk.pw;
    let tol: f64 = walk.inputs.tol_tangency;
    let ipw_nb_points: i32 = pw.nb_points()?;
    let (debut, fin) = (pw.value(1)?.value(), pw.value(ipw_nb_points)?.value());
    for ver in 1..=nb_lig_calculee {
        let verwline: Handle<IntPatchWLine> = lines[(ver - 1) as usize].cast::<IntPatchWLine>();
        let nb_points_ver: i32 = verwline.nb_pnts()?;
        if nb_points_ver < 3 {
            continue;
        }
        let (p21, p22) = (
            verwline.point(1)?.value(),
            verwline.point(nb_points_ver)?.value(),
        );
        let (d11, d12) = (debut.distance(&p21), debut.distance(&p22));
        let (d21, d22) = (fin.distance(&p21), fin.distance(&p22));
        if !((d11 <= tol && d22 <= tol) || (d12 <= tol && d21 <= tol)) {
            continue;
        }
        let mx: i32 = nb_points_ver / 2 + nb_points_ver % 2;
        let px: Point3 = verwline.point(mx)?.value();
        if is_near_walking(pw, ipw_nb_points, px, 2.0 * walk.inputs.epsilon)? {
            return Ok(Some(ver));
        }
    }
    Ok(None)
}

/// Whether `px` is within `tol` of the line of a segment of the walking.
fn is_near_walking(
    pw: &IntWalkPWalking,
    nb_points: i32,
    px: Point3,
    tol: f64,
) -> Result<bool, Failure> {
    for m in 1..nb_points {
        let (p1, p2) = (pw.value(m)?.value(), pw.value(m + 1)?.value());
        let vec12: Vector3 = Vector3::between(p1, p2);
        if vec12.square_magnitude() < 1.0e-20 {
            continue;
        }
        let lin12 = Line3::new(p1, Direction3::from_vector(vec12)?);
        if lin12.distance(px) <= tol {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The line-duplication test of the tangent zones: the last point of the walking is on
/// a line, or its first (else its last) point is at an end of a line.
fn duplicated_zone(
    walk: &PolyhWalk<'_>,
    lines: &[Handle<dyn crate::int_patch::IntPatchLine>],
    nb_lig_calculee: i32,
) -> Result<bool, Failure> {
    let pw: &IntWalkPWalking = &walk.pw;
    let tol: f64 = walk.inputs.tol_tangency;
    let point_fin = pw.value(pw.nb_points()?)?;
    let (debut, fin) = (pw.value(1)?.value(), point_fin.value());
    for ver in 1..=nb_lig_calculee {
        let verwline: Handle<IntPatchWLine> = lines[(ver - 1) as usize].cast::<IntPatchWLine>();
        // Check end point if it is on existing line. Start point is checked before.
        if super::helpers::is_point_on_line(&point_fin, &verwline, walk.inputs.deflection)? {
            return Ok(true);
        }
        let ver_debut: Point3 = verwline.point(1)?.value();
        let ver_fin: Point3 = verwline.point(verwline.nb_pnts()?)?.value();
        if debut.distance(&ver_debut) < tol || fin.distance(&ver_fin) < tol {
            return Ok(true);
        }
    }
    Ok(false)
}

impl IntPatchPrmPrmIntersection {
    /// The IntPolyh branch of the main `Perform`.
    pub(super) fn perform_polyh(
        &mut self,
        inputs: &WalkInputs<'_>,
        samples: &DomainSamples,
        clear_flag: bool,
    ) -> Result<(), Failure> {
        self.empt = true;
        if clear_flag {
            self.slin.clear();
        }
        let d2 = inputs.d2.expect("the main Perform has two domains");
        let [nbu1, nbv1, nbu2, nbv2] = samples.counts;
        let interference: IntPolyhIntersection =
            if inputs.d1.is_uniform_sampling()? || d2.is_uniform_sampling()? {
                IntPolyhIntersection::with_samples(
                    inputs.surf1,
                    (nbu1, nbv1),
                    inputs.surf2,
                    (nbu2, nbv2),
                )?
            } else {
                let pars1 = (&samples.u_pars1, &samples.v_pars1);
                let pars2 = (&samples.u_pars2, &samples.v_pars2);
                IntPolyhIntersection::with_parameters(inputs.surf1, pars1, inputs.surf2, pars2)?
            };
        self.done = interference.is_done();
        if !self.done {
            return Ok(());
        }
        let pw = IntWalkPWalking::new(
            inputs.surf1,
            inputs.surf2,
            inputs.tol_tangency,
            inputs.epsilon,
            inputs.deflection,
            inputs.increment,
        )?;
        let mut walk = PolyhWalk {
            inputs,
            pw,
            interference,
            periods: samples.periods,
        };
        if walk.interference.nb_section_lines() >= 1 {
            for line in sorted_lines(&walk.interference)? {
                self.walk_section_line(&mut walk, line)?;
            }
        }
        adjust_on_periodic(inputs.surf1, inputs.surf2, &self.slin)?;
        self.walk_tangent_zones(&mut walk)
    }

    /// The marching from the start points of one section line.
    fn walk_section_line(&mut self, walk: &mut PolyhWalk<'_>, line: i32) -> Result<(), Failure> {
        let nbp: i32 = walk.interference.nb_points_in_line(line)?;
        if nbp == 0 {
            return Ok(());
        }
        let mut tab_pt_dep: Vec<bool> = vec![false; (nbp + 1) as usize];
        let bounds = line_bounds(&walk.interference, line, nbp)?;
        const LIMIT_N: i32 = 5;
        let (mut n, mut lignetrouvee) = (0, false);
        loop {
            n += 1;
            let nbps2: i32 = match n {
                1 => {
                    if nbp < 3 {
                        n = LIMIT_N;
                    }
                    if nbp > 1 { nbp / 2 } else { 1 }
                }
                2 => 1,
                3 => nbp - 1,
                4 => 3 * nbp / 4,
                5 => nbp / 4,
                _ => {
                    n += 1;
                    n - 4
                }
            };
            if !tab_pt_dep[nbps2 as usize] {
                tab_pt_dep[nbps2 as usize] = true;
                lignetrouvee |= self.walk_section_point(walk, (line, nbps2), bounds)?;
            }
            if !(nbp > 5 && (n < LIMIT_N || !lignetrouvee) && (n - 3 < nbp || lignetrouvee)) {
                return Ok(());
            }
        }
    }

    /// The walking from the point `nbps2` of a section line; returns whether a line was
    /// added.
    fn walk_section_point(
        &mut self,
        walk: &mut PolyhWalk<'_>,
        (line, nbps2): (i32, i32),
        (min, max): ([f64; 4], [f64; 4]),
    ) -> Result<bool, Failure> {
        let p = walk.interference.get_line_point(line, nbps2)?;
        let start = start_params([p[3], p[4], p[5], p[6]])?;
        let Some(start_pon2s) = walk.pw.perform_first_point(&start)? else {
            return Ok(false);
        };
        let nb_lig_calculee: i32 = self.slin.len() as i32;
        if !is_new_start(walk.inputs, &start_pon2s, &self.slin)? {
            return Ok(false);
        }
        walk.pw.perform_bounded(&start, min, max)?;
        if !walk.pw.is_done() || walk.pw.nb_points()? <= 2 {
            return Ok(false);
        }
        // Try to extend the intersection line to the boundary, if it is possible.
        walk.pw
            .put_to_boundary(walk.inputs.surf1, walk.inputs.surf2)?;
        if walk.pw.nb_points()? < 3 {
            return Ok(false);
        }
        let has_been_added: bool = walk.densify()?;
        let ends = (
            walk.pw.value(1)?.value(),
            walk.pw.value(walk.pw.nb_points()?)?.value(),
        );
        let duplicated = duplicated_section(walk, &self.slin, nb_lig_calculee)?;
        if keep_rejected(&walk.pw, duplicated, &mut self.slin)? {
            return Ok(false);
        }
        let options = AcceptOptions {
            loose: true,
            purging: Some(!has_been_added),
            several: Some(walk.periods),
        };
        self.accept_walking(walk.inputs, &walk.pw, &options, ends)
    }

    /// The marching from the points of the tangent zones, in the bounds of all of them.
    fn walk_tangent_zones(&mut self, walk: &mut PolyhWalk<'_>) -> Result<(), Failure> {
        let nb_tan_zon: i32 = walk.interference.nb_tangent_zones();
        // NbPointsInTangentZone is always 1 (eap).
        let (mut min, mut max) = ([f64::MAX; 4], [-f64::MAX; 4]);
        for z in 1..=nb_tan_zon {
            let p = walk.interference.get_tangent_zone_point(z, 1)?;
            widen_bounds((&mut min, &mut max), [p[3], p[4], p[5], p[6]]);
        }
        for z in 1..=nb_tan_zon {
            let p = walk.interference.get_tangent_zone_point(z, 1)?;
            self.walk_zone_point(walk, [p[3], p[4], p[5], p[6]], (min, max))?;
        }
        Ok(())
    }

    /// The walking from the point of a tangent zone.
    fn walk_zone_point(
        &mut self,
        walk: &mut PolyhWalk<'_>,
        uv: [f64; 4],
        (min, max): ([f64; 4], [f64; 4]),
    ) -> Result<(), Failure> {
        let start = start_params(uv)?;
        let Some(start_pon2s) = walk.pw.perform_first_point(&start)? else {
            return Ok(());
        };
        let nb_lig_calculee: i32 = self.slin.len() as i32;
        if !is_new_start(walk.inputs, &start_pon2s, &self.slin)? {
            return Ok(());
        }
        walk.pw.perform_bounded(&start, min, max)?;
        if !walk.pw.is_done() || walk.pw.nb_points()? <= 2 {
            return Ok(());
        }
        let has_been_added: bool = walk.densify()?;
        let ends = (
            walk.pw.value(1)?.value(),
            walk.pw.value(walk.pw.nb_points()?)?.value(),
        );
        // A rejected zone line is not compared with the line it duplicates.
        if duplicated_zone(walk, &self.slin, nb_lig_calculee)? {
            return Ok(());
        }
        let options = AcceptOptions {
            loose: false,
            purging: Some(!has_been_added),
            several: None,
        };
        self.accept_walking(walk.inputs, &walk.pw, &options, ends)?;
        Ok(())
    }
}

impl PolyhWalk<'_> {
    /// A walking of fewer than 40 points gets additional points; returns whether some
    /// were added.
    fn densify(&mut self) -> Result<bool, Failure> {
        const MIN_NB_POINTS: i32 = 40;
        if self.pw.nb_points()? >= MIN_NB_POINTS {
            return Ok(false);
        }
        self.pw
            .seek_additional_points(self.inputs.surf1, self.inputs.surf2, MIN_NB_POINTS)
    }
}

/// The section lines sorted by decreasing number of points (the native bubble sort,
/// stable).
fn sorted_lines(interference: &IntPolyhIntersection) -> Result<Vec<i32>, Failure> {
    let nb_lig_sec: i32 = interference.nb_section_lines();
    let mut tab_l: Vec<i32> = (1..=nb_lig_sec).collect();
    loop {
        let mut triok = true;
        for b in 1..tab_l.len() {
            let nb_b: i32 = interference.nb_points_in_line(tab_l[b])?;
            let nb_a: i32 = interference.nb_points_in_line(tab_l[b - 1])?;
            if nb_b > nb_a {
                tab_l.swap(b, b - 1);
                triok = false;
            }
        }
        if triok {
            return Ok(tab_l);
        }
    }
}

/// The parameter bounds of the points of a section line.
fn line_bounds(
    interference: &IntPolyhIntersection,
    line: i32,
    nbp: i32,
) -> Result<([f64; 4], [f64; 4]), Failure> {
    let p = interference.get_line_point(line, 1)?;
    let (mut min, mut max) = ([p[3], p[4], p[5], p[6]], [p[3], p[4], p[5], p[6]]);
    for ilig in 2..=nbp {
        let p = interference.get_line_point(line, ilig)?;
        widen_bounds((&mut min, &mut max), [p[3], p[4], p[5], p[6]]);
    }
    Ok((min, max))
}
