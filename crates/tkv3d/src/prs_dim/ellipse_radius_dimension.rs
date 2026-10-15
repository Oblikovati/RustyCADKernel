//! `PrsDim_EllipseRadiusDimension`: the abstract radius of an ellipse (an edge, a planar face
//! bounded by an ellipse, or an extruded elliptic face), with the ellipse, its arc parameters
//! and an offset curve for offset faces.
use super::relation_support::null_object;
use super::*;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, SharedState};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::*;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Axis1, Axis3, Ellipse3, Plane3, Point3};
use std::f64::consts::PI;

/// The fields of `PrsDim_EllipseRadiusDimension` and of its concrete max/min radii.
#[derive(Clone, Default)]
pub struct PrsDimEllipseRadiusFields {
    /// `myEllipse`.
    pub ellipse: Ellipse3,
    /// `myFirstPar`.
    pub first_par: f64,
    /// `myLastPar`.
    pub last_par: f64,
    /// `myIsAnArc`.
    pub is_an_arc: bool,
    /// `myOffsetCurve`.
    pub offset_curve: Handle<GeomOffsetCurve>,
    /// `myOffset`.
    pub offset: f64,
    /// `myIsOffset`.
    pub is_offset: bool,
    /// `myApexP` (max/min radius).
    pub apex_p: Point3,
    /// `myApexN` (max/min radius).
    pub apex_n: Point3,
    /// `myEndOfArrow` (max/min radius).
    pub end_of_arrow: Point3,
}

fn construction(message: &str) -> Failure {
    Failure::with_message("Standard_ConstructionError", message)
}

/// `PrsDim_EllipseRadiusDimension` (abstract): the ellipse geometry of the measured shape.
pub trait PrsDimEllipseRadiusDimension: PrsDimRelationSource {
    /// The ellipse fields.
    fn ellipse_fields(&self) -> &SharedState<PrsDimEllipseRadiusFields>;

    /// `ComputeGeometry`: the ellipse of the face or edge with arc parameters in [0, 2 pi].
    /// Example: `dimension.compute_geometry()?`.
    fn compute_geometry(&self) -> Result<(), Failure> {
        match self.first_shape().shape_type()? {
            ShapeEnum::Face => self.compute_face_geometry()?,
            ShapeEnum::Edge => self.compute_edge_geometry()?,
            _ => {}
        }
        let mut guard = self.ellipse_fields().write();
        let f = &mut *guard;
        for parameter in [&mut f.first_par, &mut f.last_par] {
            while *parameter > 2.0 * PI {
                *parameter -= 2.0 * PI;
            }
        }
        for parameter in [&mut f.first_par, &mut f.last_par] {
            while *parameter < 0.0 {
                *parameter += 2.0 * PI;
            }
        }
        Ok(())
    }

    /// `ComputeFaceGeometry`: a planar face, else an extruded face.
    /// Example: `dimension.compute_face_geometry()?`.
    fn compute_face_geometry(&self) -> Result<(), Failure> {
        let face = topods::face(&self.first_shape())?;
        let (_, surface) = PrsDim::get_plane_from_face(&face)?;
        if surface.kind == PrsDimKindOfSurface::Plane {
            self.compute_planar_face_geometry()
        } else {
            self.compute_cyl_face_geometry(surface.kind, &surface.surface, surface.offset)
        }
    }

    /// `ComputeCylFaceGeometry`: the ellipse at the middle iso of an extruded face (with its
    /// offset), the plane normal to the extrusion; other surfaces are ignored.
    /// Example: `dimension.compute_cyl_face_geometry(kind, &surface, 0.0)?`.
    fn compute_cyl_face_geometry(
        &self,
        kind: PrsDimKindOfSurface,
        basis: &Handle<dyn GeomSurface>,
        offset: f64,
    ) -> Result<(), Failure> {
        let face = topods::face(&self.first_shape())?;
        let adaptor = BRepAdaptorSurface::with_face(&face, true)?;
        let middle = (adaptor.first_v_parameter()? + adaptor.last_v_parameter()?) * 0.5;
        if kind != PrsDimKindOfSurface::Extrusion {
            return Ok(());
        }
        let extrusion = basis.cast::<GeomSurfaceOfLinearExtrusion>();
        let direction = extrusion.get().ok_or_else(null_object)?.direction();
        let mut f = self.ellipse_fields().read().clone();
        let mut plane = Plane3::default();
        plane.set_axis(Axis1::new(Point3::from(direction.xyz()), direction))?;
        plane.set_location(f.ellipse.location());
        self.set_plane(&Handle::new(GeomPlane::from_pln(plane)));
        let curve = basis.get().ok_or_else(null_object)?.v_iso(middle)?;
        let value = curve.get().ok_or_else(null_object)?;
        if std::ptr::eq(value.dynamic_type(), GeomEllipse::static_type()) {
            f.ellipse = curve
                .cast::<GeomEllipse>()
                .get()
                .ok_or_else(null_object)?
                .elips()?;
            f.is_an_arc = false;
        } else if std::ptr::eq(value.dynamic_type(), GeomTrimmedCurve::static_type()) {
            let trimmed = curve.cast::<GeomTrimmedCurve>();
            let trimmed = trimmed.get().ok_or_else(null_object)?;
            let basis_curve = trimmed.basis_curve();
            f.first_par = trimmed.first_parameter();
            f.last_par = trimmed.last_parameter();
            f.is_an_arc = true;
            if let Some(ellipse) = basis_curve.get()
                && std::ptr::eq(ellipse.dynamic_type(), GeomEllipse::static_type())
            {
                f.ellipse = basis_curve
                    .cast::<GeomEllipse>()
                    .get()
                    .ok_or_else(null_object)?
                    .elips()?;
            }
        } else {
            return Err(construction("PrsDim:: Not expected type of surface"));
        }
        if adaptor.get_type()? == SurfaceType::OffsetSurface {
            if offset < 0.0 && offset.abs() > f.ellipse.minor_radius() {
                return Err(construction(
                    "PrsDim:: Absolute value of negative offset is larger than MinorRadius",
                ));
            }
            let normal = self
                .plane()
                .get()
                .ok_or_else(null_object)?
                .pln()
                .axis()
                .direction();
            let ellipse: Handle<dyn GeomCurve> =
                Handle::new(GeomEllipse::from_elips(f.ellipse)).cast();
            f.offset_curve = Handle::new(GeomOffsetCurve::new(&ellipse, offset, normal, false)?);
            f.offset = offset;
            f.is_offset = true;
            let original = f.ellipse;
            f.ellipse
                .set_major_radius(offset + original.major_radius())?;
            f.ellipse
                .set_minor_radius(offset + original.minor_radius())?;
        } else {
            f.is_offset = false;
        }
        *self.ellipse_fields().write() = f;
        Ok(())
    }

    /// `ComputePlanarFaceGeometry`: the first elliptic edge of a planar face (a
    /// `Standard_ConstructionError` without one) and the face plane.
    /// Example: `dimension.compute_planar_face_geometry()?`.
    fn compute_planar_face_geometry(&self) -> Result<(), Failure> {
        let face = topods::face(&self.first_shape())?;
        let mut found = None;
        for edge in Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let edge = topods::edge(&edge)?;
            if let Some((curve, ends)) = PrsDim::compute_edge_geometry(&edge)?
                && let Some(value) = curve.get()
                && std::ptr::eq(value.dynamic_type(), GeomEllipse::static_type())
            {
                let ellipse = curve
                    .cast::<GeomEllipse>()
                    .get()
                    .ok_or_else(null_object)?
                    .elips()?;
                found = Some((ellipse, ends));
                break;
            }
        }
        let Some((ellipse, (first, last))) = found else {
            return Err(construction("PrsDim:: Curve is not an ellipsee or is Null"));
        };
        {
            let mut f = self.ellipse_fields().write();
            f.ellipse = ellipse;
            f.is_an_arc = !first.is_equal(&last, precision::confusion());
            if f.is_an_arc {
                f.first_par = elclib::parameter(&ellipse, first)?;
                f.last_par = elclib::parameter(&ellipse, last)?;
            }
        }
        let surface = BRepAdaptorSurface::with_face(&face, true)?;
        self.set_plane(&Handle::new(GeomPlane::from_pln(surface.plane()?)));
        Ok(())
    }

    /// `ComputeEdgeGeometry`: the ellipse of an elliptic edge and its plane.
    /// Example: `dimension.compute_edge_geometry()?`.
    fn compute_edge_geometry(&self) -> Result<(), Failure> {
        let edge = topods::edge(&self.first_shape())?;
        let Some((curve, (first, last))) = PrsDim::compute_edge_geometry(&edge)? else {
            return Ok(());
        };
        let ellipse = curve.cast::<GeomEllipse>();
        let Some(ellipse) = ellipse.get() else {
            return Ok(());
        };
        let ellipse = ellipse.elips()?;
        let mut f = self.ellipse_fields().write();
        f.ellipse = ellipse;
        let plane = Plane3::from_position(Axis3::from_axis2(ellipse.position()));
        f.is_an_arc = !first.is_equal(&last, precision::confusion());
        if f.is_an_arc {
            f.first_par = elclib::parameter(&ellipse, first)?;
            f.last_par = elclib::parameter(&ellipse, last)?;
        }
        drop(f);
        self.set_plane(&Handle::new(GeomPlane::from_pln(plane)));
        Ok(())
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn PrsDimEllipseRadiusDimension,
    "PrsDim_EllipseRadiusDimension",
    dyn PrsDimRelationSource
);

/// The shape and text of a new ellipse radius (the protected constructor
/// `PrsDim_EllipseRadiusDimension(shape, text)`); `init_ellipse_radius(&*radius, &edge, &text)`.
pub fn init_ellipse_radius<R: PrsDimRelationSource + ?Sized>(
    relation: &R,
    shape: &Shape,
    text: &rustycad_tkernel::tcollection::ExtendedString,
) {
    let mut f = relation.relation_base().fields.write();
    f.f_shape = shape.clone();
    f.text = text.clone();
}
