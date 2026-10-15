//! The shapes and the result printer of the TKFeat probe; the twin of
//! tools/oracle/tkfeat_probe_shapes.cxx.
use super::common::{flag, int, real};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Compound, Face, Shape, Wire};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeShape,
};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_gprop::BRepGProp;

/// `ft_box`: the box of the corner (x, y, z) and the sizes.
pub fn ft_box(x: f64, y: f64, z: f64, dx: f64, dy: f64, dz: f64) -> Result<Shape, Failure> {
    Ok(
        BRepPrimAPIMakeBox::at_point(Point3::new(x, y, z), dx, dy, dz)?
            .shape()?
            .clone(),
    )
}

/// `ft_basis`: 0 the box [0, 10]^3, 1 the cylinder on the axis (5, 5), 2 the plates.
pub fn basis(kind: i32) -> Result<Shape, Failure> {
    if kind == 1 {
        let axes = Axis2::new(Point3::new(5., 5., 0.), Direction3::new(0., 0., 1.)?)?;
        return Ok(BRepPrimAPIMakeCylinder::with_axes(axes, 5., 10.)
            .shape()?
            .clone());
    }
    if kind == 2 {
        let b = BRepBuilder::default();
        let mut c = Compound::default();
        b.make_compound(&mut c);
        let mut c = c.into_shape();
        b.add(&mut c, &ft_box(0., 0., 0., 10., 10., 4.)?)?;
        b.add(&mut c, &ft_box(0., 0., 6., 10., 10., 4.)?)?;
        return Ok(c);
    }
    ft_box(0., 0., 0., 10., 10., 10.)
}

/// The polygon through the points, closed when `close`.
fn polygon(pts: &[(f64, f64, f64)], close: bool) -> Result<Wire, Failure> {
    let mut poly = BRepBuilderAPIMakePolygon::new();
    for p in pts {
        poly.add_point(Point3::new(p.0, p.1, p.2))?;
    }
    if close {
        poly.close()?;
    }
    poly.wire()
}

/// `ft_polygon_face`: the planar face bounded by the closed polygon through the points.
pub fn polygon_face(pts: &[(f64, f64, f64)]) -> Result<Face, Failure> {
    BRepBuilderAPIMakeFace::from_wire(&polygon(pts, true)?, true)?.face()
}

/// `ft_polygon_wire`: the open polygon wire through the points.
pub fn polygon_wire(pts: &[(f64, f64, f64)]) -> Result<Wire, Failure> {
    polygon(pts, false)
}

/// `ft_rect`: the rectangle [x0, x1] x [y0, y1] at the height z.
pub fn rect(x0: f64, y0: f64, x1: f64, y1: f64, z: f64) -> Result<Face, Failure> {
    polygon_face(&[(x0, y0, z), (x1, y0, z), (x1, y1, z), (x0, y1, z)])
}

/// `ft_face_at`: the first planar face of the shape at the height z with a vertical
/// normal.
pub fn face_at(s: &Shape, z: f64) -> Result<Face, Failure> {
    for f in Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)? {
        let f = topods::face(&f)?;
        let a = BRepAdaptorSurface::with_face(&f, true)?;
        if a.get_type()? != SurfaceType::Plane {
            continue;
        }
        let p = a.plane()?;
        if p.location().z() == z && p.axis().direction().z().abs() == 1. {
            return Ok(f);
        }
    }
    Ok(Face::default())
}

/// `ft_count`: the number of sub-shapes of a kind met by an explorer.
pub fn count(s: &Shape, kind: ShapeEnum) -> Result<i32, Failure> {
    Ok(Explorer::new(s, kind, ShapeEnum::Shape)?.count() as i32)
}

/// `ft_shape`: the face and edge counts, volume, area and validity.
pub fn shape(s: &Shape) -> Result<(), Failure> {
    int(count(s, ShapeEnum::Face)?);
    int(count(s, ShapeEnum::Edge)?);
    let mut v = GProps::new();
    BRepGProp::volume_properties(s, &mut v, false, false, false)?;
    real(v.mass());
    let mut a = GProps::new();
    BRepGProp::surface_properties(s, &mut a, false, false)?;
    real(a.mass());
    flag(BRepCheckAnalyzer::new(s, true, false, false)?.is_valid()?);
    Ok(())
}

/// `ft_result`: done, then the shape when done.
pub fn result(api: &mut impl BRepBuilderAPIMakeShape) -> Result<(), Failure> {
    flag(api.is_done());
    if api.is_done() {
        let s = api.shape()?.clone();
        shape(&s)?;
    }
    Ok(())
}
