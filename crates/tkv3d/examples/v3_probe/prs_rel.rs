//! PrsDim relations in a context with a recording driver, the twin of
//! tools/oracle/tkv3d_probe_prs_rel.cxx.
use super::common::{Reader, attempt, end, flag, int, real};
use super::sensitive_display::display_driver::SensitiveDisplayDriver;
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkmath::gp::{
    self, Axis2, Axis3, Circle3, Cylinder3, Ellipse3, Line3, Plane3, Point3,
};
use rustycad_tkservice::aspect::AspectNeutralWindow;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeVertex,
};
use rustycad_tkv3d::ais::*;
use rustycad_tkv3d::dsg_prs::DsgPrsArrowSide;
use rustycad_tkv3d::prs_dim::*;
use rustycad_tkv3d::prs_mgr::*;
use rustycad_tkv3d::select_mgr::*;
use rustycad_tkv3d::select3d::Select3DSensitiveEntityMembers;
use rustycad_tkv3d::v3d::{V3dViewSource, V3dViewer};
use std::f64::consts::PI;

pub(super) fn edge(a: [f64; 3], b: [f64; 3]) -> Shape {
    let e: Edge = BRepBuilderAPIMakeEdge::from_points(
        Point3::new(a[0], a[1], a[2]),
        Point3::new(b[0], b[1], b[2]),
    )
    .unwrap()
    .edge()
    .unwrap();
    e.into()
}
pub(super) fn square(z: f64, size: f64) -> Shape {
    let face: Face = BRepBuilderAPIMakeFace::from_plane_bounds(
        Plane3::new(Point3::new(0.0, 0.0, z), gp::dz()).unwrap(),
        (0.0, size, 0.0, size),
    )
    .unwrap()
    .face()
    .unwrap();
    face.into()
}
pub(super) fn cylinder() -> Shape {
    let cylinder =
        Cylinder3::new(Axis3::from_normal(gp::origin(), gp::dz()).unwrap(), 2.0).unwrap();
    BRepBuilderAPIMakeFace::from_cylinder_bounds(cylinder, (0.0, PI, 0.0, 4.0))
        .unwrap()
        .face()
        .unwrap()
        .into()
}
pub(super) fn xoy() -> Handle<GeomPlane> {
    Handle::new(GeomPlane::from_pln(Plane3::from_position(
        Axis3::from_axis2(gp::xoy()),
    )))
}
pub(super) fn text(index: i32) -> ExtendedString {
    ExtendedString::from_utf8(["", "C2", "chamfer"][index as usize])
}
fn side(value: i32) -> DsgPrsArrowSide {
    use DsgPrsArrowSide as S;
    [
        S::None,
        S::FirstAr,
        S::LastAr,
        S::BothAr,
        S::FirstPt,
        S::LastPt,
        S::BothPt,
        S::FirstArLastPt,
        S::FirstPtLastAr,
    ][value as usize]
}

type Relation = Handle<dyn PrsDimRelationSource>;

fn chamf2d(variant: i32) -> Relation {
    let e = edge([0., 0., 0.], [6., 8., 0.]);
    let p = Point3::new;
    match variant {
        0 => PrsDimChamf2dDimension::new(&e, &xoy(), 10.0, &text(1)).into_cast(),
        1 => PrsDimChamf2dDimension::new(&e, &xoy(), 120.0, &text(2)).into_cast(),
        2 => PrsDimChamf2dDimension::with_position(
            &e,
            &xoy(),
            10.0,
            &text(1),
            &p(20., 1., 0.),
            side(3),
            3.0,
        )
        .into_cast(),
        _ => PrsDimChamf2dDimension::with_position(
            &e,
            &xoy(),
            10.0,
            &text(0),
            &p(3., 4., 0.),
            side(4),
            0.5,
        )
        .into_cast(),
    }
}
fn chamf3d(variant: i32) -> Relation {
    let p = Point3::new;
    match variant {
        0 => PrsDimChamf3dDimension::new(&square(1.0, 4.0), 10.0, &text(1)).into_cast(),
        1 => PrsDimChamf3dDimension::new(&cylinder(), 60.0, &text(2)).into_cast(),
        2 => PrsDimChamf3dDimension::with_position(
            &square(1.0, 4.0),
            10.0,
            &text(1),
            &p(2., 2., 30.),
            side(5),
            2.0,
        )
        .into_cast(),
        _ => PrsDimChamf3dDimension::with_position(
            &cylinder(),
            10.0,
            &text(0),
            &p(9., 9., 9.),
            side(0),
            12.0,
        )
        .into_cast(),
    }
}

pub(super) fn arc(x: f64, z: f64, radius: f64, first: f64, last: f64) -> Edge {
    let circle = Circle3::new(
        Axis2::new(Point3::new(x, 0.0, z), gp::dz()).unwrap(),
        radius,
    )
    .unwrap();
    BRepBuilderAPIMakeEdge::from_circ_params(circle, first, last)
        .unwrap()
        .edge()
        .unwrap()
}
pub(super) fn vertex(x: f64, y: f64, z: f64) -> Shape {
    BRepBuilderAPIMakeVertex::new(Point3::new(x, y, z))
        .unwrap()
        .vertex()
        .unwrap()
        .into()
}
fn equal_radius(variant: i32) -> Relation {
    let r = match variant {
        0 => PrsDimEqualRadiusRelation::new(
            &arc(0., 0., 3., 0., PI / 2.0),
            &arc(10., 0., 3., 0., PI),
            &xoy(),
        ),
        1 => PrsDimEqualRadiusRelation::new(
            &arc(0., 0., 3., 0., PI / 2.0),
            &arc(10., 2., 3., 1., 4.),
            &xoy(),
        ),
        _ => PrsDimEqualRadiusRelation::new(
            &arc(0., 0., 4., 0., 2.0 * PI),
            &arc(-9., 0., 4., 0., 2.0 * PI),
            &xoy(),
        ),
    };
    r.into_cast()
}
fn concentric(variant: i32) -> Result<Relation, Failure> {
    let e = |x: f64, z: f64, r: f64, a: f64, b: f64| -> Shape { arc(x, z, r, a, b).into() };
    let r = match variant {
        0 => PrsDimConcentricRelation::new(&e(1., 0., 3., 0., PI), &e(1., 0., 5., 1., 3.), &xoy())?,
        1 => PrsDimConcentricRelation::new(&e(1., 0., 3., 0., PI), &vertex(1., 0., 0.), &xoy())?,
        2 => PrsDimConcentricRelation::new(
            &vertex(1., 0., 0.),
            &e(1., 0., 90., 0., 2.0 * PI),
            &xoy(),
        )?,
        3 => PrsDimConcentricRelation::new(&vertex(1., 0., 0.), &vertex(1., 0., 0.), &xoy())?,
        4 => PrsDimConcentricRelation::new(&e(1., 0., 3., 0., PI), &e(1., 4., 5., 1., 3.), &xoy())?,
        5 => PrsDimConcentricRelation::new(&e(1., 0., 3., 0., PI), &vertex(1., 0., 6.), &xoy())?,
        _ => PrsDimConcentricRelation::new(&vertex(1., 2., 3.), &vertex(1., 0., 0.), &xoy())?,
    };
    Ok(r.into_cast())
}

fn ellipse() -> Shape {
    let ellipse = Ellipse3::new(
        Axis2::with_x_direction(Point3::new(0.0, 4.0, 0.0), gp::dz(), gp::dx()).unwrap(),
        5.0,
        3.0,
    )
    .unwrap();
    BRepBuilderAPIMakeEdge::from_elips(ellipse)
        .unwrap()
        .edge()
        .unwrap()
        .into()
}
fn infinite(y: f64) -> Shape {
    BRepBuilderAPIMakeEdge::from_lin(Line3::new(Point3::new(0.0, y, 0.0), gp::dx()))
        .unwrap()
        .edge()
        .unwrap()
        .into()
}
fn perpendicular(variant: i32) -> Relation {
    let r = match variant {
        0 => PrsDimPerpendicularRelation::new(
            &edge([0., 0., 0.], [10., 0., 0.]),
            &edge([3., -2., 0.], [3., 8., 0.]),
            Some(&xoy()),
        ),
        1 => PrsDimPerpendicularRelation::new(
            &edge([0., 0., 0.], [5., 0., 0.]),
            &edge([8., 1., 0.], [8., 9., 0.]),
            Some(&xoy()),
        ),
        2 => PrsDimPerpendicularRelation::new(
            &ellipse(),
            &edge([2., -3., 0.], [2., 12., 0.]),
            Some(&xoy()),
        ),
        3 => PrsDimPerpendicularRelation::new(
            &edge([0., 0., 0.], [10., 0., 0.]),
            &edge([3., -2., 3.], [3., 8., 3.]),
            Some(&xoy()),
        ),
        4 => PrsDimPerpendicularRelation::new(&square(0.0, 4.0), &square(1.0, 4.0), Some(&xoy())),
        5 => PrsDimPerpendicularRelation::new(
            &edge([0., 0., 0.], [10., 0., 0.]),
            &edge([3., -2., 0.], [3., 8., 0.]),
            None,
        ),
        _ => PrsDimPerpendicularRelation::new(
            &infinite(1.0),
            &edge([3., -2., 0.], [3., 8., 0.]),
            Some(&xoy()),
        ),
    };
    r.into_cast()
}
fn parallel(variant: i32) -> Relation {
    let e0 = || edge([0., 0., 0.], [10., 0., 0.]);
    let e1 = || edge([2., 5., 0.], [12., 5., 0.]);
    let r = match variant {
        0 => PrsDimParallelRelation::new(&e0(), &e1(), &xoy()),
        1 => PrsDimParallelRelation::with_position(
            &e0(),
            &e1(),
            &xoy(),
            &Point3::new(5., 9., 0.),
            side(6),
            2.0,
        ),
        2 => PrsDimParallelRelation::new(&ellipse(), &edge([-6., -3., 0.], [7., -3., 0.]), &xoy()),
        3 => PrsDimParallelRelation::new(&e0(), &edge([2., 5., 3.], [12., 5., 3.]), &xoy()),
        4 => PrsDimParallelRelation::new(&infinite(-2.0), &e1(), &xoy()),
        5 => PrsDimParallelRelation::new(&square(0.0, 4.0), &square(1.0, 4.0), &xoy()),
        _ => PrsDimParallelRelation::new(&e0(), &edge([12., 0., 0.], [20., 0., 0.]), &xoy()),
    };
    r.into_cast()
}

fn make(kind: i32, variant: i32) -> Result<Relation, Failure> {
    Ok(match kind {
        4 => perpendicular(variant),
        5 => parallel(variant),
        2 => equal_radius(variant),
        3 => concentric(variant)?,
        1 => chamf3d(variant),
        _ => chamf2d(variant),
    })
}

fn state(r: &dyn PrsDimRelationSource) {
    real(r.value());
    let p = r.position();
    real(p.x());
    real(p.y());
    real(p.z());
    real(r.arrow_size());
    flag(r.automatic_position());
    int(r.symbol_prs() as i32);
    int(r.ext_shape());
    int(r.kind_of_dimension() as i32);
    flag(r.is_movable());
    flag(r.has_color());
    int(r.text().length());
    flag(r.plane().is_null());
}

fn selection(r: &Relation) -> Result<(), Failure> {
    let s = Handle::new(SelectMgrSelection::new(0));
    attempt(|| {
        r.compute_selection(&s, 0)?;
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
    Ok(())
}

fn edit(r: &Relation, op: i32, a: i32, b: i32, c: i32) {
    let (fa, fb, fc) = (f64::from(a), f64::from(b), f64::from(c));
    match op {
        0 => r.set_position(&Point3::new(fa, fb, fc)),
        1 => r.set_arrow_size(fa / 10.0),
        2 => r.set_symbol_prs(side(a)),
        3 => {
            let name = if a == 0 {
                NamedColor::RED
            } else {
                NamedColor::GREEN
            };
            attempt(|| r.set_color(Color::named(name)));
        }
        4 => attempt(|| r.unset_color()),
        5 => r.set_bnd_box(fa, fb, fc, fa + 20.0, fb + 20.0, fc + 20.0),
        6 => r.unset_bnd_box(),
        7 => r.set_value(fa / 10.0),
        8 => r.set_automatic_position(a != 0),
        9 => r.set_ext_shape(a),
        10 => r.set_text(&text(a)),
        _ => {}
    }
}

/// `v3_prsrel Kind Variant Count (Op A B C)...`.
pub fn prs_rel_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    let (kind, variant) = (reader.integer(), reader.integer());
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
    let Some(r) = created else {
        end();
        return;
    };
    state(&*r);
    let object: Handle<dyn AisInteractiveObject> = r.clone().into_cast();
    for _ in 0..reader.integer() {
        let (op, a, b, c) = (
            reader.integer(),
            reader.integer(),
            reader.integer(),
            reader.integer(),
        );
        if op == 16 {
            trace.write().events.clear();
            attempt(|| {
                if ctx.is_displayed(&object) {
                    ctx.redisplay(&object, false, true)?;
                } else {
                    ctx.display(&object, false)?;
                }
                int(r.presentations().len() as i32);
                let events = trace.read();
                int(events.events.len() as i32);
                for value in &events.events {
                    real(*value);
                }
                Ok(())
            });
            trace.write().events.clear();
            attempt(|| selection(&r));
        } else {
            edit(&r, op, a, b, c);
        }
        state(&*r);
    }
    end();
}
