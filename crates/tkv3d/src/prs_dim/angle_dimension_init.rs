//! `PrsDim_AngleDimension` geometry: the center and side points of the angle between two
//! linear edges, two faces or the generatrices of a cone.
use super::angle_dimension::PrsDimAngleDimension;
use super::dimension::PrsDimDimensionMembers;
use super::tool_geometry::null_curve;
use super::{PrsDim, PrsDimKindOfSurface};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topods::{self, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::*;
use rustycad_tkgeombase::gce::MakeCone;
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkgeombase::proj_lib;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{self, Axis3, Circle3, Line3, Plane3, Point3, Vector3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape};
use std::f64::consts::PI;

/// The line of a linear edge.
fn edge_line(shape: &rustycad_tkbrep::topods::Shape) -> Result<Option<Line3>, Failure> {
    let adaptor = BRepAdaptorCurve::with_edge(&topods::edge(shape)?)?;
    if adaptor.get_type()? != CurveType::Line {
        return Ok(None);
    }
    Ok(Some(adaptor.line()?))
}

/// The circle of an iso-V curve of a surface (`down_cast<Geom_Circle>(VIso(v))->Circ()`).
fn v_iso_circle(surface: &Handle<dyn GeomSurface>, v: f64) -> Result<Circle3, Failure> {
    let curve = surface.get().ok_or_else(null_curve)?.v_iso(v)?;
    curve
        .cast::<GeomCircle>()
        .get()
        .ok_or_else(null_curve)?
        .circ()
}

impl PrsDimAngleDimension {
    fn store_points(&self, center: Point3, first: Point3, second: Point3) {
        let mut f = self.fields.write();
        f.center_point = center;
        f.first_point = first;
        f.second_point = second;
    }
    fn valid_stored_points(&self) -> bool {
        let (first, center, second) = self.points();
        self.is_valid_points(&first, &center, &second)
    }

    /// `InitTwoEdgesAngle`: the angle between two linear edges (a straight angle for
    /// parallel ones) and the plane computed for them.
    /// Example: `dimension.init_two_edges_angle()?`.
    pub fn init_two_edges_angle(&self) -> Result<(bool, Plane3), Failure> {
        let (first_shape, second_shape) = (self.first_shape(), self.second_shape());
        let (Some(first_lin), Some(second_lin)) =
            (edge_line(&first_shape)?, edge_line(&second_shape)?)
        else {
            return Ok((false, Plane3::default()));
        };
        let (d1, d2) = (first_lin.direction(), second_lin.direction());
        let parallel = d1.is_parallel(d2, precision::angular());
        let plane = if parallel {
            Plane3::from_position(Axis3::from_axis2(gp::xoy()))
        } else {
            Plane3::new(
                second_lin.location(),
                rustycad_tkmath::gp::Direction3::from_vector(
                    Vector3::from(d1).crossed(Vector3::from(d2)),
                )?,
            )?
        };
        let edges =
            PrsDim::compute_two_edges(&topods::edge(&first_shape)?, &topods::edge(&second_shape)?)?;
        let Some(edges) = edges else {
            return Ok((false, plane));
        };
        let ((first1, last1), (first2, last2)) = (edges.first_ends, edges.second_ends);
        let tol = precision::confusion();
        let same_lines = d1.is_equal(&d2, precision::angular())
            && first_lin.location().is_equal(&second_lin.location(), tol);
        let same_edges = (first1.is_equal(&first2, tol) && last1.is_equal(&last2, tol))
            || (first1.is_equal(&last2, tol) && last1.is_equal(&first2, tol));
        let flyout = self.get_flyout().abs();
        if parallel {
            if same_lines && same_edges {
                return Ok((false, plane));
            }
            let p11 = elclib::parameter(&first_lin, first1)?;
            let p12 = elclib::parameter(&first_lin, last1)?;
            let p21 = elclib::parameter(&first_lin, first2)?;
            let p22 = elclib::parameter(&first_lin, last2)?;
            let center = elclib::value((p11.min(p12) + p21.max(p22)) * 0.5, &first_lin);
            let first = center.translated(Vector3::from(d1) * flyout);
            let second_dir = if d1.is_equal(&d2, precision::angular()) {
                d1.reversed()
            } else {
                d2
            };
            let second = Point3::from(center.xyz() + second_dir.xyz() * flyout);
            self.store_points(center, first, second);
            return Ok((self.valid_stored_points(), plane));
        }
        let lin1 = proj_lib::project_line_on_plane(&plane, &first_lin)?;
        let lin2 = proj_lib::project_line_on_plane(&plane, &second_lin)?;
        let intersection = AnaIntersection::lines(lin1, lin2);
        if !intersection.is_done() || intersection.is_empty()? {
            return Ok((false, plane));
        }
        let point2d = intersection.point(1)?.value();
        let center = elclib::to3d_point(plane.position().ax2()?, point2d);
        if edges.first_infinite || edges.second_infinite {
            let first = center.translated(Vector3::from(d1) * flyout);
            let second = center.translated(Vector3::from(d2) * flyout);
            self.store_points(center, first, second);
            return Ok((self.valid_stored_points(), plane));
        }
        let farther = |a: Point3, b: Point3| {
            if center.distance(&a) > center.distance(&b) {
                a
            } else {
                b
            }
        };
        self.store_points(center, farther(first1, last1), farther(first2, last2));
        Ok((self.valid_stored_points(), plane))
    }

    /// `InitTwoFacesAngle`: the angle between two planar or curvilinear faces, with the
    /// first attach point on the first face when given.
    /// Example: `dimension.init_two_faces_angle(None)?`.
    pub fn init_two_faces_angle(&self, point: Option<Point3>) -> Result<bool, Failure> {
        let first = topods::face(&self.first_shape())?;
        let second = topods::face(&self.second_shape())?;
        let (_, first_surface) = PrsDim::get_plane_from_face(&first)?;
        let (_, second_surface) = PrsDim::get_plane_from_face(&second)?;
        if let Some(point) = point {
            self.fields.write().first_point = point;
        }
        let found = if first_surface.kind == PrsDimKindOfSurface::Plane
            && second_surface.kind == PrsDimKindOfSurface::Plane
        {
            PrsDim::init_angle_between_planar_faces(&first, &second, point)?
        } else {
            PrsDim::init_angle_between_curvilinear_faces(
                &first,
                &second,
                first_surface.kind,
                second_surface.kind,
                self.center_point(),
                point,
            )?
        };
        let Some([center, first_attach, second_attach]) = found else {
            return Ok(false);
        };
        self.store_points(center, first_attach, second_attach);
        Ok(self.valid_stored_points())
    }

    /// `InitConeAngle`: the apex of a conical face (or of a revolved line) and the points at
    /// 0 and pi on a circle of the face.
    /// Example: `dimension.init_cone_angle()?`.
    pub fn init_cone_angle(&self) -> Result<bool, Failure> {
        let shape = self.first_shape();
        if shape.is_null() {
            return Ok(false);
        }
        let cone_face: Face = topods::face(&shape)?;
        let adaptor = BRepAdaptorSurface::with_face(&cone_face, true)?;
        let max_v = adaptor.first_v_parameter()?;
        let min_v = adaptor.last_v_parameter()?;
        let (_, face_surface) = PrsDim::get_plane_from_face(&cone_face)?;
        let mut surface = face_surface.surface.clone();
        let center = if face_surface.kind == PrsDimKindOfSurface::Revolution {
            let revolution = surface.cast::<GeomSurfaceOfRevolution>();
            let revolution = revolution.get().ok_or_else(null_curve)?;
            let axis = revolution.axis();
            let line = Line3::from_axis(axis);
            let basis = revolution.basis_curve();
            let basis = basis.get().ok_or_else(null_curve)?;
            if !std::ptr::eq(basis.dynamic_type(), GeomLine::static_type()) {
                return Ok(false);
            }
            let first1 = adaptor.value(0.0, min_v)?;
            let last1 = adaptor.value(0.0, max_v)?;
            let vec1 = Vector3::between(first1, last1);
            let first2 = elclib::value(elclib::parameter(&line, first1)?, &line);
            let last2 = elclib::value(elclib::parameter(&line, last1)?, &line);
            let vec2 = Vector3::between(first2, last2);
            if vec1.is_parallel(vec2, precision::angular())?
                || vec1.is_normal(vec2, precision::angular())?
            {
                return Ok(false);
            }
            MakeCone::from_axis_points(axis, first1, last1)?
                .value()?
                .apex()
        } else {
            let is_offset = surface.is_instance(GeomOffsetSurface::static_type());
            if is_offset || face_surface.offset > 0.01 {
                let offset = GeomOffsetSurface::new(&surface, face_surface.offset, false)?;
                surface = offset.surface()?;
                let maker = BRepBuilderAPIMakeFace::from_surface(&surface, precision::confusion())?;
                if !maker.is_done() {
                    return Ok(false);
                }
                adaptor.initialize(&maker.face()?, true)?;
            }
            adaptor.cone()?;
            let conical = surface.cast::<GeomConicalSurface>();
            conical.get().ok_or_else(null_curve)?.apex()
        };
        let circle = v_iso_circle(&surface, (min_v + max_v) / 2.5)?;
        // The native code also reads the circles at both V bounds, unused.
        v_iso_circle(&surface, max_v)?;
        v_iso_circle(&surface, min_v)?;
        self.store_points(
            center,
            elclib::value(0.0, &circle),
            elclib::value(PI, &circle),
        );
        Ok(true)
    }
}
