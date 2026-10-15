//! TopOpeBRepDS TKI, ProcessInterferencesTool, EXPORT, repvg, redu, TOOL, connex, the
//! interference tools, EIR, FIR, Filter, Check and Dumper on hand-built structures of a
//! box, against closed forms.
use rustycad_tkbool::top_ope_brep_ds::export::*;
use rustycad_tkbool::top_ope_brep_ds::process_interferences_tool::*;
use rustycad_tkbool::top_ope_brep_ds::*;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

type H = Handle<TopOpeBRepDSInterference>;

/// The box [0, 2]^3.
fn cube() -> Result<Shape, Failure> {
    Ok(BRepPrimAPIMakeBox::from_sizes(2.0, 2.0, 2.0)?
        .shape()?
        .clone())
}

/// A structure with the indices of its faces, edges and vertices.
type Built = (
    Handle<TopOpeBRepDSHDataStructure>,
    Vec<i32>,
    Vec<i32>,
    Vec<i32>,
);

/// A structure of the box (index 1), its faces, edges and vertices (rank 1).
fn structure() -> Result<Built, Failure> {
    let s = cube()?;
    let hds = Handle::new(TopOpeBRepDSHDataStructure::new());
    let (mut faces, mut edges, mut vertices) = (Vec::new(), Vec::new(), Vec::new());
    {
        let mut ds = hds.get().unwrap().change_ds();
        ds.add_shape_with_rank(&s, 1);
        for (t, v) in [
            (ShapeEnum::Face, &mut faces),
            (ShapeEnum::Edge, &mut edges),
            (ShapeEnum::Vertex, &mut vertices),
        ] {
            for x in Explorer::new(&s, t, ShapeEnum::Shape)? {
                if !ds.has_shape(&x, true) {
                    v.push(ds.add_shape_with_rank(&x, 1));
                }
            }
        }
    }
    Ok((hds, faces, edges, vertices))
}

/// A transition on faces of index `i`.
fn tr(b: State, a: State, i: i32) -> TopOpeBRepDSTransition {
    let mut t = TopOpeBRepDSTransition::new(b, a, ShapeEnum::Face, ShapeEnum::Face);
    t.set_index(i);
    t
}

/// An edge/vertex interference on the edge `s` at the vertex `g`.
fn evi(t: &TopOpeBRepDSTransition, sk: TopOpeBRepDSKind, s: i32, g: i32, p: f64) -> H {
    Handle::new(TopOpeBRepDSEdgeVertexInterference::new(
        t,
        sk,
        s,
        g,
        false,
        TopOpeBRepDSConfig::UnshGeometry,
        p,
    ))
    .cast()
}

/// A curve/point interference at the point `g`.
fn cpi(t: &TopOpeBRepDSTransition, sk: TopOpeBRepDSKind, s: i32, g: i32, p: f64) -> H {
    Handle::new(TopOpeBRepDSCurvePointInterference::new(
        t,
        sk,
        s,
        TopOpeBRepDSKind::Point,
        g,
        p,
    ))
    .cast()
}

/// A list of interferences.
fn loi(v: &[H]) -> ListOfInterference {
    v.iter().cloned().collect()
}

#[test]
fn tki_groups_by_kind_then_index() -> Result<(), Failure> {
    let t = tr(State::Out, State::In, 3);
    let l = loi(&[
        evi(&t, TopOpeBRepDSKind::Edge, 1, 9, 0.),
        cpi(&t, TopOpeBRepDSKind::Edge, 1, 2, 0.),
        evi(&t, TopOpeBRepDSKind::Face, 4, 9, 1.),
    ]);
    let mut tki = TopOpeBRepDSTKI::new();
    tki.fill_on_geometry(&l)?;
    tki.init();
    let mut seen = Vec::new();
    while let Some((k, g, x)) = tki.value() {
        seen.push((k, g, x.extent()));
        tki.next();
    }
    assert_eq!(
        seen,
        vec![
            (TopOpeBRepDSKind::Point, 2, 1),
            (TopOpeBRepDSKind::Vertex, 9, 2)
        ]
    );
    assert!(
        tki.is_bound(TopOpeBRepDSKind::Vertex, 9)
            && !tki.has_interferences(TopOpeBRepDSKind::Edge, 9)
    );
    assert!(tki.add(TopOpeBRepDSKind::Compound, 1).is_err());
    tki.change_interferences(TopOpeBRepDSKind::Vertex, 9)
        .clear();
    tki.init();
    assert_eq!(tki.value().map(|v| v.0), Some(TopOpeBRepDSKind::Point));
    Ok(())
}

#[test]
fn selectors_move_matching_interferences() -> Result<(), Failure> {
    let (a, b) = (tr(State::Out, State::In, 3), tr(State::In, State::In, 4));
    let mut l1 = loi(&[
        evi(&a, TopOpeBRepDSKind::Edge, 1, 9, 0.),
        evi(&b, TopOpeBRepDSKind::Face, 2, 8, 0.),
    ]);
    let mut l2 = ListOfInterference::default();
    assert_eq!(
        fun_select_traori_interference(&mut l1, Orientation::Internal, &mut l2)?,
        1
    );
    assert_eq!(
        (l1.extent(), l2.first().unwrap().get().unwrap().geometry()),
        (1, 8)
    );
    assert_eq!(fun_select_gi_interference(&mut l1, 0, &mut l2)?, 0);
    assert_eq!(
        fun_select_sk_interference(&mut l1, TopOpeBRepDSKind::Edge, &mut l2)?,
        2
    );
    let mut l3 = loi(&[evi(&a, TopOpeBRepDSKind::Edge, 1, 9, 0.)]);
    let mut l4 = ListOfInterference::default();
    assert_eq!(fun_select_itrasha_interference(&mut l3, 3, &mut l4)?, 1);
    let (mut l5, mut l6) = (
        loi(&[evi(&b, TopOpeBRepDSKind::Edge, 1, 9, 0.)]),
        ListOfInterference::default(),
    );
    assert!(fun_select_traint_interference(&l5, &mut l6)?);
    assert_eq!(
        fun_select_trasha_interference(&mut l5, ShapeEnum::Edge, &mut l6)?,
        1
    );
    Ok(())
}

#[test]
fn transitions_and_data_of_interferences() -> Result<(), Failure> {
    let t = tr(State::Out, State::In, 3);
    let i = evi(&t, TopOpeBRepDSKind::Edge, 1, 9, 0.25);
    assert_eq!(
        fds_data(&i),
        (TopOpeBRepDSKind::Vertex, 9, TopOpeBRepDSKind::Edge, 1)
    );
    assert_eq!(fds_tdata(&i), (ShapeEnum::Face, 3, ShapeEnum::Face, 3));
    assert_eq!(fds_parameter(&i)?, 0.25);
    assert_eq!(
        fds_parameter_checked(&Handle::new(TopOpeBRepDSInterference::new(
            &t,
            TopOpeBRepDSKind::Face,
            1,
            TopOpeBRepDSKind::Edge,
            2
        )))?,
        None
    );
    assert!(fun_transition_equal(&t, &t) && !fds_has_unk(&t));
    let mut u = TopOpeBRepDSTransition::default();
    u.set_index_before(5);
    fds_set_t(&mut u, &t);
    assert_eq!(
        (u.before(), u.after(), u.index_before()),
        (State::Out, State::In, 5)
    );
    assert!(fun_has_state_shape(&t, State::In, ShapeEnum::Face));
    let p = make_epv_interference(&t, 4, 2, 0.5, TopOpeBRepDSKind::Point, false);
    assert!(p.is_instance_name("TopOpeBRepDS_CurvePointInterference"));
    assert!(
        make_epv_interference_sk(
            &t,
            4,
            2,
            0.5,
            TopOpeBRepDSKind::Edge,
            TopOpeBRepDSKind::Face,
            false
        )
        .is_err()
    );
    let v = make_cpv_interference(&t, 1, 2, 0.5, TopOpeBRepDSKind::Point);
    assert_eq!(v.get().unwrap().support_type(), TopOpeBRepDSKind::Curve);
    Ok(())
}

#[test]
fn doubles_and_unknown_are_reduced() -> Result<(), Failure> {
    let (hds, _, edges, vertices) = structure()?;
    let ds = hds.get().unwrap().ds();
    let t = tr(State::Out, State::In, 3);
    let (e, v) = (edges[0], vertices[0]);
    let mut l = loi(&[
        evi(&t, TopOpeBRepDSKind::Edge, e, v, 0.),
        evi(&t, TopOpeBRepDSKind::Edge, e, v, 0.),
        cpi(
            &TopOpeBRepDSTransition::default(),
            TopOpeBRepDSKind::Edge,
            e,
            1,
            0.,
        ),
    ]);
    fun_reducedoublons(&mut l, &ds, e)?;
    assert_eq!(l.extent(), 2);
    drop(ds);
    fun_unkeep_unknown(&mut l, &mut hds.get().unwrap().change_ds(), e)?;
    assert_eq!(l.extent(), 1);
    Ok(())
}

#[test]
fn repvg_drops_single_interferences_per_face() -> Result<(), Failure> {
    let (hds, faces, edges, vertices) = structure()?;
    let ds = hds.get().unwrap().ds();
    let mut l = loi(&[evi(
        &tr(State::Out, State::In, faces[0]),
        TopOpeBRepDSKind::Edge,
        edges[0],
        vertices[0],
        0.,
    )]);
    let mut r = ListOfInterference::default();
    fds_repvg(&ds, edges[0], TopOpeBRepDSKind::Vertex, &mut l, &mut r)?;
    assert_eq!((l.extent(), r.extent()), (0, 0));
    Ok(())
}

#[test]
fn scanloi_and_get_tr_read_orientations() -> Result<(), Failure> {
    let (hds, faces, edges, vertices) = structure()?;
    let ds = hds.get().unwrap().ds();
    let l = loi(&[evi(
        &tr(State::Out, State::In, faces[0]),
        TopOpeBRepDSKind::Face,
        faces[0],
        vertices[0],
        0.,
    )]);
    let mut x: [ListOfInterference; 4] = Default::default();
    let [a, b, c, d] = &mut x;
    assert_eq!(fun_scanloi(&l, a, b, c, d)?, (1, 0, 0, 0));
    let ((stb, isb, bdim), (sta, isa, adim)) = fun_ds_get_tr(&ds, edges[0], 0, &l)?;
    assert_eq!(
        (stb, isb, bdim, sta, isa, adim),
        (State::Out, faces[0], 3, State::In, faces[0], 3)
    );
    Ok(())
}

#[test]
fn same_domain_queries() -> Result<(), Failure> {
    let (hds, faces, edges, _) = structure()?;
    {
        let mut ds = hds.get().unwrap().change_ds();
        let (f0, f1) = (
            ds.shape(faces[0], true).clone(),
            ds.shape(faces[1], true).clone(),
        );
        ds.fill_shapes_same_domain(&f0, &f1, true)?;
    }
    let ds = hds.get().unwrap().ds();
    let (f0, f1) = (
        ds.shape(faces[0], true).clone(),
        ds.shape(faces[1], true).clone(),
    );
    assert!(fun_ds_sdm(&ds, &f0, &f1) && fun_ds_getoov(&f0, &ds).unwrap().is_same(&f1));
    assert_eq!(fun_ds_get_vsdm(&ds, faces[0]), Some(faces[1]));
    let mut l = List::default();
    // FillShapesSameDomain orients the faces SAMEORIENTED: same domain in 2d, not in 3d
    assert!(!fds_has_same_domain3d(&ds, &f0, None));
    assert!(fds_has_same_domain2d(&ds, &f0, Some(&mut l)) && l.extent() == 1);
    assert!(fds_aresamdom(&ds, ds.shape(edges[0], true), &f0, &f1)?);
    drop(ds);
    assert!(fun_ds_getoov_hds(&f0, &hds)?.is_some());
    assert!(TopOpeBRepDSTOOL::share_g(&hds, faces[0], faces[1])?);
    Ok(())
}

#[test]
fn rank_split_and_config_of_edges() -> Result<(), Failure> {
    let (hds, _, edges, _) = structure()?;
    let ds = hds.get().unwrap().ds();
    let mut l: List<Shape> = edges.iter().map(|&i| ds.shape(i, true).clone()).collect();
    let mut l1 = List::default();
    fun_ds_sam_rk(&ds, 1, &mut l, &mut l1);
    assert_eq!((l.extent(), l1.extent()), (0, 12));
    let e = ds.shape(edges[0], true).clone();
    assert_eq!(
        fds_config3d(&e, &e)?,
        Some(TopOpeBRepDSConfig::SameOriented)
    );
    let (p1, p2) = {
        drop(ds);
        fds_getupperlower(&hds, edges[0], 0.5)?
    };
    assert!(p1 < p2);
    Ok(())
}

#[test]
fn connexity_of_the_box() -> Result<(), Failure> {
    let (hds, faces, edges, _) = structure()?;
    let s = hds.get().unwrap().ds().shape(1, true).clone();
    fdscnx_prepare(&s, &s, &hds)?;
    let e = hds.get().unwrap().ds().shape(edges[0], true).clone();
    assert_eq!(fdscnx_edge_connexity_same_shape(&e, &hds)?.extent(), 2);
    assert!(fdscnx_has_connex_face(&e, &hds)?);
    let f = hds.get().unwrap().ds().shape(faces[0], true).clone();
    assert_eq!(fdscnx_edge_connexity_same_shape(&f, &hds)?.extent(), 4);
    fds_edge_is_connex_to_same_domain_faces(&e, &hds)?;
    fdscnx_close();
    assert_eq!(fdscnx_edge_connexity_same_shape(&e, &hds)?.extent(), 0);
    Ok(())
}

#[test]
fn edge_interference_tool_on_internal_edge() -> Result<(), Failure> {
    let (hds, faces, edges, _) = structure()?;
    let e = hds
        .get()
        .unwrap()
        .ds()
        .shape(edges[0], true)
        .oriented(Orientation::Internal);
    let i = cpi(
        &tr(State::Out, State::In, faces[0]),
        TopOpeBRepDSKind::Edge,
        edges[0],
        1,
        0.,
    );
    let mut t = TopOpeBRepDSEdgeInterferenceTool::new();
    t.init(&e, &i)?;
    t.transition(&i)?;
    assert_eq!(
        i.get()
            .unwrap()
            .transition()
            .orientation(State::In, ShapeEnum::Face),
        Orientation::Internal
    );
    Ok(())
}

#[test]
fn face_tool_point_and_geometry_data() -> Result<(), Failure> {
    let (hds, faces, _, _) = structure()?;
    let ds = hds.get().unwrap().ds();
    let mut t = TopOpeBRepDSFaceInterferenceTool::new(&ds);
    assert!(!t.is_edge_pnt_par_def() && t.get_edge_pnt_par().is_err());
    t.set_edge_pnt_par(Point3::new(1., 2., 3.), 0.5);
    assert_eq!(t.get_edge_pnt_par()?.1, 0.5);
    let f = ds.shape(faces[0], true).clone();
    let (n, _, _, c1, c2) = fun_compute_geom_data(&f, rustycad_tkmath::gp::Point2::new(0.5, 0.5))?;
    assert_eq!((c1, c2), (0., 0.));
    assert!(
        (n.dot(fun_compute_geom_data_norm(
            &f,
            rustycad_tkmath::gp::Point2::new(0.5, 0.5)
        )?) - 1.)
            .abs()
            < 1e-12
    );
    assert!(fun_parameters(Point3::new(9., 9., 9.), &f)?.is_none());
    Ok(())
}

#[test]
fn reducers_keep_a_structure_without_interferences() -> Result<(), Failure> {
    let (hds, faces, edges, _) = structure()?;
    TopOpeBRepDSEIR::new(&hds).process_edge_interferences()?;
    TopOpeBRepDSFIR::new(&hds).process_face_interferences(&MapOfShapeOn1State::default())?;
    let mut f = TopOpeBRepDSFilter::new(&hds, None);
    f.process_interferences()?;
    f.process_face_interferences(&MapOfShapeOn1State::default())?;
    TopOpeBRepDSReducer::new(&hds).process_edge_interferences()?;
    let ds = hds.get().unwrap().ds();
    assert!(ds.shape_interferences_index(edges[0], true)?.is_empty());
    assert!(ds.shape_interferences_index(faces[0], true)?.is_empty());
    Ok(())
}

#[test]
fn eir_drops_interferences_on_no_face_or_edge() -> Result<(), Failure> {
    let (hds, faces, edges, vertices) = structure()?;
    {
        let mut ds = hds.get().unwrap().change_ds();
        let e = ds.shape(edges[0], true).clone();
        let t =
            TopOpeBRepDSTransition::new(State::Out, State::In, ShapeEnum::Solid, ShapeEnum::Solid);
        ds.change_shape_interferences(&e).append(evi(
            &t,
            TopOpeBRepDSKind::Face,
            faces[0],
            vertices[0],
            0.,
        ));
        ds.change_shape_interferences(&e).append(evi(
            &tr(State::Out, State::In, faces[1]),
            TopOpeBRepDSKind::Face,
            faces[1],
            vertices[0],
            0.,
        ));
    }
    TopOpeBRepDSEIR::new(&hds).process_edge_interferences_index(edges[0])?;
    assert_eq!(
        hds.get()
            .unwrap()
            .ds()
            .shape_interferences_index(edges[0], true)?
            .extent(),
        1
    );
    Ok(())
}

#[test]
fn completions_without_section_edges_change_nothing() -> Result<(), Failure> {
    let (hds, ..) = structure()?;
    for f in [
        fun_ds_complete_for_se1,
        fun_ds_complete_for_se2,
        fun_ds_complete_for_se3,
        fun_ds_complete_for_se4,
        fun_ds_complete_for_se5,
        fun_ds_complete_for_se6,
        fun_ds_complete_for_se8,
        fun_ds_complete_for_se9,
        fun_ds_complete1d_for_se_sdm,
        fun_ds_complete_for_e7,
        fun_ds_purge_for_e9,
        fun_ds_point_to_vertex,
        fun_ds_redusamsha,
        fun_ds_fill_sdm_faces,
        fun_ds_add_se_sdm1d,
    ] {
        f(&hds)?;
    }
    fun_ds_feig_b1_to0(&hds, &MapOfShapeOn1State::default())?;
    assert_eq!(hds.get().unwrap().ds().nb_section_edges(), 0);
    Ok(())
}

#[test]
fn check_and_dumper_texts() -> Result<(), Failure> {
    let (hds, faces, edges, vertices) = structure()?;
    {
        let mut ds = hds.get().unwrap().change_ds();
        let e = ds.shape(edges[0], true).clone();
        ds.change_shape_interferences(&e).append(evi(
            &tr(State::Out, State::In, faces[0]),
            TopOpeBRepDSKind::Face,
            faces[0],
            vertices[0],
            0.,
        ));
    }
    let mut c = TopOpeBRepDSCheck::with_hds(&hds);
    assert!(c.chk_intg()?);
    assert!(c.chk_intg_sam_dom()? && c.one_vertex_on_pnt()?);
    let mut out = Vec::new();
    c.print_intg(&mut out)?;
    assert!(String::from_utf8_lossy(&out).contains("Face\t(/1)\tnumber "));
    TopOpeBRepDSHDataStructure::chk_intg(&hds)?;
    let d = TopOpeBRepDSDumper::new(&hds);
    assert_eq!(d.sprint_shape_index(1)?, "*(SO, 1)");
    assert_eq!(d.sprint_shape_index(0)?, "");
    let s = hds.get().unwrap().ds().shape(faces[0], true).clone();
    let name = format!("*(FA, {})", faces[0]);
    assert_eq!(
        d.sprint_shape_ref_ori(&s, "> ")?,
        format!("> {name} ({name},SAME)")
    );
    Ok(())
}

#[test]
fn tool_eshareg_and_esd_on_a_box() -> Result<(), Failure> {
    let (hds, faces, edges, _) = structure()?;
    let e = topods::edge(hds.get().unwrap().ds().shape(edges[0], true))?;
    let mut l = List::default();
    assert_eq!(TopOpeBRepDSTOOL::e_share_g(&hds, &e, &mut l)?, 0);
    let f = hds.get().unwrap().ds().shape(faces[0], true).clone();
    assert_eq!(TopOpeBRepDSTOOL::get_esd(&hds, &f, edges[0])?, None);
    assert_eq!(
        TopOpeBRepDSTOOL::get_config(&hds, &MapOfShapeOn1State::default(), edges[0], edges[1])?,
        None
    );
    Ok(())
}
