// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of gce_MakeMirror, gce_MakeMirror2d, gce_MakeRotation, gce_MakeRotation2d,
// gce_MakeScale, gce_MakeScale2d, gce_MakeTranslation and gce_MakeTranslation2d (.hxx/.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The gce transformation makers. They are not gce_Root algorithms: every
//! construction succeeds (or raises through gp), and `Value()` returns the
//! transformation.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis2d, Direction2, Direction3, Line2, Line3, Plane3, Point2, Point3, Transform2,
    Transform3, Vector2, Vector3,
};

/// `Value()`, `Operator()` and the conversion operator of a transformation maker.
macro_rules! transformation_result {
    ($maker:ident, $result:ty, $native:literal) => {
        impl $maker {
            #[doc = concat!("`", $native, "::Value`: the transformation; `maker.value()`.")]
            pub fn value(&self) -> $result {
                self.the_result
            }
            #[doc = concat!("`", $native, "::Operator`: the transformation; `maker.operator()`.")]
            pub fn operator(&self) -> $result {
                self.value()
            }
        }
        #[doc = concat!("The conversion operator of `", $native, "`; `Type::from(&maker)`.")]
        impl From<&$maker> for $result {
            fn from(maker: &$maker) -> Self {
                maker.operator()
            }
        }
    };
}

/// gce_MakeMirror: the symmetry about a point, an axis, a line or a plane.
///
/// `let mirror = MakeMirror::from_point(Point3::default()).value();`
#[derive(Clone, Copy, Debug, Default)]
pub struct MakeMirror {
    the_result: Transform3,
}
transformation_result!(MakeMirror, Transform3, "gce_MakeMirror");

impl MakeMirror {
    fn mirror(set: impl FnOnce(&mut Transform3)) -> Self {
        let mut the_result = Transform3::default();
        set(&mut the_result);
        Self { the_result }
    }

    /// gce_MakeMirror(Point): the central symmetry about `point`.
    /// `MakeMirror::from_point(p)`.
    pub fn from_point(point: Point3) -> Self {
        Self::mirror(|t| t.set_mirror_point(point))
    }

    /// gce_MakeMirror(Axis): the axial symmetry about `axis`.
    /// `MakeMirror::from_axis(axis)`.
    pub fn from_axis(axis: Axis1) -> Self {
        Self::mirror(|t| t.set_mirror_axis(axis))
    }

    /// gce_MakeMirror(Line): the axial symmetry about `line`.
    /// `MakeMirror::from_line(line)`.
    pub fn from_line(line: Line3) -> Self {
        Self::mirror(|t| t.set_mirror_axis(Axis1::new(line.location(), line.direction())))
    }

    /// gce_MakeMirror(Point, Direc): the axial symmetry about the axis (`point`,
    /// `direction`). `MakeMirror::from_point_direction(p, d)`.
    pub fn from_point_direction(point: Point3, direction: Direction3) -> Self {
        Self::mirror(|t| t.set_mirror_axis(Axis1::new(point, direction)))
    }

    /// gce_MakeMirror(Plane): the planar symmetry about the plane of `frame`.
    /// `MakeMirror::from_frame(frame)`.
    pub fn from_frame(frame: Axis2) -> Self {
        Self::mirror(|t| t.set_mirror_plane(frame))
    }

    /// gce_MakeMirror(Plane): the planar symmetry about `plane`, through
    /// `Plane.Position().Ax2()`. `MakeMirror::from_plane(plane)?`.
    pub fn from_plane(plane: Plane3) -> Result<Self, Failure> {
        let frame: Axis2 = plane.position().ax2()?;
        Ok(Self::mirror(|t| t.set_mirror_plane(frame)))
    }
}

/// gce_MakeMirror2d: the 2D symmetry about a point, an axis or a line.
///
/// `let mirror = MakeMirror2d::from_point(Point2::default()).value();`
#[derive(Clone, Copy, Debug, Default)]
pub struct MakeMirror2d {
    the_result: Transform2,
}
transformation_result!(MakeMirror2d, Transform2, "gce_MakeMirror2d");

impl MakeMirror2d {
    fn mirror(set: impl FnOnce(&mut Transform2)) -> Self {
        let mut the_result = Transform2::default();
        set(&mut the_result);
        Self { the_result }
    }

    /// gce_MakeMirror2d(Point): the central symmetry about `point`.
    /// `MakeMirror2d::from_point(p)`.
    pub fn from_point(point: Point2) -> Self {
        Self::mirror(|t| t.set_mirror_point(point))
    }

    /// gce_MakeMirror2d(Axis): the axial symmetry about `axis`.
    /// `MakeMirror2d::from_axis(axis)`.
    pub fn from_axis(axis: Axis2d) -> Self {
        Self::mirror(|t| t.set_mirror_axis(axis))
    }

    /// gce_MakeMirror2d(Line): the axial symmetry about `line`.
    /// `MakeMirror2d::from_line(line)`.
    pub fn from_line(line: Line2) -> Self {
        Self::mirror(|t| t.set_mirror_axis(Axis2d::new(line.location(), line.direction())))
    }

    /// gce_MakeMirror2d(Point, Direc): the axial symmetry about the axis
    /// (`point`, `direction`). `MakeMirror2d::from_point_direction(p, d)`.
    pub fn from_point_direction(point: Point2, direction: Direction2) -> Self {
        Self::mirror(|t| t.set_mirror_axis(Axis2d::new(point, direction)))
    }
}

/// gce_MakeRotation: the rotation about a line or an axis.
///
/// `let rotation = MakeRotation::from_axis(Axis1::default(), 1.0)?.value();`
#[derive(Clone, Copy, Debug, Default)]
pub struct MakeRotation {
    the_result: Transform3,
}
transformation_result!(MakeRotation, Transform3, "gce_MakeRotation");

impl MakeRotation {
    /// gce_MakeRotation(Line, Angle): the rotation by `angle` about `line`.
    /// `MakeRotation::from_line(line, 1.0)?`.
    pub fn from_line(line: Line3, angle: f64) -> Result<Self, Failure> {
        Self::from_axis(line.position(), angle)
    }

    /// gce_MakeRotation(Axis, Angle): the rotation by `angle` about `axis`.
    /// `MakeRotation::from_axis(axis, 1.0)?`.
    pub fn from_axis(axis: Axis1, angle: f64) -> Result<Self, Failure> {
        let mut the_result = Transform3::default();
        the_result.set_rotation(axis, angle)?;
        Ok(Self { the_result })
    }

    /// gce_MakeRotation(Point, Direc, Angle): the rotation by `angle` about the
    /// axis (`point`, `direction`). `MakeRotation::from_point_direction(p, d, 1.0)?`.
    pub fn from_point_direction(
        point: Point3,
        direction: Direction3,
        angle: f64,
    ) -> Result<Self, Failure> {
        Self::from_axis(Axis1::new(point, direction), angle)
    }
}

/// gce_MakeRotation2d: the rotation about a point.
///
/// `let rotation = MakeRotation2d::new(Point2::default(), 1.0).value();`
#[derive(Clone, Copy, Debug, Default)]
pub struct MakeRotation2d {
    the_result: Transform2,
}
transformation_result!(MakeRotation2d, Transform2, "gce_MakeRotation2d");

impl MakeRotation2d {
    /// gce_MakeRotation2d(Point, Angle): the rotation by `angle` about `point`.
    /// `MakeRotation2d::new(p, 1.0)`.
    pub fn new(point: Point2, angle: f64) -> Self {
        let mut the_result = Transform2::default();
        the_result.set_rotation(point, angle);
        Self { the_result }
    }
}

/// gce_MakeScale: the scaling about a point.
///
/// `let scale = MakeScale::new(Point3::default(), 2.0)?.value();`
#[derive(Clone, Copy, Debug, Default)]
pub struct MakeScale {
    the_result: Transform3,
}
transformation_result!(MakeScale, Transform3, "gce_MakeScale");

impl MakeScale {
    /// gce_MakeScale(Point, Scale): the scaling by `scale` about `point`; gp
    /// raises Standard_ConstructionError for a null factor.
    /// `MakeScale::new(p, 2.0)?`.
    pub fn new(point: Point3, scale: f64) -> Result<Self, Failure> {
        let mut the_result = Transform3::default();
        the_result.set_scale(point, scale)?;
        Ok(Self { the_result })
    }
}

/// gce_MakeScale2d: the 2D scaling about a point.
///
/// `let scale = MakeScale2d::new(Point2::default(), 2.0).value();`
#[derive(Clone, Copy, Debug, Default)]
pub struct MakeScale2d {
    the_result: Transform2,
}
transformation_result!(MakeScale2d, Transform2, "gce_MakeScale2d");

impl MakeScale2d {
    /// gce_MakeScale2d(Point, Scale): the scaling by `scale` about `point`.
    /// `MakeScale2d::new(p, 2.0)`.
    pub fn new(point: Point2, scale: f64) -> Self {
        let mut the_result = Transform2::default();
        the_result.set_scale(point, scale);
        Self { the_result }
    }
}

/// gce_MakeTranslation: the translation by a vector or between two points.
///
/// `let translation = MakeTranslation::new(Vector3::new(1.0, 0.0, 0.0)).value();`
#[derive(Clone, Copy, Debug, Default)]
pub struct MakeTranslation {
    the_result: Transform3,
}
transformation_result!(MakeTranslation, Transform3, "gce_MakeTranslation");

impl MakeTranslation {
    /// gce_MakeTranslation(Vec): the translation by `vector`.
    /// `MakeTranslation::new(v)`.
    pub fn new(vector: Vector3) -> Self {
        let mut the_result = Transform3::default();
        the_result.set_translation(vector);
        Self { the_result }
    }

    /// gce_MakeTranslation(Point1, Point2): the translation by `gp_Vec(p1, p2)`.
    /// `MakeTranslation::from_points(a, b)`.
    pub fn from_points(p1: Point3, p2: Point3) -> Self {
        Self::new(Vector3::between(p1, p2))
    }
}

/// gce_MakeTranslation2d: the 2D translation by a vector or between two points.
///
/// `let translation = MakeTranslation2d::new(Vector2::new(1.0, 0.0)).value();`
#[derive(Clone, Copy, Debug, Default)]
pub struct MakeTranslation2d {
    the_result: Transform2,
}
transformation_result!(MakeTranslation2d, Transform2, "gce_MakeTranslation2d");

impl MakeTranslation2d {
    /// gce_MakeTranslation2d(Vec): the translation by `vector`.
    /// `MakeTranslation2d::new(v)`.
    pub fn new(vector: Vector2) -> Self {
        let mut the_result = Transform2::default();
        the_result.set_translation(vector);
        Self { the_result }
    }

    /// gce_MakeTranslation2d(Point1, Point2): the translation by
    /// `gp_Vec2d(p1, p2)`. `MakeTranslation2d::from_points(a, b)`.
    pub fn from_points(p1: Point2, p2: Point2) -> Self {
        Self::new(Vector2::between(p1, p2))
    }
}
