// Copyright (c) 1996-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomPlate/GeomPlate_BuildAveragePlane.hxx/.cxx
// (the point cloud constructor, DefPlan, BasePlan and the accessors) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomPlate_BuildAveragePlane`: the average plane (or line) of a point cloud. The
//! constructor from normals is in `average_plane_normals`, `HalfSpace` in
//! `half_space`.
use super::live;
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::geom::{GeomLine, GeomPlane};
use rustycad_tkgeombase::geom_lib::axe_of_inertia;
use rustycad_tkmath::elslib::plane_parameters;
use rustycad_tkmath::gp::{Axis3, Direction3, Plane3, Point3, Vector3};
use rustycad_tkmath::math::{Jacobi, Matrix, Vector};
use std::f64::consts::PI;

/// The average plane of a point cloud, or its line when the cloud is linear, with the
/// parameter box of the points on it (`GeomPlate_BuildAveragePlane`).
///
/// The box stays at zero when no plane is found (uninitialized natively), and the
/// second constructor leaves the tolerance and the bound point count at zero.
/// `let bap = GeomPlateBuildAveragePlane::new(&points, 0, 1e-7, 1, 1)?; bap.plane()?;`
pub struct GeomPlateBuildAveragePlane {
    pub(super) pts: Handle<HArray1<Point3>>,
    pub(super) umax: f64,
    pub(super) vmax: f64,
    pub(super) vmin: f64,
    pub(super) umin: f64,
    pub(super) plane: Handle<GeomPlane>,
    pub(super) tol: f64,
    pub(super) line: Handle<GeomLine>,
    pub(super) ox: Vector3,
    pub(super) oy: Vector3,
    pub(super) g: Point3,
    pub(super) nb_bound_points: i32,
}

/// `myPts->Value(i)`, Standard_OutOfRange outside the points.
pub(super) fn point_at(pts: &HArray1<Point3>, index: i32) -> Result<Point3, Failure> {
    pts.array1()
        .value(index)
        .copied()
        .map_err(errors::array_bounds)
}

/// The sum of the cross products `(P(i-1) - P(1)) ^ (P(i) - P(1))` for `i` in
/// `3..=nb_bound_points`: the flux vector of the boundary polygon.
fn boundary_flux(pts: &HArray1<Point3>, nb_bound_points: i32) -> Result<Vector3, Failure> {
    let mut a = Vector3::new(0.0, 0.0, 0.0);
    for i in 3..=nb_bound_points {
        let first: Point3 = point_at(pts, 1)?;
        let b = Vector3::between(first, point_at(pts, i - 1)?);
        let c = Vector3::between(first, point_at(pts, i)?);
        let d: Vector3 = b ^ c;
        a.set_coordinates(a.x() + d.x(), a.y() + d.y(), a.z() + d.z());
    }
    Ok(a)
}

impl GeomPlateBuildAveragePlane {
    /// The average plane of `pts`, whose first `nb_bound_points` points are a boundary
    /// polygon (`GeomPlate_BuildAveragePlane(Pts, NbBoundPoints, Tol, POption,
    /// NOption)`): the inertia plane (`n_option` 1, replaced by the boundary flux plane
    /// when their normals differ by more than PI/3) or the flux plane (2); its axes by
    /// the plane (`p_option` 1) or the eigen vectors (2). A line when two eigenvalues
    /// are within `tol`. Standard_ConstructionError for degenerate axes, as natively.
    /// `GeomPlateBuildAveragePlane::new(&points, 0, 1e-7, 1, 1)?`.
    pub fn new(
        pts: &Handle<HArray1<Point3>>,
        nb_bound_points: i32,
        tol: f64,
        p_option: i32,
        n_option: i32,
    ) -> Result<Self, Failure> {
        let mut bap = Self::empty(pts);
        bap.tol = tol;
        bap.nb_bound_points = nb_bound_points;
        let oz: Vector3 = bap.def_plan(n_option)?;
        if oz.square_magnitude() > 0.0 {
            bap.make_plane(oz, p_option)?;
            bap.fill_box()?;
        }
        if bap.is_line() {
            bap.line = Handle::new(GeomLine::from_point(
                bap.g,
                Direction3::from_vector(bap.ox)?,
            ));
        }
        Ok(bap)
    }

    pub(super) fn empty(pts: &Handle<HArray1<Point3>>) -> Self {
        let zero = Vector3::new(0.0, 0.0, 0.0);
        Self {
            pts: pts.clone(),
            umax: 0.0,
            vmax: 0.0,
            vmin: 0.0,
            umin: 0.0,
            plane: Handle::default(),
            tol: 0.0,
            line: Handle::default(),
            ox: zero,
            oy: zero,
            g: Point3::default(),
            nb_bound_points: 0,
        }
    }

    /// The plane through the centroid normal to `oz`, with the axes of `p_option`.
    fn make_plane(&mut self, oz: Vector3, p_option: i32) -> Result<(), Failure> {
        if p_option == 1 {
            let plane = GeomPlane::from_point(self.g, Direction3::from_vector(oz)?)?;
            let pln: Plane3 = plane.pln();
            self.ox = Vector3::from(pln.x_axis().direction());
            self.oy = Vector3::from(pln.y_axis().direction());
            self.plane = Handle::new(plane);
            return Ok(());
        }
        self.base_plan(oz)?;
        let n_dir = Direction3::from_vector(self.ox ^ self.oy)?;
        let u_dir = Direction3::from_vector(self.ox)?;
        self.plane = Handle::new(GeomPlane::new(Axis3::new(self.g, n_dir, u_dir)?));
        Ok(())
    }

    /// The parameter box of the centroid and the points on the plane.
    fn fill_box(&mut self) -> Result<(), Failure> {
        let position: Axis3 = live(&self.plane, "the average plane")?.pln().position();
        (self.umax, self.vmax) = plane_parameters(position, self.g);
        (self.umin, self.vmin) = (self.umax, self.vmax);
        let pts = self.pts.clone();
        let pts = live(&pts, "the points")?;
        for i in 1..=pts.array1().length() {
            let (u, v) = plane_parameters(position, point_at(pts, i)?);
            self.widen(u, v);
        }
        Ok(())
    }

    /// Widens the box to `(u, v)`, in the native order of the tests.
    pub(super) fn widen(&mut self, u: f64, v: f64) {
        if self.umax < u {
            self.umax = u;
        }
        if self.umin > u {
            self.umin = u;
        }
        if self.vmax < v {
            self.vmax = v;
        }
        if self.vmin > v {
            self.vmin = v;
        }
    }

    /// The centroid of the points (`DefPlan`, first part).
    fn centroid(pts: &HArray1<Point3>) -> Result<Point3, Failure> {
        let nb: i32 = pts.array1().length();
        let mut gb = Point3::new(0.0, 0.0, 0.0);
        for i in 1..=nb {
            let p: Point3 = point_at(pts, i)?;
            gb.set_coordinates(gb.x() + p.x(), gb.y() + p.y(), gb.z() + p.z());
        }
        let n = f64::from(nb);
        Ok(Point3::new(gb.x() / n, gb.y() / n, gb.z() / n))
    }

    /// The normal of the average plane (`DefPlan`): of the inertia frame for
    /// `n_option` 1, of the boundary flux for 2, null otherwise.
    fn def_plan(&mut self, n_option: i32) -> Result<Vector3, Failure> {
        let pts = self.pts.clone();
        let pts = live(&pts, "the points")?;
        self.g = Self::centroid(pts)?;
        match n_option {
            1 => self.inertia_normal(pts),
            2 => boundary_flux(pts, self.nb_bound_points),
            _ => Ok(Vector3::new(0.0, 0.0, 0.0)),
        }
    }

    /// The inertia normal, replaced by the boundary flux when the cloud has inner
    /// points and the two normals are more than PI/3 apart.
    fn inertia_normal(&mut self, pts: &HArray1<Point3>) -> Result<Vector3, Failure> {
        let inertia = axe_of_inertia(&pts.array1(), self.tol)?;
        self.ox = Vector3::from(inertia.axe.x_direction());
        self.oy = Vector3::from(inertia.axe.y_direction());
        let oz = Vector3::from(inertia.axe.direction());
        if self.nb_bound_points == 0 || pts.array1().length() == self.nb_bound_points {
            return Ok(oz);
        }
        let oz1: Vector3 = boundary_flux(pts, self.nb_bound_points)?;
        let mut the_angle: f64 = oz.angle(oz1)?;
        if the_angle > PI / 2.0 {
            the_angle = PI - the_angle;
        }
        Ok(if the_angle > PI / 3.0 { oz1 } else { oz })
    }

    /// The scatter matrix of the points projected on the plane normal to `oz`.
    fn projected_scatter(&self, oz: Vector3) -> Result<Matrix<'static>, Failure> {
        let mut m: Matrix<'static> =
            Matrix::with_bounds(1, 3, 1, 3).map_err(errors::array_bounds)?;
        m.init(0.0);
        let pts = live(&self.pts, "the points")?;
        let g = self.g;
        for i in 1..=pts.array1().length() {
            let p: Point3 = point_at(pts, i)?;
            let (px, py, pz) = (p.x() - g.x(), p.y() - g.y(), p.z() - g.z());
            let mut scal: f64 = px * oz.x() + py * oz.y() + pz * oz.z();
            scal /= oz.x() * oz.x() + oz.y() * oz.y() + oz.z() * oz.z();
            let proj = [px - scal * oz.x(), py - scal * oz.y(), pz - scal * oz.z()];
            m[(1, 1)] += proj[0] * proj[0];
            m[(2, 2)] += proj[1] * proj[1];
            m[(3, 3)] += proj[2] * proj[2];
            m[(1, 2)] += proj[0] * proj[1];
            m[(1, 3)] += proj[0] * proj[2];
            m[(2, 3)] += proj[1] * proj[2];
        }
        (m[(2, 1)], m[(3, 1)], m[(3, 2)]) = (m[(1, 2)], m[(1, 3)], m[(2, 3)]);
        Ok(m)
    }

    /// The axes of the plane normal to `oz` from the eigen vectors of the scatter
    /// (`BasePlan`): X along the greatest eigenvalue, Y along the middle one, or
    /// `oz ^ X` when two eigenvalues are within the tolerance.
    fn base_plan(&mut self, oz: Vector3) -> Result<(), Failure> {
        let j = Jacobi::new(&self.projected_scatter(oz)?)?;
        let n: [f64; 3] = [j.value(1)?, j.value(2)?, j.value(3)?];
        let (_, m2, m3) = eigen_order(n);
        let mut v2: Vector<'static> = Vector::with_bounds(1, 3)?;
        let mut v3: Vector<'static> = Vector::with_bounds(1, 3)?;
        j.vector(m2, &mut v2)?;
        j.vector(m3, &mut v3)?;
        let small = |x: f64| x.abs() <= self.tol;
        let (s1, s2, s3) = (small(n[0]), small(n[1]), small(n[2]));
        self.ox.set_coordinates(v3[1], v3[2], v3[3]);
        self.oy = if (s1 && s2) || (s2 && s3) || (s1 && s3) {
            oz ^ self.ox
        } else {
            Vector3::new(v2[1], v2[2], v2[3])
        };
        Ok(())
    }

    /// The average plane (`Plane`); Standard_NoSuchObject when the result is a line.
    /// `bap.plane()?`.
    pub fn plane(&self) -> Result<Handle<GeomPlane>, Failure> {
        if self.is_line() {
            return Err(errors::no_such_object(
                "Cannot use the function 'GeomPlate_BuildAveragePlane::Plane()', the Object is a 'Geom_Line'",
            ));
        }
        Ok(self.plane.clone())
    }

    /// The average line (`Line`); Standard_NoSuchObject when the result is a plane.
    /// `bap.line()?`.
    pub fn line(&self) -> Result<Handle<GeomLine>, Failure> {
        if self.is_plane() {
            return Err(errors::no_such_object(
                "Cannot use the function 'GeomPlate_BuildAveragePlane::Line()', the Object is a 'Geom_Plane'",
            ));
        }
        Ok(self.line.clone())
    }

    /// Whether the axes span a plane (`IsPlane`); `bap.is_plane()`.
    pub fn is_plane(&self) -> bool {
        (self.ox ^ self.oy).square_magnitude() != 0.0
    }

    /// Whether the axes are parallel or null: a line (`IsLine`); `bap.is_line()`.
    pub fn is_line(&self) -> bool {
        (self.ox ^ self.oy).square_magnitude() == 0.0
    }

    /// The parameter box `(Umin, Umax, Vmin, Vmax)` of the points projected on the
    /// plane (`MinMaxBox`); `let (u1, u2, v1, v2) = bap.min_max_box();`.
    pub fn min_max_box(&self) -> (f64, f64, f64, f64) {
        (self.umin, self.umax, self.vmin, self.vmax)
    }
}

/// The eigenvalue indices in increasing order, the first of equal ones first, as
/// the native `std::min` comparisons choose them.
fn eigen_order(n: [f64; 3]) -> (i32, i32, i32) {
    let r1: f64 = std_min(std_min(n[0], n[1]), n[2]);
    let pick = |a: usize, b: usize| -> (i32, i32) {
        let r2: f64 = std_min(n[a], n[b]);
        let (ia, ib) = ((a + 1) as i32, (b + 1) as i32);
        if r2 == n[a] { (ia, ib) } else { (ib, ia) }
    };
    if r1 == n[0] {
        let (m2, m3) = pick(1, 2);
        return (1, m2, m3);
    }
    if r1 == n[1] {
        let (m2, m3) = pick(0, 2);
        return (2, m2, m3);
    }
    let (m2, m3) = pick(0, 1);
    (3, m2, m3)
}

/// `std::min(a, b)`: `b` only when `b < a`.
fn std_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}
