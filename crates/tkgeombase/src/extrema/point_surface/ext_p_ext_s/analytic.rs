// Rust translation of the analytical part of Extrema_ExtPExtS::Perform and of
// Extrema_ExtPExtS::MakePreciser at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The analytical case of Extrema_ExtPExtS: the extrema of the distance from the
//! projection of the point to the conic in its plane. When the extrusion is normal
//! to that plane they are the extrema; otherwise each one (and its symmetric
//! parameter) starts a math_FunctionSetRoot on the surface.
use super::{ExtPExtS, NB_EXT_MAX, get_value, is_original_pnt, perform_ext_p_el_c, project_pnt};
use crate::extrema::ExtPElC;
use crate::extrema::pon::POnSurf;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3, Vector3};
use rustycad_tkmath::math::{FunctionSetRoot, Vector};
use std::f64::consts::PI;

/// The data of one analytical Perform: the point, its orthogonal section, the
/// solver bounds and tolerances.
struct Section {
    point: Point3,
    ortog: Axis2,
    dir: Direction3,
    tol: Vector<'static>,
    uv_inf: Vector<'static>,
    uv_sup: Vector<'static>,
}

// A 1-based math_Vector of two values.
fn pair(first: f64, second: f64) -> Result<Vector<'static>, Failure> {
    let mut vector: Vector<'static> = Vector::with_bounds(1, 2)?;
    (vector[1], vector[2]) = (first, second);
    Ok(vector)
}

impl ExtPExtS {
    /// The analytical Perform.
    pub(super) fn perform_analytic(&mut self, point: Point3) -> Result<(), Failure> {
        let dir: Direction3 = self.direction_dir()?;
        let pp: Point3 = project_pnt(&self.position, dir, point);
        let mut an_ext = ExtPElC::new();
        perform_ext_p_el_c(&mut an_ext, pp, self.curve()?, self.tolu)?;
        if !an_ext.is_done() {
            return Ok(());
        }
        let section = Section {
            point,
            ortog: Axis2::new(point, dir)?,
            dir,
            tol: pair(self.tolu, self.tolv)?,
            uv_inf: pair(self.uinf, self.vinf)?,
            uv_sup: pair(self.usup, self.vsup)?,
        };
        let is_simple_case: bool = self.direction.is_parallel(
            Vector3::from(self.position.direction()),
            precision::angular(),
        )?;
        for i in 1..=an_ext.nb_ext()? {
            if self.treat_curve_extremum(&an_ext, i, &section, is_simple_case)? {
                break;
            }
        }
        self.done = true;
        Ok(())
    }

    // One extremum on the conic; true when the arrays are full.
    fn treat_curve_extremum(
        &mut self,
        an_ext: &ExtPElC,
        i: i32,
        section: &Section,
        is_simple_case: bool,
    ) -> Result<bool, Failure> {
        let poc = an_ext.point(i)?;
        let mut u: f64 = poc.parameter();
        // OCC17194 (jgv, 23.12.2008).
        if self.curve()?.is_periodic()? {
            u = elclib::adjust_periodic(
                self.uinf,
                self.uinf + 2. * PI,
                precision::p_confusion(),
                u,
                u,
            )
            .0;
        }
        let e: Point3 = *poc.value();
        let pe: Point3 = project_pnt(&section.ortog, section.dir, e);
        if is_simple_case {
            let v: f64 = Vector3::between(e, pe).dot(self.direction);
            self.push(POnSurf::new(u, v, pe), an_ext.square_distance(i)?);
            return Ok(self.nb_ext == NB_EXT_MAX);
        }
        self.f.set_point(section.point);
        let is_min: bool = an_ext.is_min(i)?;
        let mut solver = FunctionSetRoot::new(&self.f, &section.tol, 100)?;
        self.solve_from(&mut u, is_min, section, &mut solver)?;
        if self.keep_new_roots(false)? {
            return Ok(true);
        }
        // Try the symmetric point; SetPoint clears the previous solutions.
        self.f.set_point(section.point);
        u *= -1.;
        self.solve_from(&mut u, is_min, section, &mut solver)?;
        self.keep_new_roots(true)
    }

    // MakePreciser from `u`, then math_FunctionSetRoot from `(u, v)` where `v` is the
    // extrusion parameter of the curve point in the section (natively one solver
    // object serves the start and its symmetric one).
    fn solve_from(
        &mut self,
        u: &mut f64,
        is_min: bool,
        section: &Section,
        solver: &mut FunctionSetRoot,
    ) -> Result<(), Failure> {
        self.make_preciser(u, section.point, is_min, &section.ortog, section.dir)?;
        let e: Point3 = get_value(*u, self.curve()?)?;
        let pe: Point3 = project_pnt(&section.ortog, section.dir, e);
        let v: f64 = Vector3::between(e, pe).dot(self.direction);
        let uv: Vector<'static> = pair(*u, v)?;
        solver.perform_bounded(&mut self.f, &uv, &section.uv_inf, &section.uv_sup, false)?;
        self.f.take_failure()
    }

    // The roots the function saved that are not kept yet (after the check of the
    // symmetric start, which drops a far root whose F is not small); true when the
    // arrays are full.
    fn keep_new_roots(&mut self, check_roots: bool) -> Result<bool, Failure> {
        for k in 1..=self.f.nb_ext() {
            if check_roots && !self.is_valid_root(k)? {
                continue;
            }
            let root: POnSurf = self.f.point(k)?;
            if is_original_pnt(root.value(), &self.points[..self.nb_ext as usize]) {
                let sq_dist: f64 = self.f.square_distance(k)?;
                self.push(root, sq_dist);
                if self.nb_ext == NB_EXT_MAX {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    // The additional check of a root of the symmetric start ("FSR sometimes is
    // wrong when starting point is far from solution"): `|F| / dist / |D1|` not above
    // PConfusion in each direction. Evaluating F moves the function's current point.
    // The negated test keeps the native NaN semantics.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    fn is_valid_root(&mut self, k: i32) -> Result<bool, Failure> {
        let sq_dist: f64 = self.f.square_distance(k)?;
        if !(sq_dist > precision::confusion() * precision::confusion()) {
            return Ok(true);
        }
        let dist: f64 = sq_dist.sqrt();
        let (u, v) = self.f.point(k)?.parameter();
        let uv: Vector<'static> = pair(u, v)?;
        let mut vals: Vector<'static> = Vector::with_bounds(1, 2)?;
        self.f.value(&uv, &mut vals)?;
        let d1 = super::ExtPExtS::adaptor(&self.s)?.d1(u, v)?;
        let (mdu, mdv) = (d1.d1u.magnitude(), d1.d1v.magnitude());
        let (fu, fv) = (vals[1].abs(), vals[2].abs());
        if mdu > precision::p_confusion() && fu / dist / mdu > precision::p_confusion() {
            return Ok(false);
        }
        if mdv > precision::p_confusion() && fv / dist / mdv > precision::p_confusion() {
            return Ok(false);
        }
        Ok(true)
    }

    // Appends one extremum at `myNbExt`.
    fn push(&mut self, point: POnSurf, sq_dist: f64) {
        let index: usize = self.nb_ext as usize;
        self.points[index] = point;
        self.sq_dist[index] = sq_dist;
        self.nb_ext += 1;
    }

    // `MakePreciser`: `u` clamped to the range, or moved by steps of a thirtieth of
    // the range while the distance to the projected curve point decreases (a
    // minimum) or increases (a maximum).
    fn make_preciser(
        &self,
        u: &mut f64,
        point: Point3,
        is_min: bool,
        ortog: &Axis2,
        dir: Direction3,
    ) -> Result<(), Failure> {
        if *u > self.usup {
            *u = self.usup;
            return Ok(());
        }
        if *u < self.uinf {
            *u = self.uinf;
            return Ok(());
        }
        let curve = self.curve()?;
        let projected = |t: f64| -> Result<Point3, Failure> {
            Ok(project_pnt(ortog, dir, get_value(t, curve)?))
        };
        let mut step: f64 = (self.usup - self.uinf) / 30.;
        let pe: Point3 = projected(*u)?;
        let pprev: Point3 = projected(*u - step)?;
        let pnext: Point3 = projected(*u + step)?;
        let mut d2e: f64 = point.square_distance(&pe);
        let mut d2next: f64 = point.square_distance(&pnext);
        let d2prev: f64 = point.square_distance(&pprev);
        let mut not_found: bool = if is_min {
            d2e > d2prev || d2e > d2next
        } else {
            d2e < d2prev || d2e < d2next
        };
        if not_found && (d2e < d2next && is_min) {
            step = -step;
            d2next = d2prev;
        }
        while not_found {
            *u += step;
            if *u > self.usup {
                *u = self.usup;
                break;
            }
            if *u < self.uinf {
                *u = self.uinf;
                break;
            }
            d2e = d2next;
            let pnext: Point3 = projected(*u + step)?;
            d2next = point.square_distance(&pnext);
            not_found = if is_min { d2e > d2next } else { d2e < d2next };
        }
        Ok(())
    }
}
