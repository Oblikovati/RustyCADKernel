// Created on: 1993-07-23
// Created by: Joelle CHAUVET
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
// Rust translation of the constructors of BRepLib_MakeEdge.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The constructors of `BRepLib_MakeEdge`: each builds the Geom curve of its gp
//! argument, if any, and calls one Init.
use super::BRepLibMakeEdge;
use crate::brep_lib::errors::BRepLibEdgeError;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{
    GeomCircle, GeomCurve, GeomEllipse, GeomHyperbola, GeomLine, GeomParabola, GeomSurface,
};
use rustycad_tkmath::gp::{
    self, Circle3, Direction3, Ellipse3, Hyperbola3, Line3, Parabola3, Point3, Vector3,
};

/// The four constructors of one gp curve kind: whole, between parameters, points or
/// vertices, each on `Geom_X(L)`.
macro_rules! gp_curve_constructors {
    ($gp:ty, $native:literal, $geom:expr, $plain:ident, $params:ident, $points:ident, $vertices:ident) => {
        #[doc = concat!("`BRepLib_MakeEdge(const ", $native, "& L)`: the whole curve; `BRepLibMakeEdge::", stringify!($plain), "(l)?`.")]
        pub fn $plain(curve: $gp) -> Result<Self, Failure> {
            let geom: Handle<dyn GeomCurve> = Handle::new($geom(curve)).cast();
            Self::initialized(|maker| maker.init_curve(&geom))
        }
        #[doc = concat!("`BRepLib_MakeEdge(const ", $native, "& L, p1, p2)`; `BRepLibMakeEdge::", stringify!($params), "(l, 0.0, 1.0)?`.")]
        pub fn $params(curve: $gp, p1: f64, p2: f64) -> Result<Self, Failure> {
            let geom: Handle<dyn GeomCurve> = Handle::new($geom(curve)).cast();
            Self::initialized(|maker| maker.init_curve_params(&geom, p1, p2))
        }
        #[doc = concat!("`BRepLib_MakeEdge(const ", $native, "& L, P1, P2)`; `BRepLibMakeEdge::", stringify!($points), "(l, a, b)?`.")]
        pub fn $points(curve: $gp, p1: Point3, p2: Point3) -> Result<Self, Failure> {
            let geom: Handle<dyn GeomCurve> = Handle::new($geom(curve)).cast();
            Self::initialized(|maker| maker.init_curve_points(&geom, p1, p2))
        }
        #[doc = concat!("`BRepLib_MakeEdge(const ", $native, "& L, V1, V2)`; `BRepLibMakeEdge::", stringify!($vertices), "(l, &v1, &v2)?`.")]
        pub fn $vertices(curve: $gp, v1: &Vertex, v2: &Vertex) -> Result<Self, Failure> {
            let geom: Handle<dyn GeomCurve> = Handle::new($geom(curve)).cast();
            Self::initialized(|maker| maker.init_curve_vertices(&geom, v1, v2))
        }
    };
}

/// The segment from `p1` to `p2` as a Geom line and its length, or None when the
/// points are within gp::Resolution().
/// A segment line and its length.
type Segment = (Handle<dyn GeomCurve>, f64);

fn segment(p1: Point3, p2: Point3) -> Result<Option<Segment>, Failure> {
    let length: f64 = p1.distance(&p2);
    if length <= gp::resolution() {
        return Ok(None);
    }
    let direction = Direction3::from_vector(Vector3::between(p1, p2))?;
    let line = GeomLine::from_lin(Line3::new(p1, direction));
    Ok(Some((Handle::new(line).cast(), length)))
}

impl BRepLibMakeEdge {
    /// `BRepLib_MakeEdge(V1, V2)`: the segment between two vertices;
    /// `BRepLib_LineThroughIdenticPoints` when their points are within gp::Resolution();
    /// `BRepLibMakeEdge::from_vertices(&v1, &v2)?`.
    pub fn from_vertices(v1: &Vertex, v2: &Vertex) -> Result<Self, Failure> {
        let (p1, p2) = (BRepTool::pnt(v1)?, BRepTool::pnt(v2)?);
        let Some((line, length)) = segment(p1, p2)? else {
            let mut maker = Self::new();
            maker.my_error = BRepLibEdgeError::LineThroughIdenticPoints;
            return Ok(maker);
        };
        Self::initialized(|maker| maker.init_curve_vertices_params(&line, v1, v2, 0.0, length))
    }

    /// `BRepLib_MakeEdge(P1, P2)`: the segment between two points;
    /// `BRepLib_LineThroughIdenticPoints` when they are within gp::Resolution();
    /// `BRepLibMakeEdge::from_points(a, b)?`.
    pub fn from_points(p1: Point3, p2: Point3) -> Result<Self, Failure> {
        let Some((line, length)) = segment(p1, p2)? else {
            let mut maker = Self::new();
            maker.my_error = BRepLibEdgeError::LineThroughIdenticPoints;
            return Ok(maker);
        };
        Self::initialized(|maker| maker.init_curve_points_params(&line, p1, p2, 0.0, length))
    }

    gp_curve_constructors!(
        Line3,
        "gp_Lin",
        GeomLine::from_lin,
        from_lin,
        from_lin_params,
        from_lin_points,
        from_lin_vertices
    );
    gp_curve_constructors!(
        Circle3,
        "gp_Circ",
        GeomCircle::from_circ,
        from_circ,
        from_circ_params,
        from_circ_points,
        from_circ_vertices
    );
    gp_curve_constructors!(
        Ellipse3,
        "gp_Elips",
        GeomEllipse::from_elips,
        from_elips,
        from_elips_params,
        from_elips_points,
        from_elips_vertices
    );
    gp_curve_constructors!(
        Hyperbola3,
        "gp_Hypr",
        GeomHyperbola::from_hypr,
        from_hypr,
        from_hypr_params,
        from_hypr_points,
        from_hypr_vertices
    );
    gp_curve_constructors!(
        Parabola3,
        "gp_Parab",
        GeomParabola::from_parab,
        from_parab,
        from_parab_params,
        from_parab_points,
        from_parab_vertices
    );

    /// `BRepLib_MakeEdge(L)`: the whole Geom curve; `BRepLibMakeEdge::from_curve(&c)?`.
    pub fn from_curve(curve: &Handle<dyn GeomCurve>) -> Result<Self, Failure> {
        Self::initialized(|maker| maker.init_curve(curve))
    }

    /// `BRepLib_MakeEdge(L, p1, p2)`; `BRepLibMakeEdge::from_curve_params(&c, 0.0, 1.0)?`.
    pub fn from_curve_params(
        curve: &Handle<dyn GeomCurve>,
        p1: f64,
        p2: f64,
    ) -> Result<Self, Failure> {
        Self::initialized(|maker| maker.init_curve_params(curve, p1, p2))
    }

    /// `BRepLib_MakeEdge(L, P1, P2)`; `BRepLibMakeEdge::from_curve_points(&c, a, b)?`.
    pub fn from_curve_points(
        curve: &Handle<dyn GeomCurve>,
        p1: Point3,
        p2: Point3,
    ) -> Result<Self, Failure> {
        Self::initialized(|maker| maker.init_curve_points(curve, p1, p2))
    }

    /// `BRepLib_MakeEdge(L, V1, V2)`; `BRepLibMakeEdge::from_curve_vertices(&c, &v1, &v2)?`.
    pub fn from_curve_vertices(
        curve: &Handle<dyn GeomCurve>,
        v1: &Vertex,
        v2: &Vertex,
    ) -> Result<Self, Failure> {
        Self::initialized(|maker| maker.init_curve_vertices(curve, v1, v2))
    }

    /// `BRepLib_MakeEdge(L, P1, P2, p1, p2)`;
    /// `BRepLibMakeEdge::from_curve_points_params(&c, a, b, 0.0, 1.0)?`.
    pub fn from_curve_points_params(
        curve: &Handle<dyn GeomCurve>,
        point1: Point3,
        point2: Point3,
        p1: f64,
        p2: f64,
    ) -> Result<Self, Failure> {
        Self::initialized(|maker| maker.init_curve_points_params(curve, point1, point2, p1, p2))
    }

    /// `BRepLib_MakeEdge(L, V1, V2, p1, p2)`;
    /// `BRepLibMakeEdge::from_curve_vertices_params(&c, &v1, &v2, 0.0, 1.0)?`.
    pub fn from_curve_vertices_params(
        curve: &Handle<dyn GeomCurve>,
        v1: &Vertex,
        v2: &Vertex,
        p1: f64,
        p2: f64,
    ) -> Result<Self, Failure> {
        Self::initialized(|maker| maker.init_curve_vertices_params(curve, v1, v2, p1, p2))
    }

    /// `BRepLib_MakeEdge(L, S)`: the whole 2d curve on the surface;
    /// `BRepLibMakeEdge::from_pcurve(&c, &s)?`.
    pub fn from_pcurve(
        curve: &Handle<dyn Geom2dCurve>,
        surface: &Handle<dyn GeomSurface>,
    ) -> Result<Self, Failure> {
        Self::initialized(|maker| maker.init_pcurve(curve, surface))
    }

    /// `BRepLib_MakeEdge(L, S, p1, p2)`; `BRepLibMakeEdge::from_pcurve_params(&c, &s, 0.0, 1.0)?`.
    pub fn from_pcurve_params(
        curve: &Handle<dyn Geom2dCurve>,
        surface: &Handle<dyn GeomSurface>,
        p1: f64,
        p2: f64,
    ) -> Result<Self, Failure> {
        Self::initialized(|maker| maker.init_pcurve_params(curve, surface, p1, p2))
    }

    /// `BRepLib_MakeEdge(L, S, P1, P2)`; `BRepLibMakeEdge::from_pcurve_points(&c, &s, a, b)?`.
    pub fn from_pcurve_points(
        curve: &Handle<dyn Geom2dCurve>,
        surface: &Handle<dyn GeomSurface>,
        p1: Point3,
        p2: Point3,
    ) -> Result<Self, Failure> {
        Self::initialized(|maker| maker.init_pcurve_points(curve, surface, p1, p2))
    }

    /// `BRepLib_MakeEdge(L, S, V1, V2)`;
    /// `BRepLibMakeEdge::from_pcurve_vertices(&c, &s, &v1, &v2)?`.
    pub fn from_pcurve_vertices(
        curve: &Handle<dyn Geom2dCurve>,
        surface: &Handle<dyn GeomSurface>,
        v1: &Vertex,
        v2: &Vertex,
    ) -> Result<Self, Failure> {
        Self::initialized(|maker| maker.init_pcurve_vertices(curve, surface, v1, v2))
    }

    /// `BRepLib_MakeEdge(L, S, P1, P2, p1, p2)`;
    /// `BRepLibMakeEdge::from_pcurve_points_params(&c, &s, a, b, 0.0, 1.0)?`.
    pub fn from_pcurve_points_params(
        curve: &Handle<dyn Geom2dCurve>,
        surface: &Handle<dyn GeomSurface>,
        point1: Point3,
        point2: Point3,
        p1: f64,
        p2: f64,
    ) -> Result<Self, Failure> {
        Self::initialized(|maker| {
            maker.init_pcurve_points_params(curve, surface, point1, point2, p1, p2)
        })
    }

    /// `BRepLib_MakeEdge(L, S, V1, V2, p1, p2)`;
    /// `BRepLibMakeEdge::from_pcurve_vertices_params(&c, &s, &v1, &v2, 0.0, 1.0)?`.
    pub fn from_pcurve_vertices_params(
        curve: &Handle<dyn Geom2dCurve>,
        surface: &Handle<dyn GeomSurface>,
        v1: &Vertex,
        v2: &Vertex,
        p1: f64,
        p2: f64,
    ) -> Result<Self, Failure> {
        Self::initialized(|maker| maker.init_pcurve_vertices_params(curve, surface, v1, v2, p1, p2))
    }
}
