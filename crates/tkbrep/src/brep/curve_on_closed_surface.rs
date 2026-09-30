// Rust translation of ModelingData/TKBRep/BRep/BRep_CurveOnClosedSurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRep_CurveOnClosedSurface`: the two p-curves of a seam edge.
use super::curve_on_surface::BRepCurveOnSurface;
use super::curve_representation::{BRepCurveRepresentation, CurveRepresentationBase};
use super::dump::{dump_base, dump_class_name, dump_field, dump_handle};
use super::errors::null_object;
use super::gcurve::{BRepGCurve, GCurveBase};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::toploc::Location;
use std::io;

/// The private fields of `BRep_CurveOnClosedSurface`.
#[derive(Clone)]
struct ClosedSurfaceFields {
    pcurve2: Handle<dyn Geom2dCurve>,
    continuity: Continuity,
    uv21: Point2,
    uv22: Point2,
}

/// `BRep_CurveOnClosedSurface`: a curve on a closed surface with its second p-curve
/// and the continuity across the seam. It derives from `BRep_CurveOnSurface`, which
/// it embeds.
///
/// ```
/// use rustycad_tkbrep::brep::{BRepCurveOnClosedSurface, BRepCurveRepresentation};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
/// use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
/// use rustycad_tkmath::geom_abs::Shape;
/// use rustycad_tkmath::gp::{Axis2d, Axis3};
/// use rustycad_tkmath::toploc::Location;
/// let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::default())).into_cast();
/// let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).into_cast();
/// let seam = BRepCurveOnClosedSurface::new(&line, &line, &plane, &Location::default(), Shape::C0).unwrap();
/// assert!(seam.is_curve_on_closed_surface() && seam.is_regularity());
/// ```
pub struct BRepCurveOnClosedSurface {
    parent: BRepCurveOnSurface,
    fields: SharedState<ClosedSurfaceFields>,
}
impl_transient!(BRepCurveOnClosedSurface, "BRep_CurveOnClosedSurface", BRepCurveOnSurface, parent,
    interfaces: [dyn BRepCurveRepresentation, dyn BRepGCurve]);

impl BRepCurveOnClosedSurface {
    /// `BRep_CurveOnClosedSurface(PC1, PC2, S, L, C)`; a null first p-curve raises
    /// Standard_NullObject; `BRepCurveOnClosedSurface::new(&c1, &c2, &surface, &location, Shape::C0)?`.
    pub fn new(
        pcurve1: &Handle<dyn Geom2dCurve>,
        pcurve2: &Handle<dyn Geom2dCurve>,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
        continuity: Continuity,
    ) -> Result<Self, Failure> {
        Ok(Self {
            parent: BRepCurveOnSurface::new(pcurve1, surface, location)?,
            fields: SharedState::new(ClosedSurfaceFields {
                pcurve2: pcurve2.clone(),
                continuity,
                uv21: Point2::default(),
                uv22: Point2::default(),
            }),
        })
    }

    /// The embedded `BRep_CurveOnSurface`; `seam.as_curve_on_surface().uv_points()`.
    pub fn as_curve_on_surface(&self) -> &BRepCurveOnSurface {
        &self.parent
    }

    /// `SetUVPoints(P1, P2)` of the first p-curve (inherited); `seam.set_uv_points(a, b)`.
    pub fn set_uv_points(&self, first: Point2, last: Point2) {
        self.parent.set_uv_points(first, last);
    }

    /// `UVPoints(P1, P2)` of the first p-curve (inherited); `seam.uv_points()`.
    pub fn uv_points(&self) -> (Point2, Point2) {
        self.parent.uv_points()
    }

    /// `SetUVPoints2(P1, P2)`; `seam.set_uv_points2(first, last)`.
    pub fn set_uv_points2(&self, first: Point2, last: Point2) {
        let mut fields = self.fields.write();
        fields.uv21 = first;
        fields.uv22 = last;
    }

    /// `UVPoints2(P1, P2)`; `let (first, last) = seam.uv_points2();`.
    pub fn uv_points2(&self) -> (Point2, Point2) {
        let fields = self.fields.read();
        (fields.uv21, fields.uv22)
    }

    /// The second p-curve's UV points at the finite range ends.
    fn update_uv_points2(&self) -> Result<(), Failure> {
        let (first, last) = self.range();
        let pcurve2 = self.fields.read().pcurve2.clone();
        let curve = pcurve2.get().ok_or_else(|| {
            null_object("BRep_CurveOnClosedSurface::Update: the second p-curve is null")
        })?;
        if !precision::is_negative_infinite(first) {
            self.fields.write().uv21 = curve.d0(first)?;
        }
        if !precision::is_positive_infinite(last) {
            self.fields.write().uv22 = curve.d0(last)?;
        }
        Ok(())
    }
}

impl BRepCurveRepresentation for BRepCurveOnClosedSurface {
    fn curve_representation_base(&self) -> &CurveRepresentationBase {
        self.parent.curve_representation_base()
    }
    fn is_curve_on_surface(&self) -> bool {
        self.parent.is_curve_on_surface()
    }
    fn is_curve_on_surface_at(
        &self,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> bool {
        self.parent.is_curve_on_surface_at(surface, location)
    }
    /// Returns true.
    fn is_curve_on_closed_surface(&self) -> bool {
        true
    }
    /// Returns true.
    fn is_regularity(&self) -> bool {
        true
    }
    /// Both surfaces and both locations are this representation's.
    fn is_regularity_between(
        &self,
        surface1: &Handle<dyn GeomSurface>,
        surface2: &Handle<dyn GeomSurface>,
        location1: &Location,
        location2: &Location,
    ) -> bool {
        let surface = self.parent.surface().unwrap_or_default();
        let location = self.location();
        surface == *surface1
            && surface == *surface2
            && location == *location1
            && location == *location2
    }
    fn surface(&self) -> Result<Handle<dyn GeomSurface>, Failure> {
        self.parent.surface()
    }
    fn pcurve(&self) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        self.parent.pcurve()
    }
    fn set_pcurve(&self, pcurve: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
        self.parent.set_pcurve(pcurve)
    }
    fn pcurve2(&self) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        Ok(self.fields.read().pcurve2.clone())
    }
    fn set_pcurve2(&self, pcurve: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
        self.fields.write().pcurve2 = pcurve.clone();
        Ok(())
    }
    /// Returns `Surface()`.
    fn surface2(&self) -> Result<Handle<dyn GeomSurface>, Failure> {
        self.parent.surface()
    }
    /// Returns `Location()`.
    fn location2(&self) -> Result<Location, Failure> {
        Ok(self.location())
    }
    fn continuity(&self) -> Result<Continuity, Failure> {
        Ok(self.fields.read().continuity)
    }
    fn set_continuity(&self, continuity: Continuity) -> Result<(), Failure> {
        self.fields.write().continuity = continuity;
        Ok(())
    }
    /// A new representation on the same data, then the range and both UV point pairs.
    fn copy(&self) -> Result<Handle<dyn BRepCurveRepresentation>, Failure> {
        let fields = self.fields.read().clone();
        let copy = Self::new(
            &self.parent.pcurve()?,
            &fields.pcurve2,
            &self.parent.surface()?,
            &self.location(),
            fields.continuity,
        )?;
        let (first, last) = self.range();
        copy.set_range(first, last)?;
        let (uv1, uv2) = self.parent.uv_points();
        copy.set_uv_points(uv1, uv2);
        copy.set_uv_points2(fields.uv21, fields.uv22);
        Ok(Handle::new(copy).into_cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_class_name(stream, "BRep_CurveOnClosedSurface")?;
        dump_base(stream, depth, |nested, nested_depth| {
            self.parent.dump_body(nested, nested_depth)
        })?;
        let fields = self.fields.read().clone();
        dump_handle(stream, depth, "PCurve2", &fields.pcurve2, |c, s, d| {
            c.dump_json(s, d)
        })?;
        dump::dump_integer(stream, "Continuity", fields.continuity as i32)?;
        dump_field(stream, depth, "UV21", |nested, _| {
            fields.uv21.dump_json(nested)
        })?;
        dump_field(stream, depth, "UV22", |nested, _| {
            fields.uv22.dump_json(nested)
        })
    }
}

impl BRepGCurve for BRepCurveOnClosedSurface {
    fn gcurve_base(&self) -> &GCurveBase {
        self.parent.gcurve_base()
    }
    fn d0(&self, u: f64) -> Result<Point3, Failure> {
        self.parent.d0(u)
    }
    /// The second p-curve's UV points, then `BRep_CurveOnSurface::Update`.
    fn update(&self) -> Result<(), Failure> {
        self.update_uv_points2()?;
        self.parent.update_uv_points()
    }
}
