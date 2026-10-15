//! PrsDim dimensions in a context with a recording driver, the twin of
//! tools/oracle/tkv3d_probe_prs_dim.cxx.
use super::common::{Reader, attempt, end, flag, int, real};
use super::sensitive_display::display_driver::SensitiveDisplayDriver;
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomSurface, GeomSurfaceOfRevolution};
use rustycad_tkmath::gp::{
    self, Axis2, Axis3, Circle3, Cone3, Cylinder3, Direction3, Line3, Plane3, Point3,
};
use rustycad_tkservice::aspect::AspectNeutralWindow;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeWire;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeVertex,
};
use rustycad_tkv3d::ais::*;
use rustycad_tkv3d::prs_dim::*;
use rustycad_tkv3d::prs_mgr::*;
use rustycad_tkv3d::prs3d::*;
use rustycad_tkv3d::select_mgr::*;
use rustycad_tkv3d::select3d::Select3DSensitiveEntityMembers;
use rustycad_tkv3d::v3d::{V3dViewSource, V3dViewer};
use std::f64::consts::PI;

fn edge(a: [f64; 3], b: [f64; 3]) -> Shape {
    let e: Edge = BRepBuilderAPIMakeEdge::from_points(
        Point3::new(a[0], a[1], a[2]),
        Point3::new(b[0], b[1], b[2]),
    )
    .unwrap()
    .edge()
    .unwrap();
    e.into()
}
fn face(z: f64, size: f64) -> Face {
    BRepBuilderAPIMakeFace::from_plane_bounds(
        Plane3::new(Point3::new(0.0, 0.0, z), gp::dz()).unwrap(),
        (0.0, size, 0.0, size),
    )
    .unwrap()
    .face()
    .unwrap()
}
fn vertex(x: f64, y: f64, z: f64) -> Shape {
    BRepBuilderAPIMakeVertex::new(Point3::new(x, y, z))
        .unwrap()
        .vertex()
        .unwrap()
        .into()
}
fn plane(index: i32) -> Plane3 {
    match index {
        1 => Plane3::from_position(gp::Axis3::from_axis2(gp::zox())),
        2 => Plane3::from_position(gp::Axis3::from_axis2(gp::yoz())),
        _ => Plane3::from_position(gp::Axis3::from_axis2(gp::xoy())),
    }
}

fn length(variant: i32) -> Result<Handle<dyn PrsDimDimensionSource>, Failure> {
    let as_edge = |s: Shape| rustycad_tkbrep::topods::edge(&s).unwrap();
    let d = match variant {
        0 => PrsDimLengthDimension::from_points(
            &Point3::new(0., 0., 0.),
            &Point3::new(10., 0., 0.),
            &plane(0),
        )?,
        1 => PrsDimLengthDimension::from_points(
            &Point3::new(1., 2., 0.),
            &Point3::new(1., 2., 0.),
            &plane(0),
        )?,
        2 => {
            PrsDimLengthDimension::from_edge(&as_edge(edge([0., 0., 0.], [5., 5., 0.])), &plane(0))?
        }
        3 => PrsDimLengthDimension::from_shapes(
            &edge([0., 0., 0.], [10., 0., 0.]),
            &edge([0., 5., 0.], [10., 5., 0.]),
            &plane(0),
        )?,
        4 => {
            let d = PrsDimLengthDimension::new();
            d.set_measured_shapes(
                &edge([0., 0., 0.], [10., 0., 0.]),
                &edge([2., 5., 0.], [12., 5., 0.]),
            )?;
            d
        }
        5 => {
            let d = PrsDimLengthDimension::new();
            d.set_measured_shapes(&edge([0., 0., 0.], [10., 0., 0.]), &vertex(4., 6., 0.))?;
            d
        }
        6 => PrsDimLengthDimension::from_faces(&face(0.0, 4.0), &face(5.0, 4.0))?,
        7 => PrsDimLengthDimension::from_face_edge(
            &face(0.0, 4.0),
            &as_edge(edge([1., 1., 3.], [3., 1., 3.])),
        )?,
        8 => {
            let d = PrsDimLengthDimension::new();
            d.set_measured_shapes(&vertex(0., 0., 0.), &vertex(3., 4., 0.))?;
            d
        }
        9 => {
            let d = PrsDimLengthDimension::new();
            d.set_measured_shapes(&vertex(4., 6., 0.), &edge([0., 0., 0.], [10., 0., 0.]))?;
            d
        }
        _ => PrsDimLengthDimension::from_shapes(
            &edge([0., 0., 0.], [10., 0., 0.]),
            &edge([0., 0., 3.], [10., 0., 7.]),
            &plane(1),
        )?,
    };
    Ok(d.into_cast())
}

fn circle() -> Circle3 {
    Circle3::new(
        Axis2::new(Point3::new(1.0, 2.0, 0.0), gp::dz()).unwrap(),
        3.0,
    )
    .unwrap()
}
fn circle_edge(first: f64, last: f64) -> Edge {
    BRepBuilderAPIMakeEdge::from_circ_params(circle(), first, last)
        .unwrap()
        .edge()
        .unwrap()
}
fn cylinder() -> Shape {
    let cylinder =
        Cylinder3::new(Axis3::from_normal(gp::origin(), gp::dz()).unwrap(), 2.0).unwrap();
    BRepBuilderAPIMakeFace::from_cylinder_bounds(cylinder, (0.0, 2.0 * PI, 0.0, 4.0))
        .unwrap()
        .face()
        .unwrap()
        .into()
}
fn disk() -> Shape {
    let wire = BRepBuilderAPIMakeWire::from_edge(&circle_edge(0.0, 2.0 * PI))
        .unwrap()
        .wire()
        .unwrap();
    BRepBuilderAPIMakeFace::from_wire(&wire, false)
        .unwrap()
        .face()
        .unwrap()
        .into()
}

fn radius(variant: i32) -> Result<Handle<dyn PrsDimDimensionSource>, Failure> {
    let d = match variant {
        0 => PrsDimRadiusDimension::from_circle(&circle())?,
        1 => PrsDimRadiusDimension::from_circle_anchor(&circle(), &Point3::new(1.0, 5.0, 0.0))?,
        2 => PrsDimRadiusDimension::from_circle_anchor(&circle(), &Point3::new(1.0, 2.0, 0.0))?,
        3 => PrsDimRadiusDimension::from_shape(&circle_edge(0.0, 2.0 * PI).into())?,
        4 => PrsDimRadiusDimension::from_shape(&circle_edge(0.5, 2.0).into())?,
        5 => PrsDimRadiusDimension::from_shape(&disk())?,
        6 => PrsDimRadiusDimension::from_shape(&cylinder())?,
        _ => {
            let zero = Circle3::new(
                Axis2::new(Point3::new(1.0, 2.0, 0.0), gp::dz()).unwrap(),
                0.0,
            )
            .unwrap();
            PrsDimRadiusDimension::from_circle(&zero)?
        }
    };
    Ok(d.into_cast())
}

fn diameter(variant: i32) -> Result<Handle<dyn PrsDimDimensionSource>, Failure> {
    let through = |normal: Direction3| Plane3::new(Point3::new(1.0, 2.0, 0.0), normal).unwrap();
    let d = match variant {
        0 => PrsDimDiameterDimension::from_circle(&circle())?,
        1 => PrsDimDiameterDimension::from_circle_plane(&circle(), &through(gp::dz()))?,
        2 => PrsDimDiameterDimension::from_circle_plane(&circle(), &through(gp::dy()))?,
        3 => PrsDimDiameterDimension::from_shape(&circle_edge(0.0, 2.0 * PI).into())?,
        4 => PrsDimDiameterDimension::from_shape(&circle_edge(0.5, 2.0).into())?,
        5 => PrsDimDiameterDimension::from_shape(&cylinder())?,
        _ => PrsDimDiameterDimension::from_shape_plane(
            &circle_edge(0.0, 2.0 * PI).into(),
            &through(gp::dx()),
        )?,
    };
    Ok(d.into_cast())
}

fn cone(radius: f64) -> Face {
    let cone = Cone3::new(
        Axis3::from_normal(gp::origin(), gp::dz()).unwrap(),
        PI / 6.0,
        radius,
    )
    .unwrap();
    BRepBuilderAPIMakeFace::from_cone_bounds(cone, (0.0, 2.0 * PI, 0.0, 4.0))
        .unwrap()
        .face()
        .unwrap()
}
fn revolution() -> Face {
    let line = Line3::new(
        Point3::new(2.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 1.0).unwrap(),
    );
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_lin(line)).cast();
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomSurfaceOfRevolution::new(&curve, gp::oz()).unwrap()).cast();
    BRepBuilderAPIMakeFace::from_surface_bounds(&surface, (0.0, 2.0 * PI, 0.0, 3.0), 1.0e-7)
        .unwrap()
        .face()
        .unwrap()
}
fn zox_face() -> Face {
    BRepBuilderAPIMakeFace::from_plane_bounds(
        Plane3::from_position(Axis3::from_axis2(gp::zox())),
        (0.0, 4.0, 0.0, 4.0),
    )
    .unwrap()
    .face()
    .unwrap()
}

fn angle(variant: i32) -> Result<Handle<dyn PrsDimDimensionSource>, Failure> {
    let as_edge = |s: Shape| rustycad_tkbrep::topods::edge(&s).unwrap();
    let as_vertex = |s: Shape| rustycad_tkbrep::topods::vertex(&s).unwrap();
    let p = Point3::new;
    let d = match variant {
        0 => PrsDimAngleDimension::from_points(&p(10., 0., 0.), &p(0., 0., 0.), &p(5., 5., 0.))?,
        1 => PrsDimAngleDimension::from_points(&p(10., 0., 0.), &p(0., 0., 0.), &p(-10., 0., 0.))?,
        2 => PrsDimAngleDimension::from_points(&p(0., 0., 0.), &p(0., 0., 0.), &p(5., 5., 0.))?,
        3 => PrsDimAngleDimension::from_vertices(
            &as_vertex(vertex(0., 5., 0.)),
            &as_vertex(vertex(0., 0., 0.)),
            &as_vertex(vertex(3., 0., 4.)),
        )?,
        4 => PrsDimAngleDimension::from_edges(
            &as_edge(edge([2., 0., 0.], [10., 0., 0.])),
            &as_edge(edge([0., 3., 0.], [0., 10., 0.])),
        )?,
        5 => PrsDimAngleDimension::from_edges(
            &as_edge(edge([0., 0., 0.], [10., 0., 0.])),
            &as_edge(edge([10., 5., 0.], [0., 5., 0.])),
        )?,
        6 => PrsDimAngleDimension::from_edges(
            &as_edge(edge([0., 0., 0.], [10., 0., 0.])),
            &as_edge(edge([0., 0., 0.], [10., 0., 0.])),
        )?,
        7 => PrsDimAngleDimension::from_faces(&face(0.0, 4.0), &zox_face())?,
        8 => PrsDimAngleDimension::from_faces_point(&face(0.0, 4.0), &zox_face(), &p(1., 3., 0.))?,
        9 => PrsDimAngleDimension::from_cone(&cone(2.0))?,
        10 => PrsDimAngleDimension::from_cone(&revolution())?,
        11 => {
            let cylinder = rustycad_tkbrep::topods::face(&cylinder()).unwrap();
            PrsDimAngleDimension::from_faces(&cylinder, &cone(1.0))?
        }
        _ => PrsDimAngleDimension::from_edges(
            &circle_edge(0.0, 2.0),
            &as_edge(edge([0., 0., 0.], [10., 0., 0.])),
        )?,
    };
    Ok(d.into_cast())
}

fn make(kind: i32, variant: i32) -> Result<Handle<dyn PrsDimDimensionSource>, Failure> {
    match kind {
        3 => angle(variant),
        1 => radius(variant),
        2 => diameter(variant),
        _ => length(variant),
    }
}

fn point(p: &Point3) {
    real(p.x());
    real(p.y());
    real(p.z());
}
fn dir(d: &Direction3) {
    real(d.x());
    real(d.y());
    real(d.z());
}

fn state(d: &dyn PrsDimDimensionSource) {
    flag(d.is_valid());
    real(d.get_value());
    real(d.get_flyout());
    let p = d.get_plane();
    point(&p.location());
    dir(&p.axis().direction());
    dir(&p.x_axis().direction());
    int(d.get_geometry_type());
    int(d.kind_of_dimension() as i32);
    flag(d.is_text_position_custom());
    flag(d.has_interactive_context());
    if d.has_interactive_context() {
        attempt(|| {
            rustycad_tkernel::standard::catch_native_failure(|| {
                point(&d.get_text_position());
                Ok(())
            })
        });
    }
    int(d.display_special_symbol() as i32);
    int(i32::from(d.special_symbol()));
}

fn selection(d: &Handle<dyn PrsDimDimensionSource>) -> Result<(), Failure> {
    for mode in 0..=3 {
        let s = Handle::new(SelectMgrSelection::new(mode));
        attempt(|| {
            d.compute_selection(&s, mode)?;
            int(s.entities().len() as i32);
            for e in s.entities() {
                let sensitive = e.base_sensitive();
                int(sensitive.nb_sub_elements());
                int(sensitive.owner_id().priority());
                let b = sensitive.bounding_box();
                flag(b.is_valid());
                if b.is_valid() {
                    let (lo, hi) = (b.corner_min(), b.corner_max());
                    for v in [lo.x, lo.y, lo.z, hi.x, hi.y, hi.z] {
                        real(v);
                    }
                }
            }
            Ok(())
        });
    }
    Ok(())
}

/// `v3_prsdim Kind Variant Count (Op A B C)...`.
pub fn prs_dim_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, variant) = (r.integer(), r.integer());
    let (driver, trace) = SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let view: Handle<dyn V3dViewSource> = viewer.create_view().unwrap();
    let window = Handle::new(AspectNeutralWindow::default());
    window.set_size(400, 300);
    view.set_window(&window.cast(), 0).unwrap();
    let ctx = AisInteractiveContext::new(&viewer).unwrap();
    let mut created = None;
    attempt(|| {
        created = Some(make(kind, variant)?);
        Ok(())
    });
    let Some(d) = created else {
        end();
        return;
    };
    state(&*d);
    let aspect = Handle::new(Prs3dDimensionAspect::new().unwrap());
    let object: Handle<dyn AisInteractiveObject> = d.clone().into_cast();
    for _ in 0..r.integer() {
        let (op, a, b, c) = (r.integer(), r.integer(), r.integer(), r.integer());
        match op {
            0 => d.set_flyout(f64::from(a) / 10.0),
            1 => d.set_custom_value(f64::from(a)),
            2 => d.set_computed_value(),
            3 => attempt(|| {
                rustycad_tkernel::standard::catch_native_failure(|| {
                    d.set_text_position(&Point3::new(f64::from(a), f64::from(b), f64::from(c)));
                    Ok(())
                })
            }),
            4 => {
                aspect.set_text_horizontal_position(horizontal(a));
                d.set_dimension_aspect(&aspect);
            }
            5 => {
                aspect.set_text_vertical_position(vertical(a));
                d.set_dimension_aspect(&aspect);
            }
            6 => {
                aspect.set_arrow_orientation(orientation(a));
                d.set_dimension_aspect(&aspect);
            }
            7 => {
                aspect.make_text_3d(a != 0);
                d.set_dimension_aspect(&aspect);
            }
            8 => {
                aspect.make_units_displayed(a != 0);
                d.set_dimension_aspect(&aspect);
            }
            9 => d.set_custom_plane(&plane(a)),
            10 => {
                if let Some(l) = d.clone().into_cast::<PrsDimLengthDimension>().get() {
                    let direction = [gp::dx(), gp::dy(), gp::dz()][a as usize];
                    l.set_direction(&direction, b != 0);
                }
            }
            11 => {
                d.set_display_special_symbol(special(a));
                d.set_special_symbol(b as u16);
            }
            12 => {
                let format = ["%g", "%.2f", "%5.1e"][a as usize];
                aspect.set_value_string_format(&AsciiString::from(format));
                d.set_dimension_aspect(&aspect);
            }
            13 => {
                aspect.make_arrows_3d(a != 0);
                d.set_dimension_aspect(&aspect);
            }
            14 => d.unset_fixed_text_position(),
            15 => d.set_sel_tolerance_for_text_2d(f64::from(a) / 10.0),
            16 => {
                trace.write().events.clear();
                attempt(|| {
                    if ctx.is_displayed(&object) {
                        ctx.redisplay(&object, false, true)?;
                    } else {
                        ctx.display(&object, false)?;
                    }
                    int(d.presentations().len() as i32);
                    let events = trace.read();
                    int(events.events.len() as i32);
                    for value in &events.events {
                        real(*value);
                    }
                    Ok(())
                });
                trace.write().events.clear();
                attempt(|| selection(&d));
            }
            17 => {
                aspect.set_extension_size(f64::from(a) / 10.0);
                aspect.set_arrow_tail_size(f64::from(b) / 10.0);
                d.set_dimension_aspect(&aspect);
            }
            19 => {
                if let Some(g) = d.clone().into_cast::<PrsDimAngleDimension>().get() {
                    g.set_type(angle_type(a));
                }
            }
            20 => {
                if let Some(g) = d.clone().into_cast::<PrsDimAngleDimension>().get() {
                    g.set_arrows_visibility(arrows(a));
                }
            }
            _ => {
                let arrow = aspect.arrow_aspect();
                arrow.set_length(f64::from(a) / 10.0);
                arrow.set_zoomable(b != 0);
                d.set_dimension_aspect(&aspect);
            }
        }
        state(&*d);
    }
    end();
}

fn horizontal(value: i32) -> Prs3dDimensionTextHorizontalPosition {
    use Prs3dDimensionTextHorizontalPosition as H;
    [H::Left, H::Right, H::Center, H::Fit][value as usize]
}
fn vertical(value: i32) -> Prs3dDimensionTextVerticalPosition {
    use Prs3dDimensionTextVerticalPosition as V;
    [V::Above, V::Below, V::Center][value as usize]
}
fn orientation(value: i32) -> Prs3dDimensionArrowOrientation {
    use Prs3dDimensionArrowOrientation as O;
    [O::Internal, O::External, O::Fit][value as usize]
}
fn special(value: i32) -> PrsDimDisplaySpecialSymbol {
    use PrsDimDisplaySpecialSymbol as S;
    [S::No, S::Before, S::After][value as usize]
}
fn angle_type(value: i32) -> PrsDimTypeOfAngle {
    [PrsDimTypeOfAngle::Interior, PrsDimTypeOfAngle::Exterior][value as usize]
}
fn arrows(value: i32) -> PrsDimTypeOfAngleArrowVisibility {
    use PrsDimTypeOfAngleArrowVisibility as A;
    [A::Both, A::First, A::Second, A::None][value as usize]
}
