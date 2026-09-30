// Rust translation of ExtendLineInCommonZone of
// ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_PWalking.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExtendLineInCommonZone`: the line goes on through a tangent zone along the
//! given isoparametric; the points walked are added when the zone is left (or the
//! walk stays straight along the zone).
use super::IntWalkPWalking;
use crate::int_imp::IntImpConstIsoparametric as Iso;
use crate::int_surf::IntSurfPntOn2S;
use crate::int_walk::IntWalkStatusDeflection as Status;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::adaptor3d::h_surface_tool as tool;
use rustycad_tkmath::gp::{self, Direction2, Point3, Vector2};
use rustycad_tkmath::math::FunctionSetRoot;
use std::f64::consts::PI;

/// The walk through the tangent zone.
struct ZoneWalk {
    param: Array1<'static, f64>,
    status: Status,
    b_stop: bool,
    b_out_of_tangent_zone: bool,
    d_inc_key: i32,
    nb_iter_without_append: i32,
    nb_equal_points: i32,
    new_points: Vec<IntSurfPntOn2S>,
}

/// What one step of the zone walk decided.
enum ZoneFlow {
    Next,
    /// The native `return bOutOfTangentZone` from inside the loop.
    Return(bool),
}

impl IntWalkPWalking {
    /// `ExtendLineInCommonZone(theChoixIso, theDirectionFlag)`: true when the line was
    /// extended through the tangent zone and its last point is out of it.
    pub fn extend_line_in_common_zone(
        &mut self,
        the_choix_iso: Iso,
        the_direction_flag: bool,
    ) -> Result<bool, Failure> {
        let (caro1, caro2) = self.surfaces();
        let first: [f64; 4] = [
            tool::first_u_parameter(&caro1)?,
            tool::first_v_parameter(&caro1)?,
            tool::first_u_parameter(&caro2)?,
            tool::first_v_parameter(&caro2)?,
        ];
        let last: [f64; 4] = [
            tool::last_u_parameter(&caro1)?,
            tool::last_v_parameter(&caro1)?,
            tool::last_u_parameter(&caro2)?,
            tool::last_v_parameter(&caro2)?,
        ];
        let mut zw = ZoneWalk {
            param: one_based(4)?,
            status: Status::OK,
            b_stop: !self.my_intersection_on2s.is_tangent()?,
            b_out_of_tangent_zone: false,
            d_inc_key: 1,
            nb_iter_without_append: 0,
            nb_equal_points: 0,
            new_points: Vec::new(),
        };
        let (u1, v1, u2, v2) = self.previous_point.parameters();
        let start: [f64; 4] = [u1, v1, u2, v2];
        (zw.param[1], zw.param[2], zw.param[3], zw.param[4]) = (u1, v1, u2, v2);
        if (0..4).any(|k| start[k] - first[k] < self.reso[k])
            || (0..4).any(|k| start[k] - last[k] > -self.reso[k])
        {
            return Ok(false);
        }
        while !zw.b_stop {
            zw.nb_iter_without_append += 1;
            if zw.nb_iter_without_append > 20 || zw.nb_equal_points > 20 {
                zw.b_stop = true;
                break;
            }
            if let ZoneFlow::Return(result) =
                self.zone_step(&mut zw, the_choix_iso, the_direction_flag)?
            {
                return Ok(result);
            }
        }
        self.finish_zone(zw, the_choix_iso)
    }
    /// One step of the zone walk.
    fn zone_step(
        &mut self,
        zw: &mut ZoneWalk,
        the_choix_iso: Iso,
        the_direction_flag: bool,
    ) -> Result<ZoneFlow, Failure> {
        let sv_param: [f64; 4] = self.zone_advance(zw, the_choix_iso);
        let mut rsnld =
            FunctionSetRoot::without_tolerance(self.my_intersection_on2s.function(), 100)?;
        let mut choix_iso: Iso =
            self.my_intersection_on2s
                .perform_iso(&zw.param, &mut rsnld, the_choix_iso)?;
        if !self.my_intersection_on2s.is_done() || self.my_intersection_on2s.is_empty()? {
            return Ok(ZoneFlow::Return(zw.b_out_of_tangent_zone));
        }
        zw.status = self.test_deflection(choix_iso, zw.status)?;
        if zw.status == Status::OK {
            for k in 0..4 {
                if self.pasuv[k] < self.pas_init[k] {
                    self.pasuv[k] = self.pas_init[k];
                }
            }
        }
        match zw.status {
            Status::ArretSurPointPrecedent => {
                zw.b_stop = true;
                zw.b_out_of_tangent_zone = !self.my_intersection_on2s.is_tangent()?;
            }
            Status::PasTropGrand => self.zone_too_great(zw, sv_param),
            Status::PointConfondu => {
                for k in 0..4 {
                    if self.pasuv[k] < self.pas_init[k] {
                        self.pasuv[k] += (self.pas_init[k] - self.pasuv[k]) * 0.1;
                    }
                }
            }
            Status::OK | Status::ArretSurPoint => {
                return self.zone_accept(
                    zw,
                    (&mut rsnld, &mut choix_iso),
                    (the_choix_iso, the_direction_flag),
                );
            }
            Status::StepTooSmall => {}
        }
        Ok(ZoneFlow::Next)
    }
    /// The parameters one step along the previous tangents (as in the marching
    /// loop, with `5 * dIncKey` on a tiny step of the isoparametric); returns them.
    fn zone_advance(&self, zw: &mut ZoneWalk, the_choix_iso: Iso) -> [f64; 4] {
        let mut f: f64 = match the_choix_iso {
            Iso::UIsoparametricOnCaro1 => self.previousd1.x().abs(),
            Iso::VIsoparametricOnCaro1 => self.previousd1.y().abs(),
            Iso::UIsoparametricOnCaro2 => self.previousd2.x().abs(),
            Iso::VIsoparametricOnCaro2 => self.previousd2.y().abs(),
        };
        if f < 0.1 {
            f = 0.1;
        }
        let (u1, v1, u2, v2) = self.previous_point.parameters();
        let start: [f64; 4] = [u1, v1, u2, v2];
        let sens: f64 = f64::from(self.sens_cheminement);
        let components: [f64; 4] = [
            self.previousd1.x(),
            self.previousd1.y(),
            self.previousd2.x(),
            self.previousd2.y(),
        ];
        let mut sv_param: [f64; 4] = [0.0; 4];
        for k in 0..4 {
            let mut dp: f64 = sens * self.pasuv[k] * components[k] / f;
            if the_choix_iso as usize == k && dp.abs() < 1.0e-7 {
                dp *= 5. * f64::from(zw.d_inc_key);
            }
            zw.param[k as i32 + 1] = start[k] + dp;
            sv_param[k] = zw.param[k as i32 + 1];
        }
        sv_param
    }
    /// A step too great: the parameters come back and the initial steps come a
    /// tenth of the way down to the saved ones (the iteration then does not count).
    fn zone_too_great(&mut self, zw: &mut ZoneWalk, sv_param: [f64; 4]) {
        for (k, value) in sv_param.into_iter().enumerate() {
            zw.param[k as i32 + 1] = value;
        }
        let mut decrease: bool = false;
        for k in 0..4 {
            if self.pas_sav[k] < self.pas_init[k] {
                self.pas_init[k] -= (self.pas_init[k] - self.pas_sav[k]) * 0.1;
                decrease = true;
            }
        }
        if decrease {
            zw.nb_iter_without_append -= 1;
        }
    }
    /// An accepted point in the zone walk: the stop test; a point that did not move
    /// counts as equal; the walk stops when the zone is left.
    fn zone_accept(
        &mut self,
        zw: &mut ZoneWalk,
        (rsnld, choix_iso): (&mut FunctionSetRoot, &mut Iso),
        (the_choix_iso, the_direction_flag): (Iso, bool),
    ) -> Result<ZoneFlow, Failure> {
        zw.b_stop = self.test_arret(the_direction_flag, &mut zw.param, choix_iso)?;
        if !zw.b_stop {
            let (u11, v11, u12, v12) = self.my_intersection_on2s.point()?.parameters();
            let (u21, v21, u22, v22) = self.previous_point.parameters();
            let r = self.reso;
            if ((u11 - u21).abs() < r[0] && (v11 - v21).abs() < r[1])
                || ((u12 - u22).abs() < r[2] && (v12 - v22).abs() < r[3])
            {
                zw.nb_equal_points += 1;
            } else {
                zw.nb_equal_points = 0;
            }
        }
        let tangent: bool = self.my_intersection_on2s.is_tangent()?;
        zw.b_stop = zw.b_stop || !tangent;
        zw.b_out_of_tangent_zone = !tangent;
        if !zw.b_stop {
            return self.zone_walk_on(zw);
        }
        if self.close && self.line_ref()?.nb_points() >= 1 {
            if !zw.b_out_of_tangent_zone {
                zw.new_points.push(self.line_ref()?.value(1)?);
            }
            zw.nb_iter_without_append = 0;
            return Ok(ZoneFlow::Next);
        }
        *choix_iso = self
            .my_intersection_on2s
            .perform_iso(&zw.param, rsnld, the_choix_iso)?;
        if self.my_intersection_on2s.is_empty()? {
            zw.b_stop = true;
            zw.b_out_of_tangent_zone = false;
            return Ok(ZoneFlow::Next);
        }
        self.previous_point = *self.my_intersection_on2s.point()?;
        if self.inside_walking_bounds(self.previous_point.parameters())
            && let Some(flow) = self.zone_candidate(zw, !zw.b_out_of_tangent_zone)?
        {
            return Ok(flow);
        }
        Ok(ZoneFlow::Next)
    }
    /// A point that goes on in the zone: kept as the previous point (a valid one)
    /// and as a new point, or, out of the walking bounds, the steps of the parameters
    /// out of them are halved.
    fn zone_walk_on(&mut self, zw: &mut ZoneWalk) -> Result<ZoneFlow, Failure> {
        let (u1, v1, u2, v2) = self.my_intersection_on2s.point()?.parameters();
        let valid: bool = self.inside_walking_bounds((u1, v1, u2, v2));
        if valid {
            self.take_new_point()?;
            if let Some(flow) = self.zone_candidate(zw, true)? {
                return Ok(flow);
            }
        }
        if self.line_ref()?.nb_points() == 2 {
            self.pas_sav = self.pasuv;
        }
        if !valid {
            let params: [f64; 4] = [u1, v1, u2, v2];
            for (k, value) in params.into_iter().enumerate() {
                if value > self.upper[k] || value < self.lower[k] {
                    self.pasuv[k] *= 0.5;
                }
            }
        }
        Ok(ZoneFlow::Next)
    }
    /// The previous point becomes a new point of the zone unless it is at the first
    /// point of the line (`dIncKey` grows; the 5000th time ends the extension).
    fn zone_candidate(
        &mut self,
        zw: &mut ZoneWalk,
        keep: bool,
    ) -> Result<Option<ZoneFlow>, Failure> {
        let mut add_point: bool = true;
        let line = self.line_ref()?;
        if line.nb_points() >= 1 {
            let pf: Point3 = line.value(1)?.value();
            let pl: Point3 = self.previous_point.value();
            if pf.distance(&pl) < precision::confusion() {
                zw.d_inc_key += 1;
                if zw.d_inc_key == 5000 {
                    return Ok(Some(ZoneFlow::Return(zw.b_out_of_tangent_zone)));
                }
                add_point = false;
            }
        }
        if add_point && keep {
            zw.new_points.push(self.previous_point);
            zw.nb_iter_without_append = 0;
        }
        Ok(None)
    }
    /// After the zone walk: the new points are added when the last point is at a
    /// border, or when the walk ended accepted and does not turn by more than a
    /// quarter turn across the isoparametric.
    fn finish_zone(&mut self, zw: ZoneWalk, the_choix_iso: Iso) -> Result<bool, Failure> {
        let mut extend: bool = false;
        let candidates: [Option<IntSurfPntOn2S>; 2] =
            [Some(self.previous_point), zw.new_points.last().copied()];
        for (pit, candidate) in candidates.into_iter().enumerate() {
            let Some(point) = candidate else {
                break;
            };
            if pit > 0 && extend {
                break;
            }
            if self.near_border(point.parameters()) {
                extend = true;
            }
        }
        if !extend && zw.status == Status::OK {
            extend = straight_along_zone(&zw.new_points, the_choix_iso)?;
        }
        if !extend {
            return Ok(false);
        }
        for point in &zw.new_points {
            self.add_a_point(point)?;
        }
        Ok(zw.b_out_of_tangent_zone)
    }
    /// Whether the parameters are within the resolutions of a walking bound.
    fn near_border(&self, (u1, v1, u2, v2): (f64, f64, f64, f64)) -> bool {
        let (lo, up, r) = (self.lower, self.upper, self.reso);
        ((u1 - lo[0]) < r[0])
            || ((up[0] - u1) < r[0])
            || ((u2 - lo[2]) < r[2])
            || ((up[2] - u2) < r[2])
            || ((v1 - lo[1]) < r[1])
            || ((up[1] - v1) < r[1])
            || ((v2 - lo[3]) < r[3])
            || ((up[3] - v2) < r[3])
    }
}

/// Whether the new points go straight along the zone: true unless, when the zone
/// direction crosses the isoparametric by more than a quarter turn, two successive
/// steps (skipping null ones) turn by more than a quarter turn.
fn straight_along_zone(points: &[IntSurfPntOn2S], the_choix_iso: Iso) -> Result<bool, Failure> {
    if points.len() <= 1 {
        return Ok(true);
    }
    let first_index: usize = if (the_choix_iso as usize) < 2 { 0 } else { 2 };
    let on_first: bool = first_index == 0;
    let step = |a: &IntSurfPntOn2S, b: &IntSurfPntOn2S| -> Vector2 {
        Vector2::between(a.value_on_surface(on_first), b.value_on_surface(on_first))
    };
    let zone_dir: Vector2 = step(&points[0], &points[points.len() - 1]);
    let iso_dir: Direction2 = if matches!(
        the_choix_iso,
        Iso::VIsoparametricOnCaro1 | Iso::VIsoparametricOnCaro2
    ) {
        Direction2::new(1.0, 0.0)?
    } else {
        Direction2::new(0.0, 1.0)?
    };
    if zone_dir.square_magnitude() <= gp::resolution() {
        return Ok(true);
    }
    let piquota: f64 = PI * 0.25;
    let iso_vec = Vector2::new(iso_dir.x(), iso_dir.y());
    if zone_dir.angle(iso_vec)?.abs() <= piquota {
        return Ok(true);
    }
    let asqresol: f64 = gp::resolution() * gp::resolution();
    let (mut d1, mut ii) = non_null_step(points, 0, asqresol, &step);
    while ii + 1 < points.len() {
        let (next_d1, jj) = non_null_step(points, ii, asqresol, &step);
        ii = jj;
        if d1.angle(next_d1)?.abs() > piquota {
            return Ok(false);
        }
        d1 = next_d1;
    }
    Ok(true)
}

/// The first step from index `from` (0-based) that is not null (or the last one),
/// and the index after it.
fn non_null_step(
    points: &[IntSurfPntOn2S],
    from: usize,
    asqresol: f64,
    step: &impl Fn(&IntSurfPntOn2S, &IntSurfPntOn2S) -> Vector2,
) -> (Vector2, usize) {
    let mut index: usize = from;
    loop {
        let d: Vector2 = step(&points[index], &points[index + 1]);
        index += 1;
        if !(d.square_magnitude() < asqresol && index + 1 < points.len()) {
            return (d, index);
        }
    }
}
