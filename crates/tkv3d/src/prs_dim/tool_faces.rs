//! `PrsDim` package functions on faces: the plane and kind of a face surface, the attach
//! points of angles between planar or curvilinear faces and of lengths between curvilinear
//! faces.
use super::tool_geometry::{SQUARE_TOLERANCE, null_curve};
use super::{PrsDim, PrsDimKindOfSurface};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_tools::uv_bounds;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::*;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkgeomalgo::geom_api::{
    ExtremaCurveCurve, IntSS, ProjectPointOnCurve, ProjectPointOnSurf,
};
use rustycad_tkgeombase::extrema::ExtAlgo;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Direction3, Line3, Plane3, Point2, Point3, Vector3};
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;

/// `GetPlaneFromFace` outputs: the plane, the (basis) surface, its kind and the offset.
#[derive(Clone)]
pub struct PrsDimFaceSurface {
    /// The plane of a planar (or linear extrusion) face.
    pub plane: Plane3,
    /// The surface.
    pub surface: Handle<dyn GeomSurface>,
    /// The kind of surface.
    pub kind: PrsDimKindOfSurface,
    /// The offset of an offset surface.
    pub offset: f64,
}

fn is_type(
    surface: &Handle<dyn GeomSurface>,
    kind: &'static rustycad_tkernel::standard::RuntimeType,
) -> bool {
    surface
        .get()
        .is_some_and(|s| std::ptr::eq(s.dynamic_type(), kind))
}

fn surface_kind(surface: &Handle<dyn GeomSurface>) -> PrsDimKindOfSurface {
    use PrsDimKindOfSurface as K;
    for (kind, value) in [
        (GeomCylindricalSurface::static_type(), K::Cylinder),
        (GeomConicalSurface::static_type(), K::Cone),
        (GeomSphericalSurface::static_type(), K::Sphere),
        (GeomToroidalSurface::static_type(), K::Torus),
        (GeomSurfaceOfRevolution::static_type(), K::Revolution),
        (GeomSurfaceOfLinearExtrusion::static_type(), K::Extrusion),
    ] {
        if is_type(surface, kind) {
            return value;
        }
    }
    K::OtherSurface
}

fn offset_of(surface: &Handle<dyn GeomSurface>) -> Result<Handle<GeomOffsetSurface>, Failure> {
    let offset = surface.cast::<GeomOffsetSurface>();
    if offset.is_null() {
        return Err(null_curve());
    }
    Ok(offset)
}

impl PrsDim {
    /// `GetPlaneFromFace`: the plane of a planar face (or of a linear extrusion of a line),
    /// else the kind of its (basis) surface; true for a plane.
    /// Example: `PrsDim::get_plane_from_face(&face)?`.
    pub fn get_plane_from_face(face: &Face) -> Result<(bool, PrsDimFaceSurface), Failure> {
        let adaptor = BRepAdaptorSurface::with_face(face, true)?;
        let is_offset = adaptor.get_type()? == SurfaceType::OffsetSurface;
        let basis: Handle<dyn Adaptor3dSurface> = if is_offset {
            adaptor.basis_surface()?
        } else {
            Handle::new(BRepAdaptorSurface::with_face(face, true)?).cast()
        };
        let basis = basis.get().ok_or_else(null_curve)?;
        let mut out = PrsDimFaceSurface {
            plane: Plane3::default(),
            surface: adaptor.transformed_surface().geom_surface_transformed()?,
            kind: PrsDimKindOfSurface::OtherSurface,
            offset: 0.0,
        };
        let mut result = false;
        if basis.get_type()? == SurfaceType::Plane {
            out.plane = basis.plane()?;
            out.kind = PrsDimKindOfSurface::Plane;
            result = true;
        } else if basis.get_type()? == SurfaceType::SurfaceOfExtrusion {
            let curve = basis.basis_curve()?;
            let direction = basis.direction()?;
            let curve = curve.get().ok_or_else(null_curve)?;
            if curve.get_type()? == CurveType::Line {
                let line = curve.line()?;
                out.plane = Plane3::new(line.location(), line.direction().crossed(direction)?)?;
                out.kind = PrsDimKindOfSurface::Plane;
                result = true;
            }
        }
        if result && is_offset {
            out.surface = offset_of(&out.surface)?
                .get()
                .ok_or_else(null_curve)?
                .surface()?;
            out.plane = out
                .surface
                .cast::<GeomPlane>()
                .get()
                .ok_or_else(null_curve)?
                .pln();
        }
        if !result {
            if is_offset {
                let offset = offset_of(&out.surface)?;
                let offset = offset.get().ok_or_else(null_curve)?;
                let canonical = [
                    GeomCylindricalSurface::static_type(),
                    GeomConicalSurface::static_type(),
                    GeomSphericalSurface::static_type(),
                    GeomToroidalSurface::static_type(),
                ];
                if canonical.iter().any(|kind| is_type(&out.surface, kind)) {
                    out.surface = offset.surface()?;
                } else {
                    out.offset = offset.offset();
                    out.surface = offset.basis_surface();
                }
            }
            out.kind = surface_kind(&out.surface);
        }
        Ok((result, out))
    }

    /// `InitFaceLength`: as `GetPlaneFromFace`, with a planar offset face turned into an
    /// offset surface of offset zero; `PrsDim::init_face_length(&face)?`.
    pub fn init_face_length(face: &Face) -> Result<PrsDimFaceSurface, Failure> {
        let (is_plane, mut out) = Self::get_plane_from_face(face)?;
        if is_plane && out.offset.abs() > precision::confusion() {
            out.surface =
                Handle::new(GeomOffsetSurface::new(&out.surface, out.offset, false)?).cast();
            out.offset = 0.0;
        }
        Ok(out)
    }

    /// `InitAngleBetweenPlanarFaces`: the center on the intersection line of two planar
    /// faces and the attach points one unit along each face; `None` without a line.
    /// Example: `PrsDim::init_angle_between_planar_faces(&f1, &f2, None)?`.
    pub fn init_angle_between_planar_faces(
        first: &Face,
        second: &Face,
        first_attach: Option<Point3>,
    ) -> Result<Option<[Point3; 3]>, Failure> {
        let plane1 = BRepTool::surface(first)?.cast::<GeomPlane>();
        let plane2 = BRepTool::surface(second)?.cast::<GeomPlane>();
        let surface1: Handle<dyn GeomSurface> = plane1.clone().cast();
        let surface2: Handle<dyn GeomSurface> = plane2.clone().cast();
        let intersector = IntSS::with_surfaces(&surface1, &surface2, precision::confusion())?;
        if !intersector.is_done() || intersector.nb_lines() != 1 {
            return Ok(None);
        }
        let curve = intersector.line(1)?;
        let Some(line) = curve.cast::<GeomLine>().get().map(|l| l.lin()) else {
            return Ok(None);
        };
        let (p1, p2) = (
            plane1.get().ok_or_else(null_curve)?,
            plane2.get().ok_or_else(null_curve)?,
        );
        let (u1min, u1max, v1min, v1max) = uv_bounds(first)?;
        let (u2min, u2max, v2min, v2max) = uv_bounds(second)?;
        let first_center = match first_attach {
            Some(attach) => {
                let projector = ProjectPointOnSurf::with_surface(attach, &surface1, ExtAlgo::Grad)?;
                if !projector.is_done() {
                    return Ok(None);
                }
                projector.point(1)?
            }
            None => p1.value((u1min + u1max) * 0.5, (v1min + v1max) * 0.5)?,
        };
        let second_center = p2.value((u2min + u2max) * 0.5, (v2min + v2max) * 0.5)?;
        let center = ProjectPointOnCurve::with_curve(first_center, &curve)?.nearest_point()?;
        let attach = |plane: &GeomPlane, face_center: Point3| -> Result<Point3, Failure> {
            let mut normal =
                Vector3::from(line.direction().crossed(plane.pln().axis().direction())?);
            if normal.dot(Vector3::between(center, face_center)) < 0.0 {
                normal.reverse();
            }
            Ok(center.translated(normal))
        };
        Ok(Some([
            center,
            attach(p1, first_center)?,
            attach(p2, second_center)?,
        ]))
    }

    /// `InitAngleBetweenCurvilinearFaces`: the center and attach points of the angle between
    /// a cylinder or cone and a second cylinder or cone (`center` is the input center the
    /// native code keeps for coincident rulings); `None` when not computed.
    /// Example: `PrsDim::init_angle_between_curvilinear_faces(&f1, &f2, k1, k2, center, None)?`.
    pub fn init_angle_between_curvilinear_faces(
        first: &Face,
        second: &Face,
        first_kind: PrsDimKindOfSurface,
        second_kind: PrsDimKindOfSurface,
        center: Point3,
        first_attach: Option<Point3>,
    ) -> Result<Option<[Point3; 3]>, Failure> {
        let surface1 = BRepTool::surface(first)?;
        let surface2 = BRepTool::surface(second)?;
        let intersector = IntSS::with_surfaces(&surface1, &surface2, precision::confusion())?;
        if !intersector.is_done() || intersector.nb_lines() != 1 {
            return Ok(None);
        }
        let s1 = surface1.get().ok_or_else(null_curve)?;
        let first_u = 0.0;
        let mut attach1 = match first_attach {
            Some(point) => {
                let projector = ProjectPointOnSurf::with_surface(point, &surface1, ExtAlgo::Grad)?;
                if !projector.is_done() {
                    return Ok(None);
                }
                projector.point(1)?
            }
            None => s1.value(first_u, 0.0)?,
        };
        let line1 = s1.u_iso(first_u)?.cast::<GeomLine>();
        let Some(second_u) = second_iso_parameter(&surface2, second_kind, attach1)? else {
            return Ok(None);
        };
        let s2 = surface2.get().ok_or_else(null_curve)?;
        let line2 = s2.u_iso(second_u)?.cast::<GeomLine>();
        let (l1, l2) = (
            line1.get().ok_or_else(null_curve)?.lin(),
            line2.get().ok_or_else(null_curve)?.lin(),
        );
        let (d1, d2) = (l1.direction(), l2.direction());
        let mut center = center;
        if d1.is_equal(&d2, precision::angular()) || d1.is_opposite(d2, precision::angular()) {
            let direction = Vector3::from(d1);
            return Ok(Some([
                center,
                center.translated(direction),
                center.translated(direction.reversed()),
            ]));
        }
        let extrema = ExtremaCurveCurve::with_curves(&line1.clone().cast(), &line2.clone().cast())?;
        let (point, _) = extrema.points(1)?;
        center = point;
        if center.square_distance(&attach1) <= SQUARE_TOLERANCE {
            attach1 = center.translated(Vector3::from(d1));
            let Some(on_surface) = reattach(&surface1, first_kind, first_u, attach1)? else {
                return Ok(None);
            };
            attach1 = on_surface;
        }
        let projector = ProjectPointOnSurf::with_surface(attach1, &surface2, ExtAlgo::Grad)?;
        if !projector.is_done() {
            return Ok(None);
        }
        let (u, v) = projector.lower_distance_parameters()?;
        Ok(Some([center, attach1, s2.value(u, v)?]))
    }

    /// `InitLengthBetweenCurvilinearFaces`: the attach points of a length between two
    /// curvilinear faces and the direction in the plane of the dimension.
    /// Example: `PrsDim::init_length_between_curvilinear_faces(&f1, &f2, &s1, &s2)?`.
    pub fn init_length_between_curvilinear_faces(
        first: &Face,
        second: &Face,
        first_surface: &Handle<dyn GeomSurface>,
        second_surface: &Handle<dyn GeomSurface>,
    ) -> Result<(Point3, Point3, Direction3), Failure> {
        let explorer = Explorer::new(first, ShapeEnum::Vertex, ShapeEnum::Shape)?;
        if !explorer.more() {
            return Err(null_curve());
        }
        let mut attach1 = BRepTool::pnt(&topods::vertex(explorer.current())?)?;
        let mut projector =
            ProjectPointOnSurf::with_surface(attach1, first_surface, ExtAlgo::Grad)?;
        // The nearest point is overwritten by the D1 point, as natively (it still fails
        // without a projection).
        projector.nearest_point()?;
        let (mut u, mut v) = projector.lower_distance_parameters()?;
        let s1 = first_surface.get().ok_or_else(null_curve)?;
        let mut d1 = s1.d1(u, v)?;
        attach1 = d1.point;
        if d1.d1u.square_magnitude() <= SQUARE_TOLERANCE
            || d1.d1v.square_magnitude() <= SQUARE_TOLERANCE
        {
            attach1 = Self::farest(first, &attach1)?;
            projector.init(attach1, first_surface, ExtAlgo::Grad)?;
            (u, v) = projector.lower_distance_parameters()?;
            d1 = s1.d1(u, v)?;
            attach1 = d1.point;
        }
        let du = d1.d1u.normalized()?;
        let dv = d1.d1v.normalized()?;
        let dir_on_plane = Direction3::from_vector(du)?;
        let normal1 = Direction3::from_vector(du.crossed(dv))?;
        projector.init(attach1, second_surface, ExtAlgo::Grad)?;
        let s2 = second_surface.get().ok_or_else(null_curve)?;
        let mut best = (0, f64::MAX);
        for index in 1..=projector.nb_points()? {
            let (pu, pv) = projector.parameters(index)?;
            let d = s2.d1(pu, pv)?;
            let local = if d.d1u.square_magnitude() <= SQUARE_TOLERANCE
                || d.d1v.square_magnitude() <= SQUARE_TOLERANCE
            {
                Direction3::from_vector(Vector3::between(attach1, projector.point(index)?))?
            } else {
                Direction3::from_vector(d.d1u.crossed(d.d1v))?
            };
            let distance = projector.distance(index)?;
            if normal1.is_parallel(local, precision::angular()) && distance < best.1 {
                best = (index, distance);
            }
        }
        if best.0 == 0 {
            return Ok((attach1, attach1, dir_on_plane));
        }
        let mut attach2 = projector.point(best.0)?;
        let (pu, pv) = projector.parameters(best.0)?;
        let classifier = BRepTopAdaptorFClass2d::new(second, precision::confusion())?;
        let periodic = s2.is_u_periodic() || s2.is_v_periodic();
        let state = classifier.perform(Point2::new(pu, pv), periodic)?;
        if state == State::Out || state == State::Unknown {
            attach2 = Self::nearest_vertex(second, &attach2)?;
        }
        Ok((attach1, attach2, dir_on_plane))
    }
}

/// The U parameter of the second surface's ruling through the first attach point.
fn second_iso_parameter(
    surface: &Handle<dyn GeomSurface>,
    kind: PrsDimKindOfSurface,
    attach: Point3,
) -> Result<Option<f64>, Failure> {
    let toward = |axis: rustycad_tkmath::gp::Axis1| -> Result<Direction3, Failure> {
        let foot = PrsDim::project_point_on_line(&attach, &Line3::from_axis(axis));
        Direction3::from_xyz(attach.xyz() - foot.xyz()).map_err(|_| {
            Failure::with_message("StdFail_NotDone", "gce_MakeDir::Value() - no result")
        })
    };
    match kind {
        PrsDimKindOfSurface::Cylinder => {
            let cylinder = surface.cast::<GeomCylindricalSurface>();
            let cylinder = cylinder.get().ok_or_else(null_curve)?.cylinder()?;
            Ok(Some(
                cylinder
                    .x_axis()
                    .direction()
                    .angle(&toward(cylinder.axis())?),
            ))
        }
        PrsDimKindOfSurface::Cone => {
            let cone = surface.cast::<GeomConicalSurface>();
            let cone = cone.get().ok_or_else(null_curve)?.cone()?;
            let x = cone.x_axis().direction();
            let to_attach = toward(cone.axis())?;
            let mut u = x.angle(&to_attach);
            if !x.is_equal(&to_attach, precision::angular())
                && !x.is_opposite(to_attach, precision::angular())
                && Vector3::from(x)
                    .crossed(Vector3::from(to_attach))
                    .dot(Vector3::from(cone.axis().direction()))
                    < 0.0
            {
                u = 2.0 * std::f64::consts::PI - u;
            }
            Ok(Some(u))
        }
        _ => Ok(None),
    }
}

/// The point of the first surface ruling at `u` at the height of `point`.
fn reattach(
    surface: &Handle<dyn GeomSurface>,
    kind: PrsDimKindOfSurface,
    u: f64,
    point: Point3,
) -> Result<Option<Point3>, Failure> {
    match kind {
        PrsDimKindOfSurface::Cylinder => {
            let cylinder = surface.cast::<GeomCylindricalSurface>();
            let cylinder = cylinder.get().ok_or_else(null_curve)?.cylinder()?;
            let (_, v) = elslib::parameters(&cylinder, point);
            Ok(Some(elslib::value(u, v, &cylinder)))
        }
        PrsDimKindOfSurface::Cone => {
            let cone = surface.cast::<GeomConicalSurface>();
            let cone = cone.get().ok_or_else(null_curve)?.cone()?;
            let (_, v) = elslib::parameters(&cone, point);
            Ok(Some(elslib::value(u, v, &cone)))
        }
        _ => Ok(None),
    }
}
