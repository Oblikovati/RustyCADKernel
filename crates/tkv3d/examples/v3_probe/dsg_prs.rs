//! DsgPrs dimension and relation presentations through the recording display driver, the
//! twin of tools/oracle/tkv3d_probe_dsg_prs.cxx.
use super::common::{Reader, attempt, dir, end, flag, int, pnt, real, text};
use super::sensitive_display::display_driver::{DisplayTrace, SensitiveDisplayDriver};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkg3d::geom::{
    GeomCurve, GeomEllipse, GeomOffsetCurve, GeomPlane, GeomSphericalSurface, GeomSurface,
};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis3, Circle3, Direction3, Ellipse3, Line3, Plane3, Point3,
};
use rustycad_tkservice::graphic3d::*;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon,
};
use rustycad_tkv3d::dsg_prs::*;
use rustycad_tkv3d::prs3d::Prs3dDrawer;
use rustycad_tkv3d::v3d::V3dViewer;

use super::ais_line::fixture as units_fixture;

/// The uniform request record.
struct DsgInput {
    kind: i32,
    arrow: f64,
    side: DsgPrsArrowSide,
    p: [Point3; 5],
    d: [Direction3; 3],
    v: [f64; 4],
    f: [bool; 2],
}

fn arrow_side(value: i32) -> DsgPrsArrowSide {
    use DsgPrsArrowSide::*;
    [
        None,
        FirstAr,
        LastAr,
        BothAr,
        FirstPt,
        LastPt,
        BothPt,
        FirstArLastPt,
        FirstPtLastAr,
    ][value as usize]
}

fn read_input(r: &mut Reader<'_>) -> DsgInput {
    let (kind, arrow, side) = (r.integer(), r.real(), arrow_side(r.integer()));
    let p = std::array::from_fn(|_| r.pnt());
    let d = std::array::from_fn(|_| r.dir());
    let v = std::array::from_fn(|_| r.real());
    let f = std::array::from_fn(|_| r.flag());
    DsgInput {
        kind,
        arrow,
        side,
        p,
        d,
        v,
        f,
    }
}

fn trace_texts(trace: &DisplayTrace) {
    let trace = trace.read();
    int(trace.texts.len() as i32);
    for t in &trace.texts {
        let text_value = t.text();
        let units = text_value.to_c_string();
        let length = units.iter().position(|u| *u == 0).unwrap_or(units.len());
        text(&units[..length]);
        pnt(t.position());
        real(f64::from(t.height()));
    }
}

fn circle_out(c: &Circle3) {
    pnt(c.location());
    dir(c.axis().direction());
    dir(c.x_axis().direction());
    real(c.radius());
}

fn package_case(i: &DsgInput) {
    match i.kind {
        100 => {
            let mut out = DsgPrsPlanarLength {
                end_of_arrow1: Point3::new(9., 9., 9.),
                end_of_arrow2: Point3::new(8., 8., 8.),
                dir_of_arrow1: Direction3::new(0., 1., 0.).unwrap(),
            };
            attempt(|| {
                DsgPrs::compute_planar_faces_length_presentation(
                    i.v[0],
                    i.v[1],
                    &i.p[0],
                    &i.p[1],
                    &i.d[0],
                    &i.p[2],
                    &Plane3::new(i.p[3], i.d[1])?,
                    &mut out,
                )
            });
            pnt(out.end_of_arrow1);
            pnt(out.end_of_arrow2);
            dir(out.dir_of_arrow1);
        }
        101 => faces_case(i),
        102 => fillet_case(i),
        103 => {
            let mut ends = (Point3::new(1., 2., 3.), Point3::new(4., 5., 6.));
            attempt(|| {
                ends = DsgPrs::compute_radius_line(&i.p[0], &i.p[1], &i.p[2], i.f[0])?;
                Ok(())
            });
            pnt(ends.0);
            pnt(ends.1);
        }
        104 => attempt(|| {
            let ellipse = Ellipse3::new(
                Axis2::with_x_direction(i.p[0], i.d[0], i.d[1])?,
                i.v[0],
                i.v[1],
            )?;
            real(DsgPrs::distance_from_apex(&ellipse, &i.p[1], i.v[2])?);
            Ok(())
        }),
        _ => curvilinear_case(i),
    }
}

fn faces_case(i: &DsgInput) {
    let mut out = DsgPrsFacesAngle {
        first_par_angle_circ: 7.,
        last_par_angle_circ: 7.,
        end_of_arrow1: Point3::new(1., 1., 1.),
        end_of_arrow2: Point3::new(2., 2., 2.),
        dir_of_arrow1: Direction3::new(1., 0., 0.).unwrap(),
        dir_of_arrow2: Direction3::new(0., 1., 0.).unwrap(),
        proj_attach_point2: Point3::new(3., 3., 3.),
        first_par_attach_circ: 6.,
        last_par_attach_circ: 6.,
        ..DsgPrsFacesAngle::default()
    };
    let inputs = faces_inputs(i);
    attempt(|| DsgPrs::compute_faces_angle_presentation(&inputs, &mut out));
    circle_out(&out.angle_circ);
    real(out.first_par_angle_circ);
    real(out.last_par_angle_circ);
    pnt(out.end_of_arrow1);
    pnt(out.end_of_arrow2);
    dir(out.dir_of_arrow1);
    dir(out.dir_of_arrow2);
    pnt(out.proj_attach_point2);
    circle_out(&out.attach_circ);
    real(out.first_par_attach_circ);
    real(out.last_par_attach_circ);
}

fn faces_inputs(i: &DsgInput) -> DsgPrsFacesAngleInputs {
    DsgPrsFacesAngleInputs {
        arrow_length: i.arrow,
        value: i.v[0],
        center_point: i.p[0],
        attachment_point1: i.p[1],
        attachment_point2: i.p[2],
        dir1: i.d[0],
        dir2: i.d[1],
        axis_dir: i.d[2],
        is_plane: i.f[0],
        axis_of_surf: Axis1::new(i.p[4], i.d[2]),
        offset_point: i.p[3],
    }
}

fn fillet_inputs(i: &DsgInput) -> DsgPrsFilletInputs {
    DsgPrsFilletInputs {
        arrow_length: i.arrow,
        value: i.v[0],
        position: i.p[0],
        normal_dir: i.d[0],
        first_point: i.p[1],
        second_point: i.p[2],
        center: i.p[3],
        base_pnt: i.p[4],
        draw_revers: i.f[0],
    }
}

fn fillet_case(i: &DsgInput) {
    let mut out = DsgPrsFillet {
        first_par_circ: 5.,
        last_par_circ: 5.,
        end_of_arrow: Point3::new(4., 4., 4.),
        dir_of_arrow: Direction3::new(0., 0., 1.).unwrap(),
        draw_position: Point3::new(5., 5., 5.),
        ..DsgPrsFillet::default()
    };
    attempt(|| DsgPrs::compute_fillet_radius_presentation(&fillet_inputs(i), &mut out));
    flag(out.spec_case);
    circle_out(&out.fillet_circ);
    real(out.first_par_circ);
    real(out.last_par_circ);
    pnt(out.end_of_arrow);
    dir(out.dir_of_arrow);
    pnt(out.draw_position);
}

fn curvilinear_case(i: &DsgInput) {
    let mut out = DsgPrsCurvilinearLength {
        end_of_arrow2: Point3::new(7., 7., 7.),
        dir_of_arrow1: Direction3::new(1., 0., 0.).unwrap(),
        first_u: 1.,
        delta_u: 2.,
        first_v: 3.,
        delta_v: 4.,
        ..DsgPrsCurvilinearLength::default()
    };
    attempt(|| {
        let surface: Handle<dyn GeomSurface> = Handle::new(GeomSphericalSurface::new(
            Axis3::from_axis2(Axis2::new(i.p[4], i.d[2])?),
            i.v[0],
        )?)
        .cast();
        DsgPrs::compute_curvilinear_faces_length_presentation(
            i.v[1], i.v[2], &surface, &i.p[0], &i.p[1], &i.d[0], &mut out,
        )
    });
    pnt(out.end_of_arrow2);
    dir(out.dir_of_arrow1);
    flag(out.v_curve.is_null());
    flag(out.u_curve.is_null());
    real(out.first_u);
    real(out.delta_u);
    real(out.first_v);
    real(out.delta_v);
}

/// `v3_dsgprs`.
pub fn dsg_prs_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let i = read_input(&mut r);
    if i.kind >= 100 {
        package_case(&i);
        end();
        return;
    }
    let (driver, trace) = SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    let drawer = Handle::new(Prs3dDrawer::new());
    drawer.setup_own_defaults().unwrap();
    let aspect = drawer.dimension_aspect();
    aspect.arrow_aspect().set_length(i.arrow);
    attempt(|| call(&prs.clone().cast(), &drawer, &i));
    display_events(&trace);
    trace_texts(&trace);
    int(prs.groups().len() as i32);
    real(aspect.arrow_aspect().length());
    int(aspect.line_aspect().aspect().line_kind() as i32);
    end();
}

fn circ(i: &DsgInput, c: usize, n: usize, radius: f64) -> Result<Circle3, Failure> {
    Circle3::new(Axis2::new(i.p[c], i.d[n])?, radius)
}

fn elips(i: &DsgInput) -> Result<Ellipse3, Failure> {
    Ellipse3::new(
        Axis2::with_x_direction(i.p[2], i.d[0], i.d[1])?,
        i.v[1],
        i.v[2],
    )
}

fn plane(i: &DsgInput, c: usize, n: usize) -> Result<Handle<GeomPlane>, Failure> {
    Ok(Handle::new(GeomPlane::from_point(i.p[c], i.d[n])?))
}

#[allow(clippy::too_many_lines)] // One dispatch arm per native overload, as in the C++ twin.
fn call(prs: &DsgPrsPresentation, dr: &Handle<Prs3dDrawer>, i: &DsgInput) -> Result<(), Failure> {
    let t = ExtendedString::new("12.5");
    let (p, d, v, f, s) = (&i.p, &i.d, &i.v, &i.f, i.side);
    match i.kind {
        0 => DsgPrsChamf2dPresentation::add(prs, dr, &p[0], &p[1], &t),
        1 => DsgPrsChamf2dPresentation::add_with_arrow(prs, dr, &p[0], &p[1], &t, s),
        2 => DsgPrsConcentricPresentation::add(prs, dr, &p[0], v[0], &d[0], &p[1]),
        3 => DsgPrsDatumPrs::add(prs, &Axis2::with_x_direction(p[0], d[0], d[1])?, dr),
        4 => DsgPrsDiameterPresentation::add(prs, dr, &t, &p[0], &circ(i, 1, 0, v[0])?, s, f[0]),
        5 => DsgPrsDiameterPresentation::add_arc(
            prs,
            dr,
            &t,
            &p[0],
            &circ(i, 1, 0, v[0])?,
            v[1],
            v[2],
            s,
            f[0],
        ),
        6 => DsgPrsEllipseRadiusPresentation::add(prs, dr, v[0], &t, &p[0], &p[1], &p[2], f[0], s),
        7 => DsgPrsEllipseRadiusPresentation::add_ellipse(
            prs,
            dr,
            v[0],
            &t,
            &elips(i)?,
            &p[0],
            &p[1],
            &p[2],
            v[3],
            f[1],
            f[0],
            s,
        ),
        8 => {
            let basis: Handle<dyn GeomCurve> =
                Handle::new(GeomEllipse::from_elips(elips(i)?)).cast();
            let offset = Handle::new(GeomOffsetCurve::new(&basis, 0.5, d[0], true)?);
            DsgPrsEllipseRadiusPresentation::add_offset_curve(
                prs, dr, v[0], &t, &offset, &p[0], &p[1], &p[2], v[3], f[1], f[0], s,
            )
        }
        9 => DsgPrsEqualDistancePresentation::add(
            prs,
            dr,
            &p[0],
            &p[1],
            &p[2],
            &p[3],
            &plane(i, 4, 0)?,
        ),
        10 => {
            let (a, b) = DsgPrsEqualDistancePresentation::add_interval(
                prs, dr, &p[0], &p[1], &d[0], &p[2], s,
            )?;
            pnt(a);
            pnt(b);
            Ok(())
        }
        11 => {
            let (c1, c2) = (circ(i, 0, 0, v[0])?, circ(i, 1, 0, v[1])?);
            let points = [
                elclib::value(v[2], &c1),
                elclib::value(v[3], &c1),
                elclib::value(v[2], &c2),
                elclib::value(v[3], &c2),
            ];
            DsgPrsEqualDistancePresentation::add_interval_between_two_arcs(
                prs, dr, &c1, &c2, &points[0], &points[1], &points[2], &points[3], s,
            )
        }
        12 => DsgPrsEqualRadiusPresentation::add(
            prs,
            dr,
            &p[0],
            &p[1],
            &p[2],
            &p[3],
            &plane(i, 4, 0)?,
        ),
        13 => fillet_presentation(prs, dr, i, &t),
        14 => DsgPrsFixPresentation::add(prs, dr, &p[0], &p[1], &d[0], v[0]),
        15 => DsgPrsIdenticPresentation::add(prs, dr, &t, &p[0], &p[1]),
        16 => DsgPrsIdenticPresentation::add_segment(prs, dr, &t, &p[0], &p[1], &p[2]),
        17 => DsgPrsIdenticPresentation::add_arc(
            prs,
            dr,
            &t,
            &Axis2::new(p[4], d[0])?,
            &p[0],
            &p[1],
            &p[2],
            &p[3],
        ),
        18 => DsgPrsIdenticPresentation::add_arc_point(
            prs,
            dr,
            &t,
            &Axis2::new(p[4], d[0])?,
            &p[0],
            &p[1],
            &p[2],
            &p[3],
            &p[4],
        ),
        19 => DsgPrsIdenticPresentation::add_ellipse(
            prs,
            dr,
            &t,
            &elips(i)?,
            &p[0],
            &p[1],
            &p[3],
            &p[4],
        ),
        20 => DsgPrsLengthPresentation::add(prs, dr, &t, &p[0], &p[1], &d[0], &p[2]),
        21 => DsgPrsLengthPresentation::add_with_arrow(prs, dr, &t, &p[0], &p[1], &d[0], &p[2], s),
        22 => DsgPrsLengthPresentation::add_planar(
            prs,
            dr,
            &t,
            &p[0],
            &p[1],
            &Plane3::new(p[3], d[1])?,
            &d[0],
            &p[2],
            s,
        ),
        23 => {
            let surface: Handle<dyn GeomSurface> = Handle::new(GeomSphericalSurface::new(
                Axis3::from_axis2(Axis2::new(p[4], d[2])?),
                v[0],
            )?)
            .cast();
            DsgPrsLengthPresentation::add_curvilinear(
                prs, dr, &t, &surface, &p[0], &p[1], &d[0], &p[2], s,
            )
        }
        24 => DsgPrsLengthPresentation::add_segment(prs, dr, &p[0], &p[1], s),
        25 => DsgPrsMidPointPresentation::add(
            prs,
            dr,
            &Axis2::new(p[0], d[0])?,
            &p[1],
            &p[2],
            &p[3],
            f[0],
        ),
        26 => DsgPrsMidPointPresentation::add_segment(
            prs,
            dr,
            &Axis2::new(p[0], d[0])?,
            &p[1],
            &p[2],
            &p[3],
            &p[4],
            &p[0],
            f[0],
        ),
        27 => DsgPrsMidPointPresentation::add_circle(
            prs,
            dr,
            &circ(i, 0, 0, v[0])?,
            &p[1],
            &p[2],
            &p[3],
            &p[4],
            &p[0],
            f[0],
        ),
        28 => DsgPrsMidPointPresentation::add_ellipse(
            prs,
            dr,
            &elips(i)?,
            &p[1],
            &p[0],
            &p[3],
            &p[4],
            &p[1],
            f[0],
        ),
        29 => DsgPrsOffsetPresentation::add(prs, dr, &t, &p[0], &p[1], &d[0], &d[1], &p[2]),
        30 => DsgPrsOffsetPresentation::add_axes(prs, dr, &t, &p[0], &p[1], &d[0], &d[1], &p[2]),
        31 => DsgPrsParalPresentation::add(prs, dr, &t, &p[0], &p[1], &d[0], &p[2]),
        32 => DsgPrsParalPresentation::add_with_arrow(prs, dr, &t, &p[0], &p[1], &d[0], &p[2], s),
        33 => DsgPrsPerpenPresentation::add(prs, dr, &p[0], &p[1], &p[2], &p[3], &p[4], f[0], f[1]),
        34 => DsgPrsRadiusPresentation::add(
            prs,
            dr,
            &t,
            &p[0],
            &circ(i, 1, 0, v[0])?,
            v[1],
            v[2],
            f[0],
            f[1],
        ),
        35 => DsgPrsRadiusPresentation::add_center(prs, dr, &t, &p[0], &p[1], &p[2], s, f[0], f[1]),
        36 => DsgPrsSymmetricPresentation::add_edges(
            prs,
            dr,
            &p[0],
            &p[1],
            &d[0],
            &Line3::new(p[2], d[1]),
            &p[3],
        ),
        37 => DsgPrsSymmetricPresentation::add_arcs(
            prs,
            dr,
            &p[0],
            &p[1],
            &circ(i, 4, 0, v[0])?,
            &Line3::new(p[2], d[1]),
            &p[3],
        ),
        38 => DsgPrsSymmetricPresentation::add_vertices(
            prs,
            dr,
            &p[0],
            &p[1],
            &Line3::new(p[2], d[1]),
            &p[3],
        ),
        39 => DsgPrsTangentPresentation::add(prs, dr, &p[0], &d[0], v[0]),
        40 => DsgPrsSymbPresentation::add(prs, dr, &t, &p[0]),
        41 => DsgPrsAnglePresentation::add_value(
            prs, dr, v[0], &p[0], &p[1], &p[2], &d[0], &d[1], &p[3],
        ),
        42 => DsgPrsAnglePresentation::add(
            prs, dr, v[0], &t, &p[0], &p[1], &p[2], &d[0], &d[1], &p[3],
        ),
        43 => DsgPrsAnglePresentation::add_axis(
            prs, dr, v[0], &t, &p[0], &p[1], &p[2], &d[0], &d[1], &d[2], &p[3],
        ),
        44 => DsgPrsAnglePresentation::add_with_arrow(
            prs, dr, v[0], &t, &p[0], &p[1], &p[2], &d[0], &d[1], &p[3], s,
        ),
        45 => DsgPrsAnglePresentation::add_faces(prs, dr, &t, &faces_inputs(i), s),
        46 => DsgPrsAnglePresentation::add_revolution(
            prs,
            dr,
            v[0],
            &p[0],
            &p[1],
            &Axis1::new(p[0], d[0]),
            s,
        ),
        47 => cone_case(prs, dr, i, &t),
        49 => {
            let edge = BRepBuilderAPIMakeEdge::from_points(p[0], p[1])?.edge()?;
            DsgPrsShapeDirPresentation::add(prs, dr, &edge.into_shape(), v[3] as i32)
        }
        50..=52 => shape_dir_case(prs, dr, i),
        _ => {
            let ends = DsgPrsSymbolEnds {
                first: p[0],
                last: p[1],
                first_direction: d[0],
                last_direction: d[1],
            };
            dsg_prs_compute_symbol(prs, &dr.dimension_aspect(), &ends, s, f[0])
        }
    }
}

fn fillet_presentation(
    prs: &DsgPrsPresentation,
    dr: &Handle<Prs3dDrawer>,
    i: &DsgInput,
    t: &ExtendedString,
) -> Result<(), Failure> {
    let mut out = DsgPrsFilletOutputs {
        draw_position: Point3::new(1., 1., 1.),
        end_of_arrow: Point3::new(2., 2., 2.),
        has_circle: true,
        ..DsgPrsFilletOutputs::default()
    };
    let inputs = DsgPrsFilletInputs {
        position: i.p[0],
        normal_dir: i.d[0],
        base_pnt: i.p[1],
        first_point: i.p[2],
        second_point: i.p[3],
        center: i.p[4],
        draw_revers: i.f[0],
        ..fillet_inputs(i)
    };
    DsgPrsFilletRadiusPresentation::add(prs, dr, i.v[0], t, &inputs, i.side, &mut out)?;
    pnt(out.draw_position);
    pnt(out.end_of_arrow);
    flag(out.has_circle);
    flag(out.trim_curve.is_null());
    if let Some(curve) = out.trim_curve.get() {
        real(curve.first_parameter());
        real(curve.last_parameter());
    }
    Ok(())
}

fn polygon_wire(a: Point3, b: Point3, c: Point3) -> Result<rustycad_tkbrep::topods::Wire, Failure> {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for point in [a, b, c] {
        polygon.add_point(point)?;
    }
    polygon.close()?;
    polygon.wire()
}

fn planar_face(
    wire: &rustycad_tkbrep::topods::Wire,
) -> Result<rustycad_tkbrep::topods::Shape, Failure> {
    Ok(BRepBuilderAPIMakeFace::from_wire(wire, true)?
        .face()?
        .into_shape())
}

fn shape_dir_case(
    prs: &DsgPrsPresentation,
    dr: &Handle<Prs3dDrawer>,
    i: &DsgInput,
) -> Result<(), Failure> {
    let p = &i.p;
    let wire = polygon_wire(p[0], p[1], p[2])?;
    let mut shape = wire.clone().into_shape();
    if i.kind > 50 {
        shape = planar_face(&wire)?;
    }
    if i.kind > 51 {
        let builder = rustycad_tkbrep::brep::BRepBuilder::default();
        let mut compound = rustycad_tkbrep::topods::Compound::default();
        builder.make_compound(&mut compound);
        builder.add(&mut compound, &shape)?;
        builder.add(
            &mut compound,
            &planar_face(&polygon_wire(p[2], p[3], p[4])?)?,
        )?;
        shape = compound.into_shape();
    }
    if i.f[1] {
        shape.reverse();
    }
    DsgPrsShapeDirPresentation::add(prs, dr, &shape, i.v[3] as i32)
}

fn cone_case(
    prs: &DsgPrsPresentation,
    dr: &Handle<Prs3dDrawer>,
    i: &DsgInput,
    t: &ExtendedString,
) -> Result<(), Failure> {
    let (p, d, v) = (&i.p, &i.d, &i.v);
    let cone = DsgPrsConeAngle {
        circle: Circle3::new(Axis2::new(p[0], d[0])?, v[0])?,
        position: p[3],
        apex: p[4],
        v_min_circle: Circle3::new(Axis2::new(p[1], d[0])?, v[1])?,
        v_max_circle: Circle3::new(Axis2::new(p[2], d[0])?, v[2])?,
        arrow_size: v[3],
    };
    let text_value = if i.f[0] {
        ExtendedString::default()
    } else {
        t.clone()
    };
    DsgPrsAnglePresentation::add_cone(prs, dr, 0.0, &text_value, &cone, &units_fixture::units())
}

fn display_events(trace: &DisplayTrace) {
    let trace = trace.read();
    int(trace.events.len() as i32);
    for value in &trace.events {
        real(*value);
    }
}
