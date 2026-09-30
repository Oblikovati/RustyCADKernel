// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_Polyhedron.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_Polyhedron`: the linear approximation of a surface patch by a grid of
//! triangles (two per cell), with the boxes of the triangles.
use super::h_inter_tool::IntPatchHInterTool;
use crate::int_curve_surface::polyhedron_utils::{self as utils, PolyhedronGrid};
use crate::int_curve_surface::tri_connex;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom_grid_eval::GeomGridEvalSurface;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{self, Point3, Xyz};

/// `LONGUEUR_MINI_EDGE_TRIANGLE`: the squared length below which an edge is null.
const LONGUEUR_MINI_EDGE_TRIANGLE: f64 = 1e-14; // tol:squared-length
/// `DEFLECTION_COEFF`: the safety factor of the computed deflection.
const DEFLECTION_COEFF: f64 = 1.1;
/// `NBMAXUV`: the largest automatic subdivision count.
const NBMAXUV: i32 = 30;

/// `IntPatch_Polyhedron`: the grid points of a surface patch with their parameters,
/// the box of the points, the box of each triangle and the deflection.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
/// use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
/// use rustycad_tkgeomalgo::int_patch::IntPatchPolyhedron;
/// use rustycad_tkmath::gp::Axis3;
/// let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
/// let s = Handle::new(GeomAdaptorSurface::with_bounds(&plane, (-1.0, 1.0, -1.0, 1.0)).unwrap()).cast();
/// let polyhedron = IntPatchPolyhedron::with_subdivisions(&s, 0, 2).unwrap();
/// assert_eq!(polyhedron.size(), (1, 2));
/// assert_eq!(polyhedron.nb_triangles(), 4);
/// ```
#[derive(Clone, Debug)]
pub struct IntPatchPolyhedron {
    the_bnd: BoundingBox3,
    the_components_bnd: Array1<'static, BoundingBox3>,
    the_deflection: f64,
    nbdelta_u: i32,
    nbdelta_v: i32,
    grid: PolyhedronGrid,
    // UMinSingular...VMaxSingular: always false (IntPatch_HInterTool); no member reads
    // them (their accessors are declared but defined nowhere natively).
    singular: [bool; 4],
}

/// The automatic subdivision: the tool sample count, at most `NBMAXUV`.
fn nb_samples(surface: &Adaptor3dSurfaceHandle) -> Result<(i32, i32), Failure> {
    let (u0, u1) = (surface.first_u_parameter()?, surface.last_u_parameter()?);
    let (v0, v1) = (surface.first_v_parameter()?, surface.last_v_parameter()?);
    let nbpu: i32 = IntPatchHInterTool::nb_samples_u(surface, u0, u1)?;
    let nbpv: i32 = IntPatchHInterTool::nb_samples_v(surface, v0, v1)?;
    Ok((nbpu.min(NBMAXUV), nbpv.min(NBMAXUV)))
}

impl IntPatchPolyhedron {
    /// `IntPatch_Polyhedron(Surface)`: the subdivision is the tool sample count of each
    /// direction, at most 30. `IntPatchPolyhedron::new(&surface)?`.
    pub fn new(surface: &Adaptor3dSurfaceHandle) -> Result<Self, Failure> {
        let (nbu, nbv) = nb_samples(surface)?;
        Self::build(surface, nbu, nbv)
    }

    /// `IntPatch_Polyhedron(Surface, nbdU, nbdV)`: at least one cell in each direction
    /// (bug #51). `IntPatchPolyhedron::with_subdivisions(&surface, 10, 10)?`.
    pub fn with_subdivisions(
        surface: &Adaptor3dSurfaceHandle,
        nbdu: i32,
        nbdv: i32,
    ) -> Result<Self, Failure> {
        Self::build(surface, nbdu.max(1), nbdv.max(1))
    }

    fn build(surface: &Adaptor3dSurfaceHandle, nbu: i32, nbv: i32) -> Result<Self, Failure> {
        let singular = [
            IntPatchHInterTool::singular_on_u_min(surface),
            IntPatchHInterTool::singular_on_u_max(surface),
            IntPatchHInterTool::singular_on_v_min(surface),
            IntPatchHInterTool::singular_on_v_max(surface),
        ];
        let mut polyhedron = Self {
            the_bnd: BoundingBox3::default(),
            the_components_bnd: Array1::default(),
            the_deflection: epsilon(100.0),
            nbdelta_u: nbu,
            nbdelta_v: nbv,
            grid: utils::allocate_arrays(nbu, nbv),
            singular,
        };
        polyhedron.sample(surface)?;
        Ok(polyhedron)
    }

    /// The grid evaluation, the deflection (1.1 times the largest one) and the boxes.
    fn sample(&mut self, surface: &Adaptor3dSurfaceHandle) -> Result<(), Failure> {
        let bounds: [f64; 4] = [
            surface.first_u_parameter()?,
            surface.first_v_parameter()?,
            surface.last_u_parameter()?,
            surface.last_v_parameter()?,
        ];
        let eval = GeomGridEvalSurface::from_adaptor(&**surface)?;
        let sizes = (self.nbdelta_u, self.nbdelta_v);
        utils::init_uniform(&eval, bounds, sizes, &mut self.grid, &mut self.the_bnd)?;
        let nb_triangles: i32 = self.nb_triangles();
        let mut tol: f64 =
            utils::compute_max_deflection(&**surface, &self.grid, self.nbdelta_v, nb_triangles)?;
        tol *= DEFLECTION_COEFF;
        self.set_deflection_over_estimation(tol);
        self.fill_bounding()
    }

    /// `Destroy()`: releases the point arrays (the destructor calls it); any later
    /// point access fails with Standard_OutOfRange. `polyhedron.destroy()`.
    pub fn destroy(&mut self) {
        utils::destroy(&mut self.grid);
    }

    /// `DeflectionOverEstimation(flec)`: the deflection, at least 1e-4, and the box
    /// enlarged by it. `polyhedron.set_deflection_over_estimation(0.01)`.
    pub fn set_deflection_over_estimation(&mut self, flec: f64) {
        utils::set_deflection_over_estimation(flec, &mut self.the_deflection, &mut self.the_bnd);
    }

    /// `Size(nbdu, nbdv)`: the cell counts. `polyhedron.size()`.
    pub fn size(&self) -> (i32, i32) {
        (self.nbdelta_u, self.nbdelta_v)
    }

    /// `NbTriangles()`: `2 * nbdu * nbdv`. `polyhedron.nb_triangles()`.
    pub fn nb_triangles(&self) -> i32 {
        utils::nb_triangles(self.nbdelta_u, self.nbdelta_v)
    }

    /// `Triangle(Index, P1, P2, P3)`: the point indices of a triangle, by arithmetic
    /// (no range check natively). `polyhedron.triangle(1)`.
    pub fn triangle(&self, index: i32) -> [i32; 3] {
        utils::triangle(index, self.nbdelta_v)
    }

    /// `TriConnex(Triang, Pivot, Pedge, TriCon, OtherP)`: `(returned, TriCon, OtherP)`,
    /// the triangle across the edge `pivot pedge` and its third point. When the pivot
    /// and the edge point coincide, OtherP is 0 and TriCon is `triang`; when the third
    /// point coincides with the edge point, 0 is returned ("BUG NON CORRIGE").
    /// `polyhedron.tri_connex(1, 1, 2)?`.
    pub fn tri_connex(
        &self,
        triang: i32,
        pivot: i32,
        pedge: i32,
    ) -> Result<(i32, i32, i32), Failure> {
        let (_, tri_con, other_p) =
            tri_connex(triang, pivot, pedge, (self.nbdelta_u, self.nbdelta_v));
        if pedge == 0 {
            return Ok((tri_con, tri_con, other_p));
        }
        let edge: Point3 = self.point(pedge)?;
        if self.point(pivot)?.square_distance(&edge) <= LONGUEUR_MINI_EDGE_TRIANGLE {
            return Ok((triang, triang, 0));
        }
        if self.point(other_p)?.square_distance(&edge) <= LONGUEUR_MINI_EDGE_TRIANGLE {
            return Ok((0, tri_con, other_p));
        }
        Ok((tri_con, tri_con, other_p))
    }

    /// `NbPoints()`: `(nbdu + 1) * (nbdv + 1)`. `polyhedron.nb_points()`.
    pub fn nb_points(&self) -> i32 {
        utils::nb_points(self.nbdelta_u, self.nbdelta_v)
    }

    /// `Point(thePnt, lig, col, U, V)`: does nothing ("Ne dois pas etre appelle").
    /// `polyhedron.set_point(p, 1, 1, 0.0, 0.0)`.
    pub fn set_point(&mut self, _pnt: Point3, _lig: i32, _col: i32, _u: f64, _v: f64) {}

    fn slot(&self, index: i32) -> Result<usize, Failure> {
        let len: usize = self.grid.pnts.len();
        usize::try_from(index)
            .ok()
            .filter(|&at| at < len)
            .ok_or_else(|| {
                Failure::out_of_range(&format!(
                    "IntPatch_Polyhedron::Point() - index {index} is outside the grid 0..={}",
                    len as i32 - 1
                ))
            })
    }

    /// `Point(Index, U, V)`: the point and its parameters; index 0 is the origin with
    /// null parameters (uninitialized natively); Standard_OutOfRange past the grid
    /// (an out-of-bounds read natively). `polyhedron.point_with_parameters(1)?`.
    pub fn point_with_parameters(&self, index: i32) -> Result<(Point3, f64, f64), Failure> {
        let at: usize = self.slot(index)?;
        Ok((self.grid.pnts[at], self.grid.u[at], self.grid.v[at]))
    }

    /// `Point(Index)` and `Point(Index, P)`. `polyhedron.point(1)?`.
    pub fn point(&self, index: i32) -> Result<Point3, Failure> {
        Ok(self.grid.pnts[self.slot(index)?])
    }

    /// `Bounding()`: the box of the points, enlarged by the deflection.
    /// `polyhedron.bounding()`.
    pub fn bounding(&self) -> &BoundingBox3 {
        &self.the_bnd
    }

    /// `FillBounding()`: the box of each triangle enlarged by the deflection; a
    /// triangle with an edge shorter than 1e-7 gets a void box (with that gap).
    /// `polyhedron.fill_bounding()?`.
    pub fn fill_bounding(&mut self) -> Result<(), Failure> {
        let count: i32 = self.nb_triangles();
        let mut boxes: Array1<'static, BoundingBox3> =
            Array1::with_bounds(1, count).map_err(|e| e.into_failure("NCollection_HArray1"))?;
        for i_tri in 1..=count {
            let [p1, p2, p3] = self.triangle_points(i_tri)?;
            let mut boite = BoundingBox3::default();
            if has_edges(p1, p2, p3) {
                boite.add_point(p1);
                boite.add_point(p2);
                boite.add_point(p3);
            }
            boite.enlarge(self.the_deflection);
            boxes[i_tri] = boite;
        }
        self.the_components_bnd = boxes;
        Ok(())
    }

    /// `ComponentsBounding()`: the box of each triangle. `polyhedron.components_bounding()`.
    pub fn components_bounding(&self) -> &Array1<'static, BoundingBox3> {
        &self.the_components_bnd
    }

    /// `DeflectionOverEstimation()`. `polyhedron.deflection_over_estimation()`.
    pub fn deflection_over_estimation(&self) -> f64 {
        self.the_deflection
    }

    fn triangle_points(&self, triang: i32) -> Result<[Point3; 3], Failure> {
        let [i1, i2, i3] = self.triangle(triang);
        Ok([self.point(i1)?, self.point(i2)?, self.point(i3)?])
    }

    /// `PlaneEquation(Triang, NormalVector, PolarDistance)`: the unit normal and the
    /// polar distance of the triangle. A null edge gives the normal `(1, 0, 0)` and leaves
    /// the distance unchanged (`distance` is its previous value); a normal below
    /// gp::Resolution() is kept unnormalized with the distance 0.
    /// `polyhedron.plane_equation(1, 0.0)?`.
    pub fn plane_equation(&self, triang: i32, distance: f64) -> Result<(Xyz, f64), Failure> {
        let [p1, p2, p3] = self.triangle_points(triang)?;
        let (v1, v2, v3) = (
            p2.xyz() - p1.xyz(),
            p3.xyz() - p2.xyz(),
            p1.xyz() - p3.xyz(),
        );
        let null = |v: Xyz| v.square_modulus() <= LONGUEUR_MINI_EDGE_TRIANGLE;
        if null(v1) || null(v2) || null(v3) {
            return Ok((Xyz::new(1.0, 0.0, 0.0), distance));
        }
        let mut normal: Xyz = (v1 ^ v2) + (v2 ^ v3) + (v3 ^ v1);
        let norm_len: f64 = normal.modulus();
        if norm_len < gp::resolution() {
            return Ok((normal, 0.0));
        }
        normal /= norm_len;
        Ok((normal, normal * p1.xyz()))
    }

    /// `Contain(Triang, ThePnt)`: whether the point is inside the triangle (the three
    /// cross products agree). `polyhedron.contain(1, p)?`.
    pub fn contain(&self, triang: i32, pnt: Point3) -> Result<bool, Failure> {
        let [p1, p2, p3] = self.triangle_points(triang)?;
        Ok(utils::contain(p1, p2, p3, pnt))
    }

    /// `Parameters(Index, U, V)`. `polyhedron.parameters(1)?`.
    pub fn parameters(&self, index: i32) -> Result<(f64, f64), Failure> {
        let at: usize = self.slot(index)?;
        Ok((self.grid.u[at], self.grid.v[at]))
    }

    /// `Dump()`: writes nothing natively. `polyhedron.dump()`.
    pub fn dump(&self) {}

    /// The four singularity flags (`UMinSingular`, `UMaxSingular`, `VMinSingular`,
    /// `VMaxSingular`), which `IntPatch_HInterTool` always sets false.
    /// `polyhedron.singularities()`.
    pub fn singularities(&self) -> [bool; 4] {
        self.singular
    }
}

/// Whether the three edges of the triangle are longer than the null length (the test of
/// `FillBounding`; false for a NaN point).
fn has_edges(p1: Point3, p2: Point3, p3: Point3) -> bool {
    p1.square_distance(&p2) > LONGUEUR_MINI_EDGE_TRIANGLE
        && p1.square_distance(&p3) > LONGUEUR_MINI_EDGE_TRIANGLE
        && p2.square_distance(&p3) > LONGUEUR_MINI_EDGE_TRIANGLE
}
