//! The end points of the runs off the seams: the neighbouring seam points, moved onto
//! the seam on the side of the run.
use super::boundary::{adjust_periodic, find_point, is_point_on_boundary};
use super::decomposition::{Decomposition, Runs};
use crate::int_patch::IntPatchPointLine;
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::gp::{self, Point2, Point3, Vector2};
use std::f64::consts::PI;

/// The run being extended: its points, the side (`j`: 0 before its first point, 1 after
/// its last one) and the seam point being moved.
struct RunEnd<'a> {
    list: &'a [i32],
    j: usize,
    point: IntSurfPntOn2S,
    new_p: IntSurfPntOn2S,
}

impl RunEnd<'_> {
    /// The point of the run next to the seam point (`aneighbourpointindex`).
    fn neighbour_index(&self) -> i32 {
        if self.j == 0 {
            self.list[0]
        } else {
            self.list[self.list.len() - 1]
        }
    }
    /// The next index into the run.
    fn step(&self, index: i32) -> i32 {
        if self.j == 0 { index + 1 } else { index - 1 }
    }
    /// Whether `index` is within the range of the run.
    fn within(&self, index: i32) -> bool {
        index <= self.list[self.list.len() - 1] && index >= self.list[0]
    }
}

/// The boundaries of one surface that the seam point lies on (`nbboundaries`,
/// `bIsUBoundary`, `bIsFirstBoundary`).
#[derive(Clone, Copy)]
struct OnBounds {
    count: i32,
    is_u: bool,
    is_first: bool,
}

/// The parameter bounds of the adaptor.
fn adaptor_bounds(surface: &GeomAdaptorSurface) -> Result<(f64, f64, f64, f64), Failure> {
    Ok((
        surface.first_u_parameter()?,
        surface.last_u_parameter()?,
        surface.first_v_parameter()?,
        surface.last_v_parameter()?,
    ))
}

/// The squared `gp::Resolution()`.
fn sq_resolution() -> f64 {
    gp::resolution() * gp::resolution()
}

impl Decomposition<'_> {
    /// The boundaries of `surface` that `(u, v)` lies on (the periodic parameters are
    /// adjusted first).
    fn on_bounds(
        &self,
        surface: &GeomAdaptorSurface,
        (u, v): (f64, f64),
    ) -> Result<OnBounds, Failure> {
        let tol: f64 = precision::confusion();
        let (umin, umax, vmin, vmax) = adaptor_bounds(surface)?;
        let mut on = OnBounds {
            count: 0,
            is_u: false,
            is_first: false,
        };
        for parit in 0..2 {
            let periodic: bool = if parit == 0 {
                surface.is_u_periodic()?
            } else {
                surface.is_v_periodic()?
            };
            let resolution: f64 = if parit == 0 {
                surface.u_resolution(tol)?
            } else {
                surface.v_resolution(tol)?
            };
            let (lower, upper, mut parameter) = if parit == 0 {
                (umin, umax, u)
            } else {
                (vmin, vmax, v)
            };
            if periodic {
                let period: f64 = if parit == 0 {
                    surface.u_period()?
                } else {
                    surface.v_period()?
                };
                parameter = adjust_periodic(parameter, lower, upper, period).0;
            }
            if let Some(first) = is_point_on_boundary(parameter, lower, upper, resolution) {
                on.is_u = parit == 0;
                on.is_first = first;
                on.count += 1;
            }
        }
        Ok(on)
    }

    /// `(U, V)` of the point `index` of the line on the surface `surfit`.
    fn params_on(&self, index: i32, surfit: usize) -> Result<(f64, f64), Failure> {
        Ok(self.wline.point(index)?.parameters_on_surface(surfit == 0))
    }

    /// The end points of the runs off the seams (`anArrayOfLineEnds`, indices into the
    /// returned point sequence), and that sequence (`aSeqOfPntOn2S`).
    pub(super) fn line_ends(
        &mut self,
        runs: &Runs,
    ) -> Result<(Vec<Vec<i32>>, Handle<IntSurfLineOn2S>), Failure> {
        let seq = Handle::new(IntSurfLineOn2S::new(None));
        let mut ends: Vec<Vec<i32>> = vec![Vec::new(); runs.count() as usize];
        for i in 1..=runs.count() {
            if runs.is_boundary(i) || runs.line(i).len() < 2 {
                continue;
            }
            let mut fl_index: Vec<i32> = Vec::new();
            for j in 0..2usize {
                let neighbour: i32 = if j == 0 { i - 1 } else { i + 1 };
                if neighbour < 1 || neighbour > runs.count() || !runs.is_boundary(neighbour) {
                    continue;
                }
                let nl: &[i32] = runs.line(neighbour);
                let index: i32 = if j == 0 { nl[nl.len() - 1] } else { nl[0] };
                let point: IntSurfPntOn2S = self.wline.point(index)?;
                let mut end = RunEnd {
                    list: runs.line(i),
                    j,
                    point,
                    new_p: point,
                };
                for surfit in 0..2usize {
                    self.move_end(&mut end, surfit)?;
                }
                seq.add(&end.new_p);
                fl_index.push(seq.nb_points());
            }
            ends[(i - 1) as usize] = fl_index;
        }
        Ok((ends, seq))
    }

    /// The seam point moved on the surface `surfit`: onto the other side of the seam
    /// when the run comes from there, else to where the run meets the boundary.
    fn move_end(&mut self, end: &mut RunEnd<'_>, surfit: usize) -> Result<(), Failure> {
        let surface: &GeomAdaptorSurface = self.surfaces[surfit];
        let uv: (f64, f64) = end.new_p.parameters_on_surface(surfit == 0);
        let on: OnBounds = self.on_bounds(surface, uv)?;
        let compute_line_end: bool = match on.count {
            2 => true,
            1 => self.one_boundary(end, surfit, uv, on)?,
            _ => false,
        };
        if compute_line_end {
            self.compute_line_end(end, surfit)?;
        }
        Ok(())
    }

    /// The case of one boundary: false when the run continues across a periodic seam
    /// (the point is then moved to the other side when that is its image), true when
    /// the end is to be computed.
    fn one_boundary(
        &self,
        end: &mut RunEnd<'_>,
        surfit: usize,
        (u, v): (f64, f64),
        on: OnBounds,
    ) -> Result<bool, Failure> {
        let surface: &GeomAdaptorSurface = self.surfaces[surfit];
        let periodic: bool = if on.is_u {
            surface.is_u_periodic()?
        } else {
            surface.is_v_periodic()?
        };
        if !periodic {
            return Ok(false);
        }
        let (umin, umax, vmin, vmax) = adaptor_bounds(surface)?;
        let (lower, upper) = if on.is_u { (umin, umax) } else { (vmin, vmax) };
        let period: f64 = if on.is_u {
            surface.u_period()?
        } else {
            surface.v_period()?
        };
        let parameter: f64 = if on.is_u { u } else { v };
        let (adjusted, offset) = adjust_periodic(parameter, lower, upper, period);
        let adist: f64 = if on.is_first {
            (adjusted - lower).abs()
        } else {
            (adjusted - upper).abs()
        };
        let mut another_par: f64 = if on.is_first {
            upper - adist
        } else {
            lower + adist
        };
        another_par += offset;
        let (nu1, nv1) = self.params_on(end.neighbour_index(), surfit)?;
        let adist1: f64 = if on.is_u {
            (nu1 - u).abs()
        } else {
            (nv1 - v).abs()
        };
        let adist2: f64 = if on.is_u {
            (nu1 - another_par).abs()
        } else {
            (nv1 - another_par).abs()
        };
        let (anew_u, anew_v) = if on.is_u {
            (another_par, v)
        } else {
            (u, another_par)
        };
        let n1 = Point2::new(nu1, nv1);
        let mut check_angle1 = false;
        let mut check_angle2 = false;
        let mut new_vec = Vector2::new(0.0, 0.0);
        if (adist1 - adist2) > precision::p_confusion() && adist2 < (period / 4.0) {
            check_angle1 = true;
            new_vec = Vector2::between(n1, Point2::new(anew_u, anew_v));
            if new_vec.square_magnitude() < sq_resolution() {
                end.new_p.set_parameters_on(surfit == 0, anew_u, anew_v);
                check_angle1 = false;
            }
        } else if adist1 < (period / 4.0) {
            check_angle2 = true;
            new_vec = Vector2::between(n1, Point2::new(u, v));
            if new_vec.square_magnitude() < sq_resolution() {
                check_angle2 = false;
            }
        }
        if !(check_angle1 || check_angle2) {
            return Ok(true);
        }
        self.check_angles(
            end,
            surfit,
            (n1, new_vec),
            (check_angle1, check_angle2),
            (anew_u, anew_v),
        )
    }

    /// The direction test of `one_boundary`: the first run point that differs from the
    /// neighbour point gives the direction of the run; when the new direction follows
    /// it, the end is not computed (and the point is moved across the seam when the
    /// three images coincide).
    fn check_angles(
        &self,
        end: &mut RunEnd<'_>,
        surfit: usize,
        (n1, new_vec): (Point2, Vector2),
        (check_angle1, check_angle2): (bool, bool),
        (anew_u, anew_v): (f64, f64),
    ) -> Result<bool, Failure> {
        let mut compute_line_end = true;
        // There are at least two points in the run.
        let mut index_other: i32 = end.neighbour_index();
        while end.within(index_other) {
            index_other = end.step(index_other);
            let (nu2, nv2) = self.params_on(index_other, surfit)?;
            let vec_old = Vector2::between(Point2::new(nu2, nv2), n1);
            if vec_old.square_magnitude() <= sq_resolution() {
                continue;
            }
            let angle: f64 = new_vec.angle(vec_old)?;
            if angle.abs() < PI * 0.25 && new_vec.dot(vec_old) > 0.0 {
                if check_angle1 && self.images_coincide(end, surfit, (anew_u, anew_v))? {
                    compute_line_end = false;
                    end.new_p.set_parameters_on(surfit == 0, anew_u, anew_v);
                }
                if check_angle2 {
                    compute_line_end = false;
                }
            }
            break;
        }
        Ok(compute_line_end)
    }

    /// Whether the seam point and the images of the moved parameters on both surfaces
    /// coincide within `Precision::Confusion()`.
    fn images_coincide(
        &self,
        end: &RunEnd<'_>,
        surfit: usize,
        (anew_u, anew_v): (f64, f64),
    ) -> Result<bool, Failure> {
        let tol: f64 = precision::confusion();
        let mut tmp: IntSurfPntOn2S = end.new_p;
        tmp.set_parameters_on(surfit == 0, anew_u, anew_v);
        let (u1, v1, u2, v2) = tmp.parameters();
        let p1: Point3 = self.surfaces[0].value(u1, v1)?;
        let p2: Point3 = self.surfaces[1].value(u2, v2)?;
        let p0: Point3 = end.point.value();
        Ok(p0.is_equal(&p1, tol) && p0.is_equal(&p2, tol) && p1.is_equal(&p2, tol))
    }

    /// The end of the run on the boundary: the ray of its last distinct points meets the
    /// boundary; its image is projected on the other surface and kept when the
    /// projection is within `aTolSum`.
    fn compute_line_end(&mut self, end: &mut RunEnd<'_>, surfit: usize) -> Result<(), Failure> {
        let index1: i32 = end.neighbour_index();
        let (nu1, nv1) = self.params_on(index1, surfit)?;
        let ap1 = Point2::new(nu1, nv1);
        let mut ap2 = ap1;
        let mut index2: i32 = index1;
        while end.within(index2) {
            index2 = end.step(index2);
            let (nu2, nv2) = self.params_on(index2, surfit)?;
            ap2 = Point2::new(nu2, nv2);
            if ap1.square_distance(ap2) > sq_resolution() {
                break;
            }
        }
        let surface: &GeomAdaptorSurface = self.surfaces[surfit];
        let Some(new_point) = find_point(ap2, ap1, adaptor_bounds(surface)?)? else {
            return Ok(());
        };
        let p3d: Point3 = surface.value(new_point.x(), new_point.y())?;
        let projector = &mut self.projectors[1 - surfit];
        projector.perform(p3d)?;
        if !(projector.is_done() && projector.lower_distance()? < self.tol_sum) {
            return Ok(());
        }
        let (found_u, found_v) = projector.lower_distance_parameters()?;
        if surfit == 0 {
            end.new_p
                .set_value_all(p3d, new_point.x(), new_point.y(), found_u, found_v);
        } else {
            end.new_p
                .set_value_all(p3d, found_u, found_v, new_point.x(), new_point.y());
        }
        Ok(())
    }
}
