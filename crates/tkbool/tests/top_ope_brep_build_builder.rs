//! TopOpeBRepBuild builders: the tools of the builder (states, normals, faces,
//! tolerances), the builder, Builder1 and HBuilder on structures without intersection,
//! the shell to solid maker and the face fuser.
use rustycad_tkbool::top_ope_brep_build::{
    DataMapOfShapeState, TopOpeBRepBuildBuilder, TopOpeBRepBuildBuilder1, TopOpeBRepBuildFuseFace,
    TopOpeBRepBuildHBuilder, TopOpeBRepBuildShellToSolid, TopOpeBRepBuildTools,
};
use rustycad_tkbool::top_ope_brep_ds::{
    MapOfShapeWithState, TopOpeBRepDSBuildTool, TopOpeBRepDSHDataStructure,
};
use rustycad_tkbool::top_ope_brep_tool::TopOpeBRepToolShapeClassifier;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

/// The sub-shapes of a type.
fn subs(s: &Shape, t: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    Ok(Explorer::new(s, t, ShapeEnum::Shape)?.collect())
}

/// The box [x, x + 2] x [0, 2] x [0, 2].
fn cube(x: f64) -> Result<Shape, Failure> {
    Ok(
        BRepPrimAPIMakeBox::at_point(Point3::new(x, 0., 0.), 2., 2., 2.)?
            .shape()?
            .clone(),
    )
}

#[test]
fn tools_normal_and_tangent_of_a_box_face_edge() -> Result<(), Failure> {
    let b = cube(0.)?;
    let f = topods::face(&subs(&b, ShapeEnum::Face)?[0])?;
    let e = topods::edge(&subs(f.as_ref(), ShapeEnum::Edge)?[0])?;
    let n = TopOpeBRepBuildTools::get_normal_to_face_on_edge(&f, &e)?;
    assert!(n.magnitude() > 0.);
    let nn = TopOpeBRepBuildTools::get_normal_in_nearest_point(&f, &e)?;
    assert!(
        n.crossed(nn).magnitude() < 1e-12,
        "a plane has one normal direction"
    );
    let (ok, t) = TopOpeBRepBuildTools::get_tangent_to_edge(&e)?;
    assert!(ok && t.dot(n).abs() < 1e-12);
    let (rev, t2) = TopOpeBRepBuildTools::get_tangent_to_edge_edge(&f, &e, &e)?;
    assert!(!rev);
    assert!(t2.crossed(t).magnitude() < 1e-12);
    assert!(TopOpeBRepBuildTools::is_deg_edges_the_same(
        e.as_ref(),
        e.as_ref()
    )?);
    assert!(TopOpeBRepBuildTools::check_face_closed2d(&f)?);
    Ok(())
}

#[test]
fn tools_normalize_face_drops_internal_edges() -> Result<(), Failure> {
    let b = cube(0.)?;
    let f = subs(&b, ShapeEnum::Face)?[0].clone();
    let nf = TopOpeBRepBuildTools::normalize_face(&f)?;
    assert_eq!(subs(&nf, ShapeEnum::Edge)?.len(), 4);
    assert_eq!(nf.orientation(), f.orientation());
    assert_eq!(subs(&nf, ShapeEnum::Wire)?.len(), 1);
    Ok(())
}

#[test]
fn tools_correct_tolerances_keep_an_exact_box() -> Result<(), Failure> {
    let b = cube(0.)?;
    TopOpeBRepBuildTools::correct_tolerances(&b, 1.0)?;
    for e in subs(&b, ShapeEnum::Edge)? {
        assert!(BRepTool::edge_tolerance(&topods::edge(&e)?)? <= 1e-7);
    }
    Ok(())
}

#[test]
fn tools_state_through_vertex_and_propagation() -> Result<(), Failure> {
    let (a, b) = (cube(0.)?, cube(10.)?);
    let shell = subs(&a, ShapeEnum::Shell)?[0].clone();
    let mut sc = TopOpeBRepToolShapeClassifier::new();
    sc.set_reference(&b)?;
    let mut m = MapOfShapeWithState::default();
    let st = TopOpeBRepBuildTools::find_state_through_vertex(
        &shell,
        &mut sc,
        &mut m,
        &MapOfShape::default(),
    )?;
    assert_eq!(st, State::Out);
    assert_eq!(
        m.extent(),
        1 + 6 + 6 + 12,
        "the shell, its faces, wires and edges"
    );
    let faces: Vec<Shape> = subs(&shell, ShapeEnum::Face)?;
    let mut spl = DataMapOfShapeState::default();
    spl.bind(faces[0].clone(), State::In);
    let mut rest = IndexedMapOfShape::default();
    for f in &faces[1..] {
        rest.add(f.clone());
    }
    let mut m2 = MapOfShapeWithState::default();
    TopOpeBRepBuildTools::propagate_state(
        &spl,
        &rest,
        ShapeEnum::Edge,
        ShapeEnum::Face,
        &mut sc,
        &mut m2,
        &MapOfShape::default(),
    )?;
    for f in &faces[1..] {
        assert_eq!(
            m2.find_from_key(f)?.state(),
            State::In,
            "spread through the shared edges"
        );
    }
    TopOpeBRepBuildTools::propagate_state_for_wires(&rest, &mut m2)?;
    let w = subs(&faces[1], ShapeEnum::Wire)?[0].clone();
    assert_eq!(m2.find_from_key(&w)?.state(), State::In);
    Ok(())
}

#[test]
fn builder_without_intersection_finds_no_particular_case() -> Result<(), Failure> {
    let (a, b) = (cube(0.)?, cube(10.)?);
    let hds = Handle::new(TopOpeBRepDSHDataStructure::new());
    let mut builder = TopOpeBRepBuildBuilder::new(&TopOpeBRepDSBuildTool::new());
    builder.perform_shapes(&hds, &a, &b)?;
    assert_eq!(builder.is_k_part(), 0);
    assert!(builder.section()?.is_empty());
    assert!(!builder.is_split(&a, State::In));
    Ok(())
}

#[test]
fn builder1_reaches_its_members_through_the_builder() -> Result<(), Failure> {
    let (a, b) = (cube(0.)?, cube(10.)?);
    let hds = Handle::new(TopOpeBRepDSHDataStructure::new());
    let mut b1 = TopOpeBRepBuildBuilder1::new(&TopOpeBRepDSBuildTool::new());
    b1.perform_shapes(&hds, &a, &b)?;
    let mut solid = subs(&a, ShapeEnum::Solid)?[0].clone();
    assert_eq!(
        b1.correct_result2d(&mut solid)?,
        0,
        "no edge corrected in 2d"
    );
    let plain = TopOpeBRepBuildBuilder::new(&TopOpeBRepDSBuildTool::new());
    let mut s = a.clone();
    let err = plain.orientate_edge_on_face(
        &mut s,
        &a,
        &a,
        &rustycad_tkbool::top_ope_brep_build::TopOpeBRepBuildGTool::g_fus_same(
            ShapeEnum::Face,
            ShapeEnum::Face,
        ),
    );
    assert!(err.is_err(), "a Builder1 member on a plain builder");
    Ok(())
}

#[test]
fn hbuilder_section_iteration_on_an_empty_section() -> Result<(), Failure> {
    let (a, b) = (cube(0.)?, cube(10.)?);
    let hds = Handle::new(TopOpeBRepDSHDataStructure::new());
    let hb = TopOpeBRepBuildHBuilder::new(&TopOpeBRepDSBuildTool::new());
    hb.perform_shapes(&hds, &a, &b)?;
    assert_eq!(hb.is_k_part(), 0);
    hb.init_section(3)?;
    assert!(!hb.more_section());
    assert_eq!(
        hb.current_section()
            .map_err(|e| e.exception_type().to_string()),
        Err("Standard_ProgramError".to_string())
    );
    let e = subs(&a, ShapeEnum::Edge)?[0].clone();
    assert_eq!(hb.get_ds_edge_from_sect_edge(&e, 1)?, 0);
    assert_eq!(hb.get_ds_curve_from_sect_edge(&e)?, 0);
    assert!(hb.edge_curve_ancestors(&e)?.is_none());
    let (lists, found) = hb.edge_section_ancestors(&e)?.expect("an edge");
    assert!(!found && lists.iter().all(List::is_empty));
    hb.merge_k_part(State::Out, State::Out)?;
    hb.clear()?;
    Ok(())
}

#[test]
fn shell_to_solid_makes_the_solid_of_a_box_shell() -> Result<(), Failure> {
    let a = cube(0.)?;
    let shell = topods::shell(&subs(&a, ShapeEnum::Shell)?[0])?;
    let mut sts = TopOpeBRepBuildShellToSolid::new();
    sts.add_shell(&shell);
    let mut lso = List::default();
    let so = topods::solid(&subs(&a, ShapeEnum::Solid)?[0])?;
    sts.make_solids(&so, &mut lso)?;
    assert_eq!(lso.len(), 1);
    assert_eq!(
        subs(lso.first().expect("one solid"), ShapeEnum::Face)?.len(),
        6
    );
    Ok(())
}

#[test]
fn fuse_face_keeps_a_single_face() -> Result<(), Failure> {
    let a = cube(0.)?;
    let mut l = List::default();
    l.append(subs(&a, ShapeEnum::Face)?[0].clone());
    let mut ff = TopOpeBRepBuildFuseFace::with_faces(&l, &l, 0);
    ff.perform_face()?;
    assert!(!ff.is_modified());
    assert_eq!(ff.l_fuse_face().len(), 1);
    let mut vs = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(&a, ShapeEnum::Vertex, &mut vs)?;
    assert_eq!(vs.extent(), 8);
    let _ = Orientation::Forward;
    Ok(())
}
