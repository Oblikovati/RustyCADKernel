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
// Rust translation of ModelingData/TKGeomBase/ProjLib/ProjLib_Plane.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ProjLib_Plane`: the exact projection of a line or a conic onto a plane, in the
//! plane coordinates (dot products with the plane axes).
use super::projector::{Project, Projector, projector_base};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{
    Axis2, Axis22d, Circle2, Circle3, Direction2, Direction3, Ellipse2, Ellipse3, Hyperbola2,
    Hyperbola3, Line2, Line3, Parabola2, Parabola3, Plane3, Point2, Point3, Vector3,
};

/// `ProjLib_Plane`; the base-class result is reached through `Deref`.
///
/// ```
/// use rustycad_tkgeombase::proj_lib::Plane;
/// use rustycad_tkmath::gp::{Line3, Plane3};
/// let projection = Plane::with_line(&Plane3::default(), &Line3::default()).unwrap();
/// assert!(projection.is_done());
/// ```
#[derive(Clone, Default)]
pub struct Plane {
    base: Projector,
    plane: Plane3,
}
projector_base!(Plane);

impl Plane {
    /// `ProjLib_Plane()`: an undefined projection; `Plane::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `ProjLib_Plane(Pl)`; `Plane::with_plane(&plane)`.
    pub fn with_plane(plane: &Plane3) -> Self {
        let mut projection: Self = Self::new();
        projection.init(plane);
        projection
    }
    /// `ProjLib_Plane(Pl, L)`; `Plane::with_line(&plane, &line)?`.
    pub fn with_line(plane: &Plane3, line: &Line3) -> Result<Self, Failure> {
        let mut projection: Self = Self::with_plane(plane);
        projection.project_line(line)?;
        Ok(projection)
    }
    /// `ProjLib_Plane(Pl, C)`; `Plane::with_circle(&plane, &circle)?`.
    pub fn with_circle(plane: &Plane3, circle: &Circle3) -> Result<Self, Failure> {
        let mut projection: Self = Self::with_plane(plane);
        projection.project_circle(circle)?;
        Ok(projection)
    }
    /// `ProjLib_Plane(Pl, E)`; `Plane::with_ellipse(&plane, &ellipse)?`.
    pub fn with_ellipse(plane: &Plane3, ellipse: &Ellipse3) -> Result<Self, Failure> {
        let mut projection: Self = Self::with_plane(plane);
        projection.project_ellipse(ellipse)?;
        Ok(projection)
    }
    /// `ProjLib_Plane(Pl, P)`; `Plane::with_parabola(&plane, &parabola)?`.
    pub fn with_parabola(plane: &Plane3, parabola: &Parabola3) -> Result<Self, Failure> {
        let mut projection: Self = Self::with_plane(plane);
        projection.project_parabola(parabola)?;
        Ok(projection)
    }
    /// `ProjLib_Plane(Pl, H)`; `Plane::with_hyperbola(&plane, &hyperbola)?`.
    pub fn with_hyperbola(plane: &Plane3, hyperbola: &Hyperbola3) -> Result<Self, Failure> {
        let mut projection: Self = Self::with_plane(plane);
        projection.project_hyperbola(hyperbola)?;
        Ok(projection)
    }
    /// `Init(Pl)`: resets the result to an undone OtherCurve; `projection.init(&plane)`.
    pub fn init(&mut self, plane: &Plane3) {
        self.base.my_type = CurveType::OtherCurve;
        self.base.is_done = false;
        self.base.is_periodic = false;
        self.plane = *plane;
    }
}

// `EvalPnt2d`: the plane coordinates of a point by dot products with the plane axes.
fn eval_pnt2d(point: Point3, plane: &Plane3) -> Point2 {
    let op: Vector3 = Vector3::between(plane.location(), point);
    let position = plane.position();
    Point2::new(
        op.dot(Vector3::from(position.x_direction())),
        op.dot(Vector3::from(position.y_direction())),
    )
}

// `EvalDir2d`: the plane components of a direction, normalized by gp_Dir2d (which
// raises for a direction normal to the plane).
fn eval_dir2d(direction: Direction3, plane: &Plane3) -> Result<Direction2, Failure> {
    let position = plane.position();
    Direction2::new(
        direction.dot(position.x_direction()),
        direction.dot(position.y_direction()),
    )
}

// The gp_Ax22d of a conic position: its location, X and Y directions in the plane.
fn eval_frame(position: Axis2, plane: &Plane3) -> Result<Axis22d, Failure> {
    let location: Point2 = eval_pnt2d(position.location(), plane);
    let x: Direction2 = eval_dir2d(position.x_direction(), plane)?;
    let y: Direction2 = eval_dir2d(position.y_direction(), plane)?;
    Axis22d::with_directions(location, x, y)
}

impl Project for Plane {
    fn projector(&self) -> &Projector {
        &self.base
    }
    fn project_line(&mut self, line: &Line3) -> Result<(), Failure> {
        self.base.my_type = CurveType::Line;
        let location: Point2 = eval_pnt2d(line.location(), &self.plane);
        self.base.lin = Line2::new(location, eval_dir2d(line.direction(), &self.plane)?);
        self.base.is_done = true;
        Ok(())
    }
    fn project_circle(&mut self, circle: &Circle3) -> Result<(), Failure> {
        self.base.my_type = CurveType::Circle;
        let frame: Axis22d = eval_frame(circle.position(), &self.plane)?;
        self.base.circ = Circle2::new(frame, circle.radius())?;
        self.base.is_periodic = true;
        self.base.is_done = true;
        Ok(())
    }
    fn project_ellipse(&mut self, ellipse: &Ellipse3) -> Result<(), Failure> {
        self.base.my_type = CurveType::Ellipse;
        let frame: Axis22d = eval_frame(ellipse.position(), &self.plane)?;
        self.base.elips = Ellipse2::new(frame, ellipse.major_radius(), ellipse.minor_radius())?;
        self.base.is_periodic = true;
        self.base.is_done = true;
        Ok(())
    }
    fn project_parabola(&mut self, parabola: &Parabola3) -> Result<(), Failure> {
        self.base.my_type = CurveType::Parabola;
        let frame: Axis22d = eval_frame(parabola.position(), &self.plane)?;
        self.base.parab = Parabola2::new(frame, parabola.focal())?;
        self.base.is_done = true;
        Ok(())
    }
    fn project_hyperbola(&mut self, hyperbola: &Hyperbola3) -> Result<(), Failure> {
        self.base.my_type = CurveType::Hyperbola;
        let frame: Axis22d = eval_frame(hyperbola.position(), &self.plane)?;
        let (major, minor) = (hyperbola.major_radius(), hyperbola.minor_radius());
        self.base.hypr = Hyperbola2::new(frame, major, minor)?;
        self.base.is_done = true;
        Ok(())
    }
}
