// Created on: 1993-04-07
// Created by: Laurent BUCHARD
// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of IntCurveSurface_ThePolyhedronOfHInter.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurveSurface_ThePolyhedronOfHInter`: a surface patch sampled on a grid and
//! cut into two triangles per cell.
use super::failures::out_of_bounds;
use super::polyhedron_utils::{self as utils, PolyhedronGrid};
use super::tri_connex::tri_connex;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom_grid_eval::GeomGridEvalSurface;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Point3, Xyz};

/// `IntCurveSurface_ThePolyhedronOfHInter`: the grid points with their (u, v) and
/// boundary flags, the box, the box of each triangle, the deflection of the
/// triangles and of the borders, and four singularity flags.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
/// use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
/// use rustycad_tkgeomalgo::int_curve_surface::ThePolyhedronOfHInter;
/// use rustycad_tkmath::gp::Axis3;
/// let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
/// let s = Handle::new(GeomAdaptorSurface::with_bounds(&plane, (-1.0, 1.0, -1.0, 1.0)).unwrap()).cast();
/// let polyhedron = ThePolyhedronOfHInter::new(&s, 4, 4, -1.0, -1.0, 1.0, 1.0).unwrap();
/// assert_eq!(polyhedron.nb_triangles(), 32);
/// ```
#[derive(Clone, Debug)]
pub struct ThePolyhedronOfHInter {
    nbdelta_u: i32,
    nbdelta_v: i32,
    the_bnd: BoundingBox3,
    the_components_bnd: Array1<'static, BoundingBox3>,
    the_deflection: f64,
    grid: PolyhedronGrid,
    u_min_singular: bool,
    u_max_singular: bool,
    v_min_singular: bool,
    v_max_singular: bool,
    the_border_deflection: f64,
}

/// `nbDeltaFromParameters`: the subdivision count of a parameter array, which
/// needs at least two values.
fn nb_delta_from_parameters(pars: &Array1<'_, f64>, name: &str) -> Result<i32, Failure> {
    if pars.length() < 2 {
        return Err(Failure::out_of_range(&format!(
            "IntCurveSurface_ThePolyhedronOfHInter::IntCurveSurface_ThePolyhedronOfHInter() - {name} \
             must contain at least two values, got {}",
            pars.length()
        )));
    }
    Ok(pars.length() - 1)
}

impl ThePolyhedronOfHInter {
    fn allocated(nbdelta_u: i32, nbdelta_v: i32) -> Self {
        Self {
            nbdelta_u,
            nbdelta_v,
            the_bnd: BoundingBox3::default(),
            the_components_bnd: Array1::default(),
            the_deflection: epsilon(100.0),
            grid: utils::allocate_arrays(nbdelta_u, nbdelta_v),
            u_min_singular: false,
            u_max_singular: false,
            v_min_singular: false,
            v_max_singular: false,
            the_border_deflection: 0.0,
        }
    }
    /// Constructor (Surface, nbdU, nbdV, U1, V1, U2, V2): a uniform grid of at least
    /// 3 x 3 cells; `ThePolyhedronOfHInter::new(&s, 10, 10, 0.0, 0.0, 1.0, 1.0)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        surface: &Adaptor3dSurfaceHandle,
        nbd_u: i32,
        nbd_v: i32,
        u1: f64,
        v1: f64,
        u2: f64,
        v2: f64,
    ) -> Result<Self, Failure> {
        let mut polyhedron = Self::allocated(nbd_u.max(3), nbd_v.max(3));
        polyhedron.init(surface, [u1, v1, u2, v2])?;
        Ok(polyhedron)
    }
    /// Constructor (Surface, Upars, Vpars): the grid at the given parameters;
    /// Standard_OutOfRange when an array has fewer than two values;
    /// `ThePolyhedronOfHInter::with_parameters(&s, &upars, &vpars)?`.
    pub fn with_parameters(
        surface: &Adaptor3dSurfaceHandle,
        upars: &Array1<'_, f64>,
        vpars: &Array1<'_, f64>,
    ) -> Result<Self, Failure> {
        let nbdu: i32 = nb_delta_from_parameters(upars, "Upars")?;
        let nbdv: i32 = nb_delta_from_parameters(vpars, "Vpars")?;
        let mut polyhedron = Self::allocated(nbdu, nbdv);
        polyhedron.init_with_params(surface, upars, vpars)?;
        Ok(polyhedron)
    }
    /// `Init(Surface, U1, V1, U2, V2)` (protected).
    fn init(&mut self, surface: &Adaptor3dSurfaceHandle, bounds: [f64; 4]) -> Result<(), Failure> {
        let eval = GeomGridEvalSurface::from_adaptor(&**surface)?;
        let sizes = (self.nbdelta_u, self.nbdelta_v);
        utils::init_uniform(&eval, bounds, sizes, &mut self.grid, &mut self.the_bnd)?;
        self.finish(surface, &eval, bounds)
    }
    /// `Init(Surface, Upars, Vpars)` (protected).
    fn init_with_params(
        &mut self,
        surface: &Adaptor3dSurfaceHandle,
        upars: &Array1<'_, f64>,
        vpars: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let eval = GeomGridEvalSurface::from_adaptor(&**surface)?;
        let sizes = (self.nbdelta_u, self.nbdelta_v);
        utils::init_with_params(
            &eval,
            (upars, vpars),
            sizes,
            &mut self.grid,
            &mut self.the_bnd,
        )?;
        let bound = |a: &Array1<'_, f64>, i: i32| a[i];
        let bounds = [
            bound(upars, upars.lower()),
            bound(vpars, vpars.lower()),
            bound(upars, upars.upper()),
            bound(vpars, vpars.upper()),
        ];
        self.finish(surface, &eval, bounds)
    }
    /// The common end of both Init: deflection (1.2 times the largest), boxes and
    /// border deflection.
    fn finish(
        &mut self,
        surface: &Adaptor3dSurfaceHandle,
        eval: &GeomGridEvalSurface<'_>,
        bounds: [f64; 4],
    ) -> Result<(), Failure> {
        let tol: f64 = utils::compute_max_deflection(
            &**surface,
            &self.grid,
            self.nbdelta_v,
            self.nb_triangles(),
        )?;
        self.set_deflection_over_estimation(tol * 1.2);
        self.fill_bounding()?;
        let sizes = (self.nbdelta_u, self.nbdelta_v);
        self.the_border_deflection = utils::compute_max_border_deflection(eval, bounds, sizes)?;
        Ok(())
    }
    /// `Destroy()`: releases the point arrays (the destructor calls it); any later
    /// point access fails with Standard_OutOfRange; `polyhedron.destroy()`.
    pub fn destroy(&mut self) {
        utils::destroy(&mut self.grid);
    }
    /// `DeflectionOverEstimation(flec)`: the deflection, at least 1e-4, and the box
    /// enlarged by it; `polyhedron.set_deflection_over_estimation(0.01)`.
    pub fn set_deflection_over_estimation(&mut self, flec: f64) {
        utils::set_deflection_over_estimation(flec, &mut self.the_deflection, &mut self.the_bnd);
    }
    /// `UMinSingularity(Sing)`; `polyhedron.u_min_singularity(true)`.
    pub fn u_min_singularity(&mut self, sing: bool) {
        self.u_min_singular = sing;
    }
    /// `UMaxSingularity(Sing)`; `polyhedron.u_max_singularity(true)`.
    pub fn u_max_singularity(&mut self, sing: bool) {
        self.u_max_singular = sing;
    }
    /// `VMinSingularity(Sing)`; `polyhedron.v_min_singularity(true)`.
    pub fn v_min_singularity(&mut self, sing: bool) {
        self.v_min_singular = sing;
    }
    /// `VMaxSingularity(Sing)`; `polyhedron.v_max_singularity(true)`.
    pub fn v_max_singularity(&mut self, sing: bool) {
        self.v_max_singular = sing;
    }
    /// `Size(nbdu, nbdv)`: the cell counts; `let (nbdu, nbdv) = polyhedron.size();`.
    pub fn size(&self) -> (i32, i32) {
        (self.nbdelta_u, self.nbdelta_v)
    }
    /// `NbTriangles()`: `2 * nbdu * nbdv`; `polyhedron.nb_triangles()`.
    pub fn nb_triangles(&self) -> i32 {
        utils::nb_triangles(self.nbdelta_u, self.nbdelta_v)
    }
    /// `Triangle(Index, P1, P2, P3)`: the point indices of a triangle, by arithmetic
    /// (no range check natively); `polyhedron.triangle(1)`.
    pub fn triangle(&self, index: i32) -> [i32; 3] {
        utils::triangle(index, self.nbdelta_v)
    }
    /// `TriConnex(Triang, Pivot, Pedge, TriCon, OtherP)`: `(returned, TriCon,
    /// OtherP)`; `polyhedron.tri_connex(1, 1, 2)`.
    pub fn tri_connex(&self, triang: i32, pivot: i32, pedge: i32) -> (i32, i32, i32) {
        tri_connex(triang, pivot, pedge, (self.nbdelta_u, self.nbdelta_v))
    }
    /// `NbPoints()`: `(nbdu + 1) * (nbdv + 1)`; `polyhedron.nb_points()`.
    pub fn nb_points(&self) -> i32 {
        utils::nb_points(self.nbdelta_u, self.nbdelta_v)
    }
    /// `Point(thePnt, lig, col, U, V)`: does nothing natively; `polyhedron.set_point(p, 1, 1, 0.0, 0.0)`.
    pub fn set_point(&mut self, _pnt: Point3, _lig: i32, _col: i32, _u: f64, _v: f64) {}
    fn slot(&self, index: i32) -> Result<usize, Failure> {
        let len: usize = self.grid.pnts.len();
        let found = usize::try_from(index).ok().filter(|&at| at < len);
        found.ok_or_else(|| {
            out_of_bounds(
                "IntCurveSurface_ThePolyhedronOfHInter::Point",
                index,
                0,
                len as i32 - 1,
            )
        })
    }
    /// `Point(Index, U, V)`: the point with its parameters; index 0 is the origin with
    /// null parameters; Standard_OutOfRange past the grid (an out-of-bounds read
    /// natively); `let (p, u, v) = polyhedron.point_with_parameters(1)?;`.
    pub fn point_with_parameters(&self, index: i32) -> Result<(Point3, f64, f64), Failure> {
        let at: usize = self.slot(index)?;
        Ok((self.grid.pnts[at], self.grid.u[at], self.grid.v[at]))
    }
    /// `Point(Index)` and `Point(Index, P)`; `polyhedron.point(1)?`.
    pub fn point(&self, index: i32) -> Result<Point3, Failure> {
        Ok(self.grid.pnts[self.slot(index)?])
    }
    /// `Bounding()`: the box of the points; `polyhedron.bounding()`.
    pub fn bounding(&self) -> &BoundingBox3 {
        &self.the_bnd
    }
    /// `FillBounding()`: recomputes the box of each triangle with the current
    /// deflection; `polyhedron.fill_bounding()?`.
    pub fn fill_bounding(&mut self) -> Result<(), Failure> {
        let sizes = (self.nbdelta_u, self.nbdelta_v);
        self.the_components_bnd =
            utils::fill_bounding(&self.grid.pnts, sizes, self.the_deflection)?;
        Ok(())
    }
    /// `ComponentsBounding()`: the box of each triangle; `polyhedron.components_bounding()`.
    pub fn components_bounding(&self) -> &Array1<'static, BoundingBox3> {
        &self.the_components_bnd
    }
    /// `DeflectionOverEstimation()`; `polyhedron.deflection_over_estimation()`.
    pub fn deflection_over_estimation(&self) -> f64 {
        self.the_deflection
    }
    /// `HasUMinSingularity()`; `polyhedron.has_u_min_singularity()`.
    pub fn has_u_min_singularity(&self) -> bool {
        self.u_min_singular
    }
    /// `HasUMaxSingularity()`; `polyhedron.has_u_max_singularity()`.
    pub fn has_u_max_singularity(&self) -> bool {
        self.u_max_singular
    }
    /// `HasVMinSingularity()`; `polyhedron.has_v_min_singularity()`.
    pub fn has_v_min_singularity(&self) -> bool {
        self.v_min_singular
    }
    /// `HasVMaxSingularity()`; `polyhedron.has_v_max_singularity()`.
    pub fn has_v_max_singularity(&self) -> bool {
        self.v_max_singular
    }
    fn triangle_points(&self, triang: i32) -> Result<[Point3; 3], Failure> {
        let [i1, i2, i3] = self.triangle(triang);
        Ok([self.point(i1)?, self.point(i2)?, self.point(i3)?])
    }
    /// `PlaneEquation(Triang, NormalVector, PolarDistance)`;
    /// `let (normal, distance) = polyhedron.plane_equation(1)?;`.
    pub fn plane_equation(&self, triang: i32) -> Result<(Xyz, f64), Failure> {
        let [p1, p2, p3] = self.triangle_points(triang)?;
        Ok(utils::plane_equation(p1, p2, p3))
    }
    /// `Contain(Triang, ThePnt)`: whether the point is inside the triangle;
    /// `polyhedron.contain(1, p)?`.
    pub fn contain(&self, triang: i32, pnt: Point3) -> Result<bool, Failure> {
        let [p1, p2, p3] = self.triangle_points(triang)?;
        Ok(utils::contain(p1, p2, p3, pnt))
    }
    /// `Parameters(Index, U, V)`; `let (u, v) = polyhedron.parameters(1)?;`.
    pub fn parameters(&self, index: i32) -> Result<(f64, f64), Failure> {
        let at: usize = self.slot(index)?;
        Ok((self.grid.u[at], self.grid.v[at]))
    }
    /// `IsOnBound(Index1, Index2)`: whether the edge joins two boundary points of a
    /// grid line or column; `polyhedron.is_on_bound(1, 2)`.
    pub fn is_on_bound(&self, index1: i32, index2: i32) -> bool {
        utils::is_on_bound(
            index1,
            index2,
            &self.grid.is_on_bounds,
            (self.nbdelta_u, self.nbdelta_v),
        )
    }
    /// `GetBorderDeflection()`; `polyhedron.get_border_deflection()`.
    pub fn get_border_deflection(&self) -> f64 {
        self.the_border_deflection
    }
    /// `Dump()`: writes nothing natively; `polyhedron.dump()`.
    pub fn dump(&self) {}
}
