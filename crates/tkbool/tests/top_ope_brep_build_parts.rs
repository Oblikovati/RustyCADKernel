//! TopOpeBRepBuild parts: the paves and the edge builder, the shell face set and the
//! solid builder, the operation tables, the path of a wire and the 2d correction of
//! a face.
use rustycad_tkbool::top_ope_brep_build::{
    MapOfShapeVertexInfo, MapOfTrans2dInfo, TopOpeBRepBuildCorrectFace2d, TopOpeBRepBuildEdgeBuilder,
    TopOpeBRepBuildGIter, TopOpeBRepBuildGTool, TopOpeBRepBuildPave, TopOpeBRepBuildPaveClassifier,
    TopOpeBRepBuildPaveSet, TopOpeBRepBuildShapeSet, TopOpeBRepBuildShellFaceSet,
    TopOpeBRepBuildSolidBuilder, TopOpeBRepBuildTools2d,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape, Wire};
use rustycad_tkernel::ncollection::IndexedMap;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_lib::{BRepLibMakeEdge, BRepLibMakePolygon, BRepLibMakeVertex};

/// The sub-shapes of a type.
fn subs(s: &Shape, t: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    Ok(Explorer::new(s, t, ShapeEnum::Shape)?.collect())
}

/// The closed square wire [0, 1]^2 of the plane z = 0.
fn unit_square() -> Result<Wire, Failure> {
    let mut mp = BRepLibMakePolygon::new();
    for (x, y) in [(0., 0.), (1., 0.), (1., 1.), (0., 1.)] {
        mp.add_point(Point3::new(x, y, 0.))?;
    }
    mp.close()?;
    mp.wire()
}

/// The pave of a vertex oriented `o` at the parameter `p`.
fn pave(v: &Shape, o: Orientation, p: f64) -> Handle<TopOpeBRepBuildPave> {
    Handle::new(TopOpeBRepBuildPave::new(&v.oriented(o), p, false))
}

#[test]
fn edge_builder_splits_an_edge_at_an_inner_pave() -> Result<(), Failure> {
    let e = BRepLibMakeEdge::from_points(Point3::new(0., 0., 0.), Point3::new(10., 0., 0.))?.edge()?;
    let (v1, v2) = topexp::vertices(&e, false);
    let vm = BRepLibMakeVertex::new(Point3::new(4., 0., 0.))?.vertex()?.into_shape();
    let mut ps = TopOpeBRepBuildPaveSet::new(e.as_ref())?;
    ps.append(&pave(v1.as_ref(), Orientation::Forward, 0.));
    ps.append(&pave(&vm, Orientation::Reversed, 4.));
    ps.append(&pave(&vm, Orientation::Forward, 4.));
    ps.append(&pave(v2.as_ref(), Orientation::Reversed, 10.));
    let mut pc = TopOpeBRepBuildPaveClassifier::new(e.as_ref())?;
    let mut eb = TopOpeBRepBuildEdgeBuilder::with_paves(&mut ps, &mut pc, false)?;
    let mut ranges = Vec::new();
    eb.init_edge();
    while eb.more_edge() {
        let mut r = Vec::new();
        eb.init_vertex();
        while eb.more_vertex() {
            r.push(eb.parameter()?);
            eb.next_vertex();
        }
        ranges.push(r);
        eb.next_edge();
    }
    ranges.sort_by(|a, b| a[0].total_cmp(&b[0]));
    assert_eq!(ranges, vec![vec![0., 4.], vec![4., 10.]]);
    Ok(())
}

#[test]
fn pave_set_detects_equal_parameters() -> Result<(), Failure> {
    let e = BRepLibMakeEdge::from_points(Point3::new(0., 0., 0.), Point3::new(1., 0., 0.))?.edge()?;
    let (v1, _) = topexp::vertices(&e, false);
    let mut ps = TopOpeBRepBuildPaveSet::new(e.as_ref())?;
    ps.append(&pave(v1.as_ref(), Orientation::Forward, 0.5));
    ps.append(&pave(v1.as_ref(), Orientation::Reversed, 0.5));
    assert!(ps.has_equal_parameters()?);
    assert_eq!(ps.equal_parameters(), 0.5);
    Ok(())
}

#[test]
fn solid_builder_rebuilds_the_shell_of_a_box() -> Result<(), Failure> {
    let solid = BRepPrimAPIMakeBox::from_sizes(1., 2., 3.)?.shape()?.clone();
    let mut sfs = TopOpeBRepBuildShellFaceSet::with_solid(&solid)?;
    for f in subs(&solid, ShapeEnum::Face)? {
        sfs.add_start_element(&f)?;
    }
    let mut sb = TopOpeBRepBuildSolidBuilder::with_shell_face_set(&mut sfs, false)?;
    let (mut solids, mut faces) = (0, 0);
    sb.init_solid();
    while sb.more_solid() {
        solids += 1;
        sb.init_shell();
        while sb.more_shell() {
            sb.init_face()?;
            while sb.more_face() {
                assert_eq!(sb.face()?.shape_type()?, ShapeEnum::Face);
                faces += 1;
                sb.next_face();
            }
            sb.next_shell();
        }
        sb.next_solid();
    }
    assert_eq!((solids, faces), (1, 6));
    Ok(())
}

#[test]
fn gtool_tables_keep_the_parts_of_each_operation() -> Result<(), Failure> {
    let fus = TopOpeBRepBuildGTool::g_fus_unsh(ShapeEnum::Solid, ShapeEnum::Solid);
    assert!(fus.value(State::On, State::Out)? && fus.value(State::Out, State::On)?);
    assert!(!fus.value(State::In, State::Out)?);
    let cut = TopOpeBRepBuildGTool::g_cut_unsh(ShapeEnum::Solid, ShapeEnum::Solid);
    assert!(cut.value(State::In, State::On)? && cut.value(State::On, State::Out)?);
    let com = TopOpeBRepBuildGTool::g_com_unsh(ShapeEnum::Solid, ShapeEnum::Solid);
    let permuted = com.copy_permuted();
    assert_eq!(permuted.value(State::In, State::On)?, com.value(State::On, State::In)?);
    let mut it = TopOpeBRepBuildGIter::with_gtopo(&fus);
    let mut kept = Vec::new();
    while it.more() {
        kept.push(it.current()?);
        it.next();
    }
    assert_eq!(kept, vec![(State::On, State::Out), (State::Out, State::On)]);
    Ok(())
}

#[test]
fn tools2d_path_walks_every_edge_of_a_wire() -> Result<(), Failure> {
    let w = unit_square()?;
    let mut m = MapOfShapeVertexInfo::default();
    TopOpeBRepBuildTools2d::make_map_of_shape_vertex_info(&w, &mut m)?;
    assert_eq!(m.extent(), 4);
    assert!((1..=4).all(|i| m.find_from_index(i).is_ok_and(|vi| !vi.smart())));
    let path = TopOpeBRepBuildTools2d::path(&w)?;
    assert_eq!(path.extent(), 4);
    let first = topods::vertex(m.find_key(1)?)?;
    let last = topods::edge(path.first().ok_or_else(|| Failure::new("empty path"))?)?;
    let end = if last.orientation() == Orientation::Forward {
        topexp::last_vertex(&last, false)
    } else {
        topexp::first_vertex(&last, false)
    };
    assert!(end.is_same(first.as_ref()));
    Ok(())
}

#[test]
fn correct_face2d_skips_a_plane_and_keeps_a_connected_cylinder() -> Result<(), Failure> {
    let mut map = MapOfTrans2dInfo::default();
    let plane_face = topods::face(&subs(&BRepPrimAPIMakeBox::from_sizes(1., 1., 1.)?.shape()?.clone(), ShapeEnum::Face)?[0])?;
    let mut cf = TopOpeBRepBuildCorrectFace2d::with_face(&plane_face, &IndexedMap::default(), &mut map);
    cf.perform()?;
    assert!(cf.is_done());
    assert_eq!(cf.error_status(), 4);
    let cyl = BRepPrimAPIMakeCylinder::new(1., 2.).shape()?.clone();
    let lateral = subs(&cyl, ShapeEnum::Face)?
        .into_iter()
        .find(|f| topods::face(f).is_ok_and(|f| BRepTool::surface(&f).is_ok_and(|s| s.get().is_some_and(|s| s.is_u_periodic()))))
        .ok_or_else(|| Failure::new("no lateral face"))?;
    let lateral = topods::face(&lateral)?;
    let mut map = MapOfTrans2dInfo::default();
    let mut cf = TopOpeBRepBuildCorrectFace2d::with_face(&lateral, &IndexedMap::default(), &mut map);
    cf.perform()?;
    assert!(cf.is_done());
    assert_eq!(cf.error_status(), 0);
    assert_eq!(subs(cf.corrected_face().as_ref(), ShapeEnum::Edge)?.len(), 4);
    Ok(())
}
