//! TopOpeBRepTool_TOOL, TOPOLOGY, connexity, CLASSI, makeTransition, the regularization
//! of wires, PurgeInternalEdges and FuseEdges on primitive solids, against closed forms.
use rustycad_tkbo::brep_algo_api::BRepAlgoAPIFuse;
use rustycad_tkbool::top_ope_brep_tool::*;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::{Direction3, Plane3, Point2, Point3, Transform3, Vector2};
use rustycad_tkmath::toploc::Location;
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon,
    BRepBuilderAPIMakeShape, BRepBuilderAPIMakeVertex,
};
use std::f64::consts::PI;

/// The box [0, a] x [0, b] x [0, c].
fn boxed(a: f64, b: f64, c: f64) -> Result<Shape, Failure> {
    Ok(BRepPrimAPIMakeBox::from_sizes(a, b, c)?.shape()?.clone())
}

/// The cylinder of radius `r` and height `h` on the Z axis.
fn cylinder(r: f64, h: f64) -> Result<Shape, Failure> {
    Ok(BRepPrimAPIMakeCylinder::new(r, h).shape()?.clone())
}

/// The edges of a solid and their faces.
fn edge_faces(s: &Shape) -> Result<IndexedDataMapOfShapeListOfShape, Failure> {
    let mut m = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(s, ShapeEnum::Edge, ShapeEnum::Face, &mut m)?;
    Ok(m)
}

/// The first face of the solid on a surface of the U-periodicity asked.
fn face_where(s: &Shape, periodic: bool) -> Result<Face, Failure> {
    for f in Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)? {
        let f = topods::face(&f)?;
        if BRepTool::surface(&f)?
            .get()
            .is_some_and(|su| su.is_u_periodic())
            == periodic
        {
            return Ok(f);
        }
    }
    Err(Failure::new("no such face"))
}

#[test]
fn oriin_sor_and_closures_follow_the_seam_of_a_cylinder() -> Result<(), Failure> {
    let cyl = cylinder(1., 2.)?;
    let lateral = face_where(&cyl, true)?;
    assert!(TopOpeBRepToolTOOL::closed_s(&lateral)?);
    let mut seams = 0;
    for e in Explorer::new(&lateral, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let e = topods::edge(&e)?;
        if TopOpeBRepToolTOOL::is_closing_e(&e, &lateral)? {
            seams += 1;
            assert_eq!(
                TopOpeBRepToolTOOL::oriin_sor(&e, &lateral, true)?,
                TOOL_CLOSING
            );
        }
    }
    // The seam occurs twice in the face.
    assert_eq!(seams, 2);
    let plane = face_where(&cyl, false)?;
    assert!(!TopOpeBRepToolTOOL::closed_s(&plane)?);
    Ok(())
}

#[test]
fn vertices_and_boundaries_of_a_box_edge() -> Result<(), Failure> {
    let m = edge_faces(&boxed(2., 3., 4.)?)?;
    let e = topods::edge(m.find_key(1)?)?;
    let (f, l) = BRepTool::range(&e)?;
    let [v1, v2] = TopOpeBRepToolTOOL::vertices(&e)?;
    let (p1, p2) = (BRepTool::parameter(&v1, &e)?, BRepTool::parameter(&v2, &e)?);
    // A FORWARD edge starts at its smaller parameter; REVERSED swaps them.
    if e.orientation() == Orientation::Forward {
        assert!(p1 < p2);
    } else {
        assert!(p1 > p2);
    }
    assert_eq!(TopOpeBRepToolTOOL::on_boundary(f, &e)?, TOOL_FORWARD);
    assert_eq!(TopOpeBRepToolTOOL::on_boundary(l, &e)?, TOOL_REVERSED);
    assert_eq!(
        TopOpeBRepToolTOOL::on_boundary(0.5 * (f + l), &e)?,
        TOOL_INTERNAL
    );
    assert_eq!(TopOpeBRepToolTOOL::on_boundary(l + 1., &e)?, TOOL_EXTERNAL);
    Ok(())
}

#[test]
fn matter_angles_of_vectors_are_measured_to_the_reversed_second() -> Result<(), Failure> {
    let a = TopOpeBRepToolTOOL::matter_2d(Vector2::new(1., 0.), Vector2::new(0., 1.))?;
    assert!((a - 1.5 * PI).abs() < 1e-12, "{a}");
    assert_eq!(
        TopOpeBRepToolTOOL::matter_2d(Vector2::new(1., 0.), Vector2::new(-1., 0.))?,
        0.
    );
    assert_eq!(
        TopOpeBRepToolTOOL::matter_2d(Vector2::new(1., 0.), Vector2::new(1., 0.))?,
        PI
    );
    Ok(())
}

#[test]
fn faces_of_a_convex_box_edge_hold_a_right_matter_angle() -> Result<(), Failure> {
    let m = edge_faces(&boxed(2., 2., 2.)?)?;
    for i in 1..=m.extent() {
        let e = topods::edge(m.find_key(i)?)?;
        let faces = m.find_from_index(i)?;
        let f1 = topods::face(faces.first().unwrap())?;
        let f2 = topods::face(faces.last().unwrap())?;
        let (a, b) = BRepTool::range(&e)?;
        let ang = TopOpeBRepToolTOOL::matter_faces(&f1, &f2, &e, 0.5 * (a + b), 1e-9)?;
        // The matter between two faces of a convex right-angled edge spans PI / 2.
        assert!(
            (ang.expect("opposite faces") - 0.5 * PI).abs() < 1e-9,
            "{ang:?}"
        );
    }
    Ok(())
}

#[test]
fn curvatures_of_a_cylinder_and_its_circles() -> Result<(), Failure> {
    let r = 0.5;
    let cyl = cylinder(r, 2.)?;
    let lateral = face_where(&cyl, true)?;
    for e in Explorer::new(&lateral, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let e = topods::edge(&e)?;
        if TopOpeBRepToolTOOL::is_closing_e(&e, &lateral)? {
            continue;
        }
        let (ok, curv) = TopOpeBRepToolTOOL::curv_e(&e, 0.3, Direction3::Z)?;
        assert!(ok && (curv - 1. / r).abs() < 1e-9, "{ok} {curv}");
    }
    // Seen along the axis, the contour of the cylinder has the curvature 1 / r.
    let (ok, curv, _) =
        TopOpeBRepToolTOOL::curv_f(&lateral, Point2::new(0.3, 1.), Direction3::Z, false)?;
    assert!(ok && (curv - 1. / r).abs() < 1e-9, "{ok} {curv}");
    Ok(())
}

#[test]
fn a_point_is_in_or_out_of_a_box_face_by_its_normal() -> Result<(), Failure> {
    let b = boxed(2., 2., 2.)?;
    let mut top = None;
    for f in Explorer::new(&b, ShapeEnum::Face, ShapeEnum::Shape)? {
        let f = topods::face(&f)?;
        let mut n = Direction3::Z;
        let mut uv = Point2::default();
        fun_tool_paron_ef(
            &topods::edge(Explorer::new(&f, ShapeEnum::Edge, ShapeEnum::Shape)?.current())?,
            0.,
            &f,
            &mut uv,
        )?;
        nt(uv, &f, &mut n)?;
        if n.z() > 0.5 {
            top = Some(f);
        }
    }
    let top = top.expect("top face");
    let mut uv = Point2::default();
    let (ok, st) = TopOpeBRepToolTOOL::getstp3d_f(Point3::new(1., 1., 3.), &top, &mut uv)?;
    assert!(ok);
    assert_eq!(st, State::Out);
    let (_, st) = TopOpeBRepToolTOOL::getstp3d_f(Point3::new(1., 1., 1.), &top, &mut uv)?;
    assert_eq!(st, State::In);
    let (_, st) = TopOpeBRepToolTOOL::getstp3d_f(Point3::new(1., 1., 2.), &top, &mut uv)?;
    assert_eq!(st, State::On);
    Ok(())
}

#[test]
fn a_box_face_lies_in_its_box() -> Result<(), Failure> {
    let b = boxed(2., 2., 2.)?;
    let f = face_where(&b, false)?;
    // The box of a face is inside the box of the solid (flat in one direction).
    assert_eq!(fun_tool_comparebndkole(&f, &b)?, 2);
    assert_eq!(fun_tool_comparebndkole(&b, &b)?, 0);
    assert_eq!(fun_tool_nbshapes(&b, ShapeEnum::Edge)?, 24);
    Ok(())
}

#[test]
fn a_connexity_sorts_its_items_by_orientation() -> Result<(), Failure> {
    let m = edge_faces(&boxed(1., 1., 1.)?)?;
    let e1 = m.find_key(1)?.clone();
    let e2 = m.find_key(2)?.clone();
    let mut c = TopOpeBRepToolConnexity::with_key(&e1);
    c.add_item(TOOL_FORWARD, &e1)?;
    assert!(c.is_faulty());
    c.add_item(TOOL_REVERSED, &e2)?;
    assert!(!c.is_faulty() && !c.is_multiple());
    c.add_item(TOOL_CLOSING, &e2)?;
    assert!(c.is_multiple());
    assert_eq!(c.all_items().0, 2);
    assert!(c.remove_item(&e2));
    assert_eq!(c.item(TOOL_CLOSING)?.0, 0);
    assert!(c.item(6).is_err());
    Ok(())
}

/// The square wire [x0, x1] x [y0, y1] on the plane z = 0.
fn square(x0: f64, y0: f64, x1: f64, y1: f64) -> Result<Shape, Failure> {
    let mut poly = BRepBuilderAPIMakePolygon::new();
    for (x, y) in [(x0, y0), (x1, y0), (x1, y1), (x0, y1)] {
        poly.add_point(Point3::new(x, y, 0.))?;
    }
    poly.close()?;
    Ok(poly.wire()?.into_shape())
}

#[test]
fn classi_nests_squares() -> Result<(), Failure> {
    let (outer, hole, away) = (
        square(0., 0., 10., 10.)?,
        square(2., 2., 4., 4.)?,
        square(12., 0., 14., 2.)?,
    );
    let mut c = TopOpeBRepToolCLASSI::new();
    c.init2d(&BRepBuilderAPIMakeFace::from_plane(Plane3::default())?.face()?);
    assert_eq!(
        c.classi_bnd2d(&hole, &outer, 1e-7, false)?,
        CLASSI_ONE_IN_TWO
    );
    assert_eq!(c.classi_bnd2d(&outer, &away, 1e-7, false)?, CLASSI_DIFF);
    assert_eq!(c.classi_bnd2d(&outer, &outer, 1e-7, false)?, CLASSI_SAME);
    let mut l: List<Shape> = List::default();
    for w in [&outer, &hole, &away] {
        l.append(w.clone());
    }
    let mut m = ShapeListMap::default();
    assert!(c.classilist(&l, &mut m)?);
    assert_eq!(m.seek(&outer).map(|l| l.extent()), Some(1));
    assert!(m.seek(&hole).is_none());
    assert_eq!(m.seek(&away).map(|l| l.extent()), Some(0));
    Ok(())
}

/// The face on the plane z = 0 of the closed walk through the vertices of the points.
fn walk_face(points: &[(f64, f64)], ids: &[usize]) -> Result<Face, Failure> {
    let v: Vec<Vertex> = points
        .iter()
        .map(|&(x, y)| BRepBuilderAPIMakeVertex::new(Point3::new(x, y, 0.))?.vertex())
        .collect::<Result<_, _>>()?;
    let bb = BRepBuilder::default();
    let mut w = Wire::default();
    bb.make_wire(&mut w);
    for i in 0..ids.len() {
        let e = BRepBuilderAPIMakeEdge::from_vertices(&v[ids[i]], &v[ids[(i + 1) % ids.len()]])?
            .edge()?;
        bb.add(&mut w, &e)?;
    }
    BRepBuilderAPIMakeFace::from_plane_wire(Plane3::default(), &w, true)?.face()
}

#[test]
fn a_figure_eight_face_is_regularized_into_two_faces() -> Result<(), Failure> {
    let pts = [(0., 0.), (2., -1.), (2., 1.), (-2., 1.), (-2., -1.)];
    let face = walk_face(&pts, &[0, 1, 2, 0, 3, 4])?;
    let (mut lof, mut es) = (List::default(), ShapeListMap::default());
    assert!(regularize(&face, &mut lof, &mut es)?);
    assert_eq!(lof.extent(), 2);
    for f in lof.iter() {
        // Each lobe is a triangle.
        assert_eq!(fun_tool_nbshapes(f, ShapeEnum::Edge)?, 3);
    }
    // A simple square is left alone.
    let sq = walk_face(&[(0., 0.), (1., 0.), (1., 1.), (0., 1.)], &[0, 1, 2, 3])?;
    let (mut ow, mut es) = (ShapeListMap::default(), ShapeListMap::default());
    assert!(regularize_wires(&sq, &mut ow, &mut es)?);
    let w = Explorer::new(&sq, ShapeEnum::Wire, ShapeEnum::Shape)?
        .current()
        .clone();
    assert_eq!(ow.seek(&w).map(|l| l.extent()), Some(0));
    Ok(())
}

#[test]
fn an_internal_diagonal_is_purged() -> Result<(), Failure> {
    let face = walk_face(&[(0., 0.), (1., 0.), (1., 1.), (0., 1.)], &[0, 1, 2, 3])?;
    let w = Explorer::new(&face, ShapeEnum::Wire, ShapeEnum::Shape)?
        .current()
        .clone();
    let vs: Vec<Vertex> = Explorer::new(&w, ShapeEnum::Vertex, ShapeEnum::Shape)?
        .map(|v| topods::vertex(&v))
        .collect::<Result<_, _>>()?;
    let at = |p: Point3| {
        vs.iter()
            .find(|v| BRepTool::pnt(v).is_ok_and(|q| q.distance(&p) < 1e-9))
            .cloned()
    };
    let d = BRepBuilderAPIMakeEdge::from_vertices(
        &at(Point3::new(0., 0., 0.)).unwrap(),
        &at(Point3::new(1., 1., 0.)).unwrap(),
    )?
    .edge()?;
    let mut w = w;
    if let Some(t) = w.tshape().get() {
        t.set_free(true);
    }
    BRepBuilder::default().add(&mut w, &d.oriented(Orientation::Internal))?;
    assert_eq!(fun_tool_nbshapes(&face, ShapeEnum::Edge)?, 5);
    let mut p = TopOpeBRepToolPurgeInternalEdges::new(&face, true)?;
    assert_eq!(p.nb_edges()?, 1);
    assert_eq!(fun_tool_nbshapes(p.shape()?, ShapeEnum::Edge)?, 4);
    Ok(())
}

/// A box translated by `(x, y, z)`.
fn moved_box(a: f64, b: f64, c: f64, (x, y, z): (f64, f64, f64)) -> Result<Shape, Failure> {
    let mut tr = Transform3::default();
    tr.set_translation(rustycad_tkmath::gp::Vector3::new(x, y, z));
    boxed(a, b, c)?.moved(&Location::new(tr), false)
}

#[test]
fn collinear_edges_of_a_face_are_fused() -> Result<(), Failure> {
    // (0, 0)-(1, 0) and (1, 0)-(2, 0) are on one line, their vertex has no other edge.
    let face = walk_face(
        &[(0., 0.), (1., 0.), (2., 0.), (2., 1.), (0., 1.)],
        &[0, 1, 2, 3, 4],
    )?;
    let mut fe = TopOpeBRepToolFuseEdges::new(&face, false)?;
    assert_eq!(fe.edges()?.extent(), 1);
    assert_eq!(fe.nb_vertices()?, 1);
    assert_eq!(fun_tool_nbshapes(fe.shape()?, ShapeEnum::Edge)?, 4);
    // Two cubes fused keep the edge of their shared face at the middle vertices.
    let fused =
        BRepAlgoAPIFuse::from_shapes(&boxed(1., 1., 1.)?, &moved_box(1., 1., 1., (1., 0., 0.))?)?
            .shape()
            .clone();
    assert_eq!(
        TopOpeBRepToolFuseEdges::new(&fused, false)?.nb_vertices()?,
        0
    );
    Ok(())
}

#[test]
fn an_edge_crossing_the_top_of_a_box_goes_from_in_to_out() -> Result<(), Failure> {
    let b = boxed(2., 2., 2.)?;
    let e: Edge =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(1., 1., 1.), Point3::new(1., 1., 3.))?
            .edge()?;
    let mut top = None;
    for f in Explorer::new(&b, ShapeEnum::Face, ShapeEnum::Shape)? {
        let f = topods::face(&f)?;
        let mut uv = Point2::default();
        if TopOpeBRepToolTOOL::getstp3d_f(Point3::new(1., 1., 2.), &f, &mut uv)?.1 == State::On
            && fun_tool_value_on_face(uv, &f)?.z() > 1.9
        {
            top = Some((f, uv));
        }
    }
    let (top, uv) = top.expect("top face");
    let (a, l) = BRepTool::range(&e)?;
    let mut mt = TopOpeBRepToolMakeTransition::new();
    assert!(mt.initialize(&e, (a, l, 1.), &top, uv, 0.2)?);
    assert!(!mt.is_t2d());
    let (ok, before, after) = mt.mk_ton_e()?;
    assert!(ok);
    assert_eq!((before, after), (State::In, State::Out));
    Ok(())
}
