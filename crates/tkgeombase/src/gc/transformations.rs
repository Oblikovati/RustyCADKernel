// Rust translation of GC_MakeMirror, GC_MakeMirror2d, GC_MakeRotation, GC_MakeRotation2d,
// GC_MakeScale, GC_MakeScale2d, GC_MakeTranslation and GC_MakeTranslation2d (.hxx/.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The GC transformation makers: a new Geom_Transformation or
//! Geom2d_Transformation set by its own setter. They are not GC_Root algorithms.
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dTransformation;
use rustycad_tkg3d::geom::GeomTransformation;
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis2d, Direction2, Direction3, Line2, Line3, Plane3, Point2, Point3, Transform2,
    Transform3, Vector2, Vector3,
};

/// `new Geom_Transformation()` then one setter that can raise.
fn try_transformation3(
    set: impl FnOnce(&GeomTransformation) -> Result<(), Failure>,
) -> Result<Handle<GeomTransformation>, Failure> {
    let transformation = GeomTransformation::new(Transform3::default());
    set(&transformation)?;
    Ok(Handle::new(transformation))
}

/// `new Geom_Transformation()` then one setter.
fn transformation3(set: impl FnOnce(&GeomTransformation)) -> Handle<GeomTransformation> {
    let transformation = GeomTransformation::new(Transform3::default());
    set(&transformation);
    Handle::new(transformation)
}

/// `new Geom2d_Transformation()` then one setter.
fn transformation2(set: impl FnOnce(&Geom2dTransformation)) -> Handle<Geom2dTransformation> {
    let transformation = Geom2dTransformation::new(Transform2::default());
    set(&transformation);
    Handle::new(transformation)
}

/// GC_MakeMirror: the Geom symmetry about a point, an axis, a line or a plane.
///
/// `let mirror = MakeMirror::from_point(Point3::default()).value();`
#[derive(Clone, Default)]
pub struct MakeMirror {
    the_result: Handle<GeomTransformation>,
}
gc_transformation!(MakeMirror, GeomTransformation, "GC_MakeMirror");

impl MakeMirror {
    fn mirror(set: impl FnOnce(&GeomTransformation)) -> Self {
        Self {
            the_result: transformation3(set),
        }
    }

    /// GC_MakeMirror(Point): the central symmetry about `point`.
    /// `MakeMirror::from_point(p)`.
    pub fn from_point(point: Point3) -> Self {
        Self::mirror(|t| t.set_mirror_point(point))
    }

    /// GC_MakeMirror(Axis): the axial symmetry about `axis`.
    /// `MakeMirror::from_axis(axis)`.
    pub fn from_axis(axis: Axis1) -> Self {
        Self::mirror(|t| t.set_mirror_axis(axis))
    }

    /// GC_MakeMirror(Line): the axial symmetry about `line`.
    /// `MakeMirror::from_line(line)`.
    pub fn from_line(line: Line3) -> Self {
        Self::mirror(|t| t.set_mirror_axis(Axis1::new(line.location(), line.direction())))
    }

    /// GC_MakeMirror(Point, Direc): the axial symmetry about (`point`, `direction`).
    /// `MakeMirror::from_point_direction(p, d)`.
    pub fn from_point_direction(point: Point3, direction: Direction3) -> Self {
        Self::mirror(|t| t.set_mirror_axis(Axis1::new(point, direction)))
    }

    /// GC_MakeMirror(Plane): the planar symmetry about the plane of `frame`.
    /// `MakeMirror::from_frame(frame)`.
    pub fn from_frame(frame: Axis2) -> Self {
        Self::mirror(|t| t.set_mirror_plane(frame))
    }

    /// GC_MakeMirror(Plane): the planar symmetry about `plane`, through
    /// `Plane.Position().Ax2()`. `MakeMirror::from_plane(plane)?`.
    pub fn from_plane(plane: Plane3) -> Result<Self, Failure> {
        let the_result = try_transformation3(|t| {
            t.set_mirror_plane(plane.position().ax2()?);
            Ok(())
        })?;
        Ok(Self { the_result })
    }
}

/// GC_MakeMirror2d: the Geom2d symmetry about a point, an axis or a line.
///
/// `let mirror = MakeMirror2d::from_point(Point2::default()).value();`
#[derive(Clone, Default)]
pub struct MakeMirror2d {
    the_result: Handle<Geom2dTransformation>,
}
gc_transformation!(MakeMirror2d, Geom2dTransformation, "GC_MakeMirror2d");

impl MakeMirror2d {
    /// GC_MakeMirror2d(Point): the central symmetry about `point`.
    /// `MakeMirror2d::from_point(p)`.
    pub fn from_point(point: Point2) -> Self {
        Self {
            the_result: transformation2(|t| t.set_mirror_point(point)),
        }
    }

    /// GC_MakeMirror2d(Axis): the axial symmetry about `axis`.
    /// `MakeMirror2d::from_axis(axis)`.
    pub fn from_axis(axis: Axis2d) -> Self {
        Self {
            the_result: transformation2(|t| t.set_mirror_axis(axis)),
        }
    }

    /// GC_MakeMirror2d(Line): the axial symmetry about `line`.
    /// `MakeMirror2d::from_line(line)`.
    pub fn from_line(line: Line2) -> Self {
        Self::from_axis(Axis2d::new(line.location(), line.direction()))
    }

    /// GC_MakeMirror2d(Point, Direc): the axial symmetry about (`point`,
    /// `direction`). `MakeMirror2d::from_point_direction(p, d)`.
    pub fn from_point_direction(point: Point2, direction: Direction2) -> Self {
        Self::from_axis(Axis2d::new(point, direction))
    }
}

/// GC_MakeRotation: the Geom rotation about a line or an axis.
///
/// `let rotation = MakeRotation::from_axis(Axis1::default(), 1.0)?.value();`
#[derive(Clone, Default)]
pub struct MakeRotation {
    the_result: Handle<GeomTransformation>,
}
gc_transformation!(MakeRotation, GeomTransformation, "GC_MakeRotation");

impl MakeRotation {
    /// GC_MakeRotation(Line, Angle): the rotation by `angle` about `line`.
    /// `MakeRotation::from_line(line, 1.0)?`.
    pub fn from_line(line: Line3, angle: f64) -> Result<Self, Failure> {
        Self::from_axis(Axis1::new(line.location(), line.direction()), angle)
    }

    /// GC_MakeRotation(Axis, Angle): the rotation by `angle` about `axis`.
    /// `MakeRotation::from_axis(axis, 1.0)?`.
    pub fn from_axis(axis: Axis1, angle: f64) -> Result<Self, Failure> {
        let the_result = try_transformation3(|t| t.set_rotation(axis, angle))?;
        Ok(Self { the_result })
    }

    /// GC_MakeRotation(Point, Direc, Angle): the rotation by `angle` about
    /// (`point`, `direction`). `MakeRotation::from_point_direction(p, d, 1.0)?`.
    pub fn from_point_direction(
        point: Point3,
        direction: Direction3,
        angle: f64,
    ) -> Result<Self, Failure> {
        Self::from_axis(Axis1::new(point, direction), angle)
    }
}

/// GC_MakeRotation2d: the Geom2d rotation about a point.
///
/// `let rotation = MakeRotation2d::new(Point2::default(), 1.0).value();`
#[derive(Clone, Default)]
pub struct MakeRotation2d {
    the_result: Handle<Geom2dTransformation>,
}
gc_transformation!(MakeRotation2d, Geom2dTransformation, "GC_MakeRotation2d");

impl MakeRotation2d {
    /// GC_MakeRotation2d(Point, Angle): the rotation by `angle` about `point`.
    /// `MakeRotation2d::new(p, 1.0)`.
    pub fn new(point: Point2, angle: f64) -> Self {
        Self {
            the_result: transformation2(|t| t.set_rotation(point, angle)),
        }
    }
}

/// GC_MakeScale: the Geom scaling about a point.
///
/// `let scale = MakeScale::new(Point3::default(), 2.0)?.value();`
#[derive(Clone, Default)]
pub struct MakeScale {
    the_result: Handle<GeomTransformation>,
}
gc_transformation!(MakeScale, GeomTransformation, "GC_MakeScale");

impl MakeScale {
    /// GC_MakeScale(Point, Scale): the scaling by `scale` about `point`;
    /// Standard_ConstructionError for a null factor. `MakeScale::new(p, 2.0)?`.
    pub fn new(point: Point3, scale: f64) -> Result<Self, Failure> {
        let the_result = try_transformation3(|t| t.set_scale(point, scale))?;
        Ok(Self { the_result })
    }
}

/// GC_MakeScale2d: the Geom2d scaling about a point.
///
/// `let scale = MakeScale2d::new(Point2::default(), 2.0).value();`
#[derive(Clone, Default)]
pub struct MakeScale2d {
    the_result: Handle<Geom2dTransformation>,
}
gc_transformation!(MakeScale2d, Geom2dTransformation, "GC_MakeScale2d");

impl MakeScale2d {
    /// GC_MakeScale2d(Point, Scale): the scaling by `scale` about `point`.
    /// `MakeScale2d::new(p, 2.0)`.
    pub fn new(point: Point2, scale: f64) -> Self {
        Self {
            the_result: transformation2(|t| t.set_scale(point, scale)),
        }
    }
}

/// GC_MakeTranslation: the Geom translation by a vector or between two points.
///
/// `let translation = MakeTranslation::new(Vector3::new(1.0, 0.0, 0.0)).value();`
#[derive(Clone, Default)]
pub struct MakeTranslation {
    the_result: Handle<GeomTransformation>,
}
gc_transformation!(MakeTranslation, GeomTransformation, "GC_MakeTranslation");

impl MakeTranslation {
    /// GC_MakeTranslation(Vec): the translation by `vector`.
    /// `MakeTranslation::new(v)`.
    pub fn new(vector: Vector3) -> Self {
        Self {
            the_result: transformation3(|t| t.set_translation(vector)),
        }
    }

    /// GC_MakeTranslation(Point1, Point2): the translation from `p1` to `p2`
    /// (`gp_Trsf::SetTranslation(P1, P2)`). `MakeTranslation::from_points(a, b)`.
    pub fn from_points(p1: Point3, p2: Point3) -> Self {
        Self {
            the_result: transformation3(|t| t.set_translation_between(p1, p2)),
        }
    }
}

/// GC_MakeTranslation2d: the Geom2d translation by a vector or between two
/// points.
///
/// `let translation = MakeTranslation2d::new(Vector2::new(1.0, 0.0)).value();`
#[derive(Clone, Default)]
pub struct MakeTranslation2d {
    the_result: Handle<Geom2dTransformation>,
}
gc_transformation!(
    MakeTranslation2d,
    Geom2dTransformation,
    "GC_MakeTranslation2d"
);

impl MakeTranslation2d {
    /// GC_MakeTranslation2d(Vec): the translation by `vector`.
    /// `MakeTranslation2d::new(v)`.
    pub fn new(vector: Vector2) -> Self {
        Self {
            the_result: transformation2(|t| t.set_translation(vector)),
        }
    }

    /// GC_MakeTranslation2d(Point1, Point2): the translation from `p1` to `p2`.
    /// `MakeTranslation2d::from_points(a, b)`.
    pub fn from_points(p1: Point2, p2: Point2) -> Self {
        Self {
            the_result: transformation2(|t| t.set_translation_between(p1, p2)),
        }
    }
}
