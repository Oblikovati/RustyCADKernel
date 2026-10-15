//! BRepOffset_Tool probe operations, the Rust twin of
//! tools/oracle/tkoffset_probe_brepoffset_tool.cxx (the records are documented there).
use super::common::{Reader, attempt, end, flag, int, pnt, real};
use super::shape::shape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Shell};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkgeombase::gc::MakeArcOfCircle;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3, Vector3};
use rustycad_tkoffset::brep_offset::{BRepOffsetTool, EnlargeFaceOptions};
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCone, BRepPrimAPIMakeCylinder, BRepPrimAPIMakePrism,
    BRepPrimAPIMakeSphere, BRepPrimAPIMakeTorus,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_lib::{
    BRepLibMakeEdge, BRepLibMakeFace, BRepLibMakePolygon, BRepLibMakeWire,
};

/// The primitive of a request (`ot_primitive`).
pub(super) fn primitive(r: &mut Reader) -> Result<Shape, Failure> {
    let kind = r.integer();
    let (a, b, c) = (r.real(), r.real(), r.real());
    let (o, n) = (r.pnt(), r.dir());
    let ax = Axis2::new(o, n)?;
    let s = match kind {
        0 => BRepPrimAPIMakeBox::with_axes(ax, a, b, c).shape()?.clone(),
        1 => BRepPrimAPIMakeCylinder::with_axes(ax, a, b)
            .shape()?
            .clone(),
        2 => BRepPrimAPIMakeCone::with_axes(ax, a, b, c)?
            .shape()?
            .clone(),
        3 => BRepPrimAPIMakeSphere::new(a)?.shape()?.clone(),
        5 => BRepPrimAPIMakeTorus::with_axes_angle(ax, a, b, c)?
            .shape()?
            .clone(),
        6 => slot(ax, o, n, (a, b, c))?,
        7 | 8 => polygon_prism(ax, o, n, kind == 7, (a, b, c))?,
        _ => BRepPrimAPIMakeTorus::with_axes(ax, a, b)?.shape()?.clone(),
    };
    Ok(s)
}

/// The prism of height `c` of the slot of half length `a` and radius `b` (kind 6 of
/// `ot_primitive`).
fn slot(ax: Axis2, o: Point3, n: Direction3, (a, b, c): (f64, f64, f64)) -> Result<Shape, Failure> {
    let (x, y) = (
        Vector3::from(ax.x_direction()),
        Vector3::from(ax.y_direction()),
    );
    let at = |u: f64, v: f64| o.translated(x * u + y * v);
    let (p1, p2, p3, p4) = (at(-a, -b), at(a, -b), at(a, b), at(-a, b));
    let arc = |s: Point3, m: Point3, e: Point3| -> Result<Edge, Failure> {
        let curve: Handle<dyn GeomCurve> =
            MakeArcOfCircle::through_points(s, m, e)?.value()?.cast();
        BRepLibMakeEdge::from_curve(&curve)?.edge()
    };
    let mut w = BRepLibMakeWire::new();
    w.add_edge(&BRepLibMakeEdge::from_points(p1, p2)?.edge()?)?;
    w.add_edge(&arc(p2, at(a + b, 0.), p3)?)?;
    w.add_edge(&BRepLibMakeEdge::from_points(p3, p4)?.edge()?)?;
    w.add_edge(&arc(p4, at(-a - b, 0.), p1)?)?;
    let face = BRepLibMakeFace::from_wire(&w.wire()?, true)?.face()?;
    Ok(
        BRepPrimAPIMakePrism::new(&face, Vector3::from(n) * c, false, true)?
            .shape()?
            .clone(),
    )
}

/// The prism of height `c` of the L (kind 7) or U (kind 8) polygon of width `a` and depth
/// `b` of `ot_primitive`.
fn polygon_prism(
    ax: Axis2,
    o: Point3,
    n: Direction3,
    l: bool,
    (a, b, c): (f64, f64, f64),
) -> Result<Shape, Failure> {
    let (x, y) = (
        Vector3::from(ax.x_direction()),
        Vector3::from(ax.y_direction()),
    );
    let at = |u: f64, v: f64| o.translated(x * (u * a) + y * (v * b));
    let l7: &[(f64, f64)] = &[
        (0., 0.),
        (1., 0.),
        (1., 0.4),
        (0.4, 0.4),
        (0.4, 1.),
        (0., 1.),
    ];
    let l8: &[(f64, f64)] = &[
        (0., 0.),
        (1., 0.),
        (1., 1.),
        (0.6, 1.),
        (0.6, 0.3),
        (0.4, 0.3),
        (0.4, 1.),
        (0., 1.),
    ];
    let mut poly = BRepLibMakePolygon::new();
    for &(u, v) in if l { l7 } else { l8 } {
        poly.add_point(at(u, v))?;
    }
    poly.close()?;
    let face = BRepLibMakeFace::from_wire(&poly.wire()?, true)?.face()?;
    Ok(
        BRepPrimAPIMakePrism::new(&face, Vector3::from(n) * c, false, true)?
            .shape()?
            .clone(),
    )
}

/// The sub-shape of a type at an explorer index (`ot_sub`), null past the end.
pub(super) fn sub(s: &Shape, kind: ShapeEnum, index: i32) -> Result<Shape, Failure> {
    let found = Explorer::new(s, kind, ShapeEnum::Shape)?.nth(index.max(0) as usize);
    Ok(if index < 0 { None } else { found }.unwrap_or_default())
}

/// The face of a request on a primitive with the original face (`ot_face`).
fn face(r: &mut Reader, prim: &Shape) -> Result<(Face, Shape), Failure> {
    let index = r.integer();
    let orig = sub(prim, ShapeEnum::Face, index)?;
    let f = topods::face(&orig)?;
    if r.flag() {
        let mut nf = Face::default();
        BRepOffsetTool::en_large_face(&f, &mut nf, EnlargeFaceOptions::with_change_geom(true))?;
        return Ok((nf, orig));
    }
    Ok((f, orig))
}

/// The sections of a list pair (`ot_sections`).
fn sections(l1: &List<Shape>, l2: &List<Shape>) -> Result<(), Failure> {
    int(l1.len() as i32);
    int(l2.len() as i32);
    for (s1, s2) in l1.iter().zip(l2.iter()) {
        int(s1.orientation() as i32);
        int(s2.orientation() as i32);
        shape(s1)?;
    }
    Ok(())
}

/// Prints the exception type of a failed request body, then ends the record.
pub(super) fn finish(result: Result<(), Failure>) {
    if let Err(error) = result {
        print!("{} ", error.exception_type());
    }
    end();
}

/// `ot_inter3d`.
pub fn inter3d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let op = r.integer();
    finish((|| {
        let prim1 = primitive(&mut r)?;
        let (f1, p1) = face(&mut r, &prim1)?;
        let prim2 = if r.flag() { prim1 } else { primitive(&mut r)? };
        let (f2, p2) = face(&mut r, &prim2)?;
        let side = if r.flag() { State::Out } else { State::In };
        let reference = r.flag();
        let (mut l1, mut l2) = (List::default(), List::default());
        match op {
            0 => {
                let mut e = Edge::default();
                if reference {
                    let mut le = List::default();
                    BRepOffsetTool::find_common_shapes(&p1, &p2, ShapeEnum::Edge, &mut le)?;
                    if let Some(first) = le.first() {
                        e = topods::edge(first)?;
                    }
                }
                let nf = Face::default();
                BRepOffsetTool::inter3d(&f1, &f2, &mut l1, &mut l2, side, &e, &nf, &nf)?;
            }
            1 => BRepOffsetTool::pipe_inter(&f1, &f2, &mut l1, &mut l2, side)?,
            _ => BRepOffsetTool::inter_or_extent(&f1, &f2, &mut l1, &mut l2, side)?,
        }
        sections(&l1, &l2)
    })());
}

/// `ot_enlarge`.
pub fn enlarge_case(values: &[f64]) {
    let mut r = Reader::new(values);
    finish((|| {
        let p = primitive(&mut r)?;
        let index = r.integer();
        let f = topods::face(&sub(&p, ShapeEnum::Face, index)?)?;
        let options = EnlargeFaceOptions {
            can_extent_surface: r.flag(),
            update_pcurve: r.flag(),
            enlarge_u: r.flag(),
            enlarge_vfirst: r.flag(),
            enlarge_vlast: r.flag(),
            extension_mode: r.integer(),
            lengths: [r.real(), r.real(), r.real(), r.real()],
        };
        let mut nf = Face::default();
        flag(BRepOffsetTool::en_large_face(&f, &mut nf, options)?);
        shape(nf.as_ref())?;
        shape(f.as_ref())
    })());
}

/// `ot_inter2d`.
pub fn inter2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    finish((|| {
        let p = primitive(&mut r)?;
        let fi = r.integer();
        let enlarge = r.flag();
        let mut f = topods::face(&sub(&p, ShapeEnum::Face, fi)?)?;
        let e1i = r.integer();
        let e1 = topods::edge(&sub(f.as_ref(), ShapeEnum::Edge, e1i)?)?;
        let e2i = r.integer();
        let e2 = topods::edge(&sub(f.as_ref(), ShapeEnum::Edge, e2i)?)?;
        let tol = r.real();
        if enlarge {
            let mut nf = Face::default();
            BRepOffsetTool::en_large_face(&f, &mut nf, EnlargeFaceOptions::with_change_geom(true))?;
            f = nf;
        }
        let mut lv = List::default();
        BRepOffsetTool::inter2d(&f, &e1, &e2, &mut lv, tol)?;
        int(lv.len() as i32);
        let forward1 = topods::edge(&e1.oriented(Orientation::Forward))?;
        let forward2 = topods::edge(&e2.oriented(Orientation::Forward))?;
        for v in lv.iter() {
            let vx = topods::vertex(v)?;
            pnt(BRepTool::pnt(&vx)?);
            int(v.orientation() as i32);
            attempt(|| BRepTool::parameter(&vx, &forward1).map(real));
            attempt(|| BRepTool::parameter(&vx, &forward2).map(real));
        }
        Ok(())
    })());
}

/// `ot_queries`.
pub fn queries_case(values: &[f64]) {
    let mut r = Reader::new(values);
    finish((|| {
        let p = primitive(&mut r)?;
        let i1 = r.integer();
        let f1 = topods::face(&sub(&p, ShapeEnum::Face, i1)?)?;
        let i2 = r.integer();
        let f2 = topods::face(&sub(&p, ShapeEnum::Face, i2)?)?;
        let (mut le, mut lv) = (List::default(), List::default());
        flag(BRepOffsetTool::find_common_shapes_of_faces(
            &f1, &f2, &mut le, &mut lv,
        )?);
        int(le.len() as i32);
        int(lv.len() as i32);
        flag(BRepOffsetTool::check_planes_normals(&f1, &f2, 1e-8)?);
        for e in le.iter() {
            attempt(|| {
                let (o1, o2) = BRepOffsetTool::orient_section(&topods::edge(e)?, &f1, &f2)?;
                int(o1 as i32);
                int(o2 as i32);
                Ok(())
            });
        }
        for e in Explorer::new(f1.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)? {
            let (c, _, _) = BRepTool::curve(&topods::edge(&e)?)?;
            if !c.is_null() {
                attempt(|| BRepOffsetTool::gabarit(&c).map(real));
            }
        }
        Ok(())
    })());
}

/// `ot_deboucle`.
pub fn deboucle_case(values: &[f64]) {
    let mut r = Reader::new(values);
    finish((|| {
        let p = primitive(&mut r)?;
        let removed = r.integer();
        let bound = r.flag();
        let bb = BRepBuilder::default();
        let mut sh = Shell::default();
        bb.make_shell(&mut sh);
        let mut sh = sh.into_shape();
        let mut boundary = MapOfShape::default();
        for (i, f) in Explorer::new(&p, ShapeEnum::Face, ShapeEnum::Shape)?.enumerate() {
            if i as i32 == removed {
                if bound {
                    for e in Explorer::new(&f, ShapeEnum::Edge, ShapeEnum::Shape)? {
                        boundary.add(e);
                    }
                }
                continue;
            }
            bb.add(&mut sh, &f)?;
        }
        shape(&BRepOffsetTool::deboucle_3d(&sh, &boundary)?)
    })());
}
