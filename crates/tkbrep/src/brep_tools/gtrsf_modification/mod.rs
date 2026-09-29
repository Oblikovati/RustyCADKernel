// Created on: 1996-12-30
// Created by: Stagiaire Mary FABIEN
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
// Rust translation of ModelingData/TKBRep/BRepTools/BRepTools_GTrsfModification.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools_GTrsfModification`: the modification that applies a gp_GTrsf to
//! B-spline and Bezier geometry.
mod mesh;

use super::modification::{BRepToolsModification, NewCurve, NewSurface};
use crate::brep::BRepTool;
use crate::topods::{Edge, Face, Vertex};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBSplineSurface, GeomBezierCurve, GeomBezierSurface, GeomCurve,
    GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{GeneralTransform3, Matrix3, Point3, Xyz};
use rustycad_tkmath::toploc::Location;

/// The nine `Value(i, j)` coefficients of the vectorial part, row by row: the scale
/// factor is applied for a similarity form, unlike `VectorialPart()`.
fn vectorial_values(gtrsf: &GeneralTransform3) -> [[f64; 3]; 3] {
    let mut values = [[0.0; 3]; 3];
    for (row, line) in values.iter_mut().enumerate() {
        for (column, value) in line.iter_mut().enumerate() {
            let cell = gtrsf.value(row as i32 + 1, column as i32 + 1);
            *value = cell.expect("the 3x3 cells of a gp_GTrsf are in range");
        }
    }
    values
}

/// `maxAbsCoefficient`: the largest absolute coefficient `Value(i, j)` of the vectorial
/// part, as `std::max({...})` (the first value, replaced by each larger one).
fn max_abs_coefficient(gtrsf: &GeneralTransform3) -> f64 {
    let values = vectorial_values(gtrsf);
    let mut largest = values[0][0].abs();
    for value in values.iter().flatten().skip(1) {
        if largest < value.abs() {
            largest = value.abs();
        }
    }
    largest
}

/// `normalizedVectorialPart`: the `Value(i, j)` matrix divided by `scale` when it is
/// positive.
pub(super) fn normalized_vectorial_part(gtrsf: &GeneralTransform3, scale: f64) -> Matrix3 {
    let mut matrix = Matrix3::new(vectorial_values(gtrsf));
    if scale > 0.0 {
        matrix *= 1.0 / scale;
    }
    matrix
}

/// `representationTransformation`: `Target^-1`, then the GTrsf, then `Source`.
pub(super) fn representation_transformation(
    gtrsf: &GeneralTransform3,
    source: &Location,
    target: &Location,
) -> Result<GeneralTransform3, Failure> {
    let mut local = GeneralTransform3::from(target.transformation().inverted()?);
    local.multiply(*gtrsf);
    local.multiply(GeneralTransform3::from(source.transformation()));
    Ok(local)
}

/// The GTrsf and its scale (`myGTrsf`, `myGScale`).
#[derive(Clone, Copy)]
struct GTrsfFields {
    gtrsf: GeneralTransform3,
    scale: f64,
}

impl GTrsfFields {
    fn new(gtrsf: GeneralTransform3) -> Self {
        Self {
            gtrsf,
            scale: max_abs_coefficient(&gtrsf),
        }
    }
}

/// `BRepTools_GTrsfModification`: B-spline and Bezier surfaces and curves get their
/// poles transformed by a gp_GTrsf; tolerances and deflections are scaled by the
/// largest absolute coefficient of the vectorial part. Other surfaces and curves
/// raise Standard_NoSuchObject, as natively.
///
/// ```
/// use rustycad_tkbrep::brep_tools::GTrsfModification;
/// use rustycad_tkmath::gp::GeneralTransform3;
/// let modification = GTrsfModification::new(GeneralTransform3::default());
/// assert_eq!(modification.gtrsf().form(), GeneralTransform3::default().form());
/// ```
pub struct GTrsfModification {
    parent: StandardTransient,
    fields: SharedState<GTrsfFields>,
}
impl_transient!(
    GTrsfModification,
    "BRepTools_GTrsfModification",
    dyn BRepToolsModification,
    parent,
    interfaces: [dyn BRepToolsModification]
);

impl GTrsfModification {
    /// `BRepTools_GTrsfModification(T)`; `GTrsfModification::new(gtrsf)`.
    pub fn new(gtrsf: GeneralTransform3) -> Self {
        Self {
            parent: StandardTransient::default(),
            fields: SharedState::new(GTrsfFields::new(gtrsf)),
        }
    }

    /// `SetGTrsf(theGTrsf)`: replaces the transformation and its scale;
    /// `modification.set_gtrsf(gtrsf)`.
    pub fn set_gtrsf(&self, gtrsf: GeneralTransform3) {
        *self.fields.write() = GTrsfFields::new(gtrsf);
    }

    /// `GTrsf()`; `modification.gtrsf()`.
    pub fn gtrsf(&self) -> GeneralTransform3 {
        self.fields.read().gtrsf
    }

    /// `myGScale`.
    fn scale(&self) -> f64 {
        self.fields.read().scale
    }

    /// `myGTrsf.Transforms(pole.ChangeCoord())`.
    fn transformed_point(&self, point: Point3) -> Point3 {
        let mut coordinates: Xyz = point.xyz();
        self.gtrsf().transforms(&mut coordinates);
        Point3::new(coordinates.x(), coordinates.y(), coordinates.z())
    }
}

/// Standard_NoSuchObject of an unsupported surface or curve type.
fn unsupported(what: &str) -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        &format!(
            "BRepTools_GTrsfModification : Pb no BSpline/Bezier Type {what}; expected a Geom_BSpline or Geom_Bezier {what}"
        ),
        "",
    )
}

impl GTrsfModification {
    /// The poles of a B-spline or Bezier surface transformed in place.
    fn transform_surface_poles(&self, surface: &Handle<dyn GeomSurface>) -> Result<(), Failure> {
        if let Some(bspline) = surface.cast::<GeomBSplineSurface>().get() {
            for (i, j) in pole_grid(bspline.nb_u_poles(), bspline.nb_v_poles()) {
                bspline.set_pole(i, j, self.transformed_point(bspline.pole(i, j)?))?;
            }
            return Ok(());
        }
        if let Some(bezier) = surface.cast::<GeomBezierSurface>().get() {
            for (i, j) in pole_grid(bezier.nb_u_poles(), bezier.nb_v_poles()) {
                bezier.set_pole(i, j, self.transformed_point(bezier.pole(i, j)?))?;
            }
            return Ok(());
        }
        Err(unsupported("Surface"))
    }

    /// The poles of a B-spline or Bezier curve transformed in place.
    fn transform_curve_poles(&self, curve: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
        if let Some(bspline) = curve.cast::<GeomBSplineCurve>().get() {
            for i in 1..=bspline.nb_poles() {
                bspline.set_pole(i, self.transformed_point(bspline.pole(i)?))?;
            }
            return Ok(());
        }
        if let Some(bezier) = curve.cast::<GeomBezierCurve>().get() {
            for i in 1..=bezier.nb_poles() {
                bezier.set_pole(i, self.transformed_point(bezier.pole(i)?))?;
            }
            return Ok(());
        }
        Err(unsupported("Curve"))
    }
}

/// The `(i, j)` pole indices, U outer, V inner.
fn pole_grid(nb_u: i32, nb_v: i32) -> impl Iterator<Item = (i32, i32)> {
    (1..=nb_u).flat_map(move |i| (1..=nb_v).map(move |j| (i, j)))
}

impl BRepToolsModification for GTrsfModification {
    /// A copy of the located surface with its poles transformed (identity location),
    /// the scaled tolerance, the face reversed for a negative normalised vectorial
    /// part; false for a face without surface.
    fn new_surface(&self, face: &Face, result: &mut NewSurface) -> Result<bool, Failure> {
        let (surface, location) = BRepTool::surface_with_location(face)?;
        result.surface = surface;
        result.location = location;
        let Some(stored) = result.surface.get() else {
            return Ok(false);
        };
        result.tolerance = BRepTool::face_tolerance(face)? * self.scale();
        result.rev_wires = false;
        let determinant = normalized_vectorial_part(&self.gtrsf(), self.scale()).determinant();
        result.rev_face = determinant < 0.0;
        result.surface = stored.transformed(result.location.transformation())?.cast();
        self.transform_surface_poles(&result.surface)?;
        result.location = Location::default();
        Ok(true)
    }

    fn new_triangulation(
        &self,
        face: &Face,
        triangulation: &mut Handle<rustycad_tkmath::poly::Triangulation>,
    ) -> Result<bool, Failure> {
        self.transformed_triangulation(face, triangulation)
    }

    /// A copy of the located 3D curve with its poles transformed, trimmed to the edge
    /// range (identity location), and the scaled tolerance; false for an edge without
    /// curve.
    fn new_curve(&self, edge: &Edge, result: &mut NewCurve) -> Result<bool, Failure> {
        result.tolerance = BRepTool::edge_tolerance(edge)? * self.scale();
        let found = BRepTool::curve_with_location(edge)?;
        result.curve = found.curve;
        result.location = found.location;
        if let Some(stored) = result.curve.get() {
            let moved: Handle<dyn GeomCurve> =
                stored.transformed(result.location.transformation())?.cast();
            self.transform_curve_poles(&moved)?;
            let trimmed = GeomTrimmedCurve::new(&moved, found.first, found.last, true, true)?;
            result.curve = Handle::new(trimmed).cast();
        }
        result.location = Location::default();
        Ok(!result.curve.is_null())
    }

    fn new_polygon(
        &self,
        edge: &Edge,
        polygon: &mut Handle<rustycad_tkmath::poly::Polygon3D>,
    ) -> Result<bool, Failure> {
        self.transformed_polygon(edge, polygon)
    }

    fn new_polygon_on_triangulation(
        &self,
        edge: &Edge,
        face: &Face,
        polygon: &mut Handle<rustycad_tkmath::poly::PolygonOnTriangulation>,
    ) -> Result<bool, Failure> {
        self.copied_polygon_on_triangulation(edge, face, polygon)
    }

    /// The transformed point and the scaled tolerance; always true.
    fn new_point(
        &self,
        vertex: &Vertex,
        point: &mut Point3,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        let source = BRepTool::pnt(vertex)?;
        *tolerance = BRepTool::vertex_tolerance(vertex)? * self.scale();
        *point = self.transformed_point(source);
        Ok(true)
    }

    /// The p-curve trimmed to its range (the parametrisation is kept) and the scaled
    /// tolerance; false for an edge without p-curve.
    fn new_curve2d(
        &self,
        edge: &Edge,
        face: &Face,
        _new_edge: &Edge,
        _new_face: &Face,
        curve: &mut Handle<dyn Geom2dCurve>,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        *tolerance = BRepTool::edge_tolerance(edge)? * self.scale();
        let found = BRepTool::curve_on_face(edge, face)?;
        *curve = found.pcurve;
        if curve.is_null() {
            return Ok(false);
        }
        *curve = Handle::new(Geom2dTrimmedCurve::new(
            curve,
            found.first,
            found.last,
            true,
            true,
        )?)
        .cast();
        Ok(true)
    }

    /// The parameter of the vertex on the edge (kept) and the scaled tolerance.
    fn new_parameter(
        &self,
        vertex: &Vertex,
        edge: &Edge,
        parameter: &mut f64,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        *tolerance = BRepTool::vertex_tolerance(vertex)? * self.scale();
        *parameter = BRepTool::parameter(vertex, edge)?;
        Ok(true)
    }

    /// The continuity of the edge between the original faces.
    fn continuity(
        &self,
        edge: &Edge,
        face1: &Face,
        face2: &Face,
        _new_edge: &Edge,
        _new_face1: &Face,
        _new_face2: &Face,
    ) -> Result<Continuity, Failure> {
        BRepTool::continuity_on_faces(edge, face1, face2)
    }
}
