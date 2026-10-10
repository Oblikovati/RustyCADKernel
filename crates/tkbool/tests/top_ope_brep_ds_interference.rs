//! TopOpeBRepDS kinds, transitions and interferences: the package names, the
//! transition orientations and the interference hierarchy shared through handles.
use rustycad_tkbool::top_ope_brep_ds::{
    TopOpeBRepDSConfig, TopOpeBRepDSCurvePointInterference, TopOpeBRepDSEdgeVertexInterference,
    TopOpeBRepDSFaceEdgeInterference, TopOpeBRepDSInterference, TopOpeBRepDSKind,
    TopOpeBRepDSShapeShapeInterference, TopOpeBRepDSSurfaceCurveInterference,
    TopOpeBRepDSTransition, kind_to_shape, shape_to_kind, sprint_kind_index,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};

#[test]
fn kinds_name_and_convert_shape_types() -> Result<(), Failure> {
    assert_eq!(
        sprint_kind_index(TopOpeBRepDSKind::Edge, 3, "a", "b"),
        "a(ED, 3)b"
    );
    assert_eq!(
        sprint_kind_index(TopOpeBRepDSKind::Face, 12, "", ""),
        "(FA,12)"
    );
    assert_eq!(shape_to_kind(ShapeEnum::Solid)?, TopOpeBRepDSKind::Solid);
    assert_eq!(kind_to_shape(TopOpeBRepDSKind::Curve), ShapeEnum::Shape);
    let error = shape_to_kind(ShapeEnum::Shape)
        .err()
        .map(|e| e.exception_type().to_string());
    assert_eq!(error.as_deref(), Some("Standard_ProgramError"));
    Ok(())
}

#[test]
fn a_forward_transition_complements_to_reversed() {
    let t = TopOpeBRepDSTransition::from_orientation(Orientation::Forward);
    assert_eq!((t.before(), t.after()), (State::Out, State::In));
    assert_eq!(
        t.orientation(State::In, ShapeEnum::Face),
        Orientation::Forward
    );
    let c = t.complement();
    assert_eq!((c.before(), c.after()), (State::In, State::Out));
    assert_eq!(
        c.orientation(State::In, ShapeEnum::Face),
        Orientation::Reversed
    );
}

#[test]
fn on_transitions_swap_states_and_shapes() {
    let t = TopOpeBRepDSTransition::new(State::On, State::In, ShapeEnum::Edge, ShapeEnum::Face);
    assert_eq!(
        t.orientation(State::In, ShapeEnum::Face),
        Orientation::Forward
    );
    assert_eq!(
        t.orientation(State::Out, ShapeEnum::Face),
        Orientation::Reversed
    );
    let c = t.complement();
    assert_eq!(
        (c.before(), c.after(), c.shape_before(), c.shape_after()),
        (State::In, State::On, ShapeEnum::Face, ShapeEnum::Edge)
    );
}

#[test]
fn different_indices_have_no_common_index() {
    let mut t = TopOpeBRepDSTransition::default();
    t.set_index(4);
    assert_eq!(t.index().ok(), Some(4));
    t.set_index_after(5);
    assert!(t.index().is_err());
}

#[test]
fn derived_interferences_cast_to_their_bases_and_back() {
    let t = TopOpeBRepDSTransition::from_orientation(Orientation::Reversed);
    let ev: Handle<TopOpeBRepDSInterference> =
        Handle::new(TopOpeBRepDSEdgeVertexInterference::on_edge(
            &t,
            2,
            7,
            true,
            TopOpeBRepDSConfig::SameOriented,
            0.5,
        ))
        .cast();
    let base = ev.get().map(|i| i.gkgsks());
    assert_eq!(
        base,
        Some((TopOpeBRepDSKind::Vertex, 7, TopOpeBRepDSKind::Edge, 2))
    );
    let ss: Handle<TopOpeBRepDSShapeShapeInterference> = ev.cast();
    assert_eq!(
        ss.get().map(|i| (i.gbound(), i.config())),
        Some((true, TopOpeBRepDSConfig::SameOriented))
    );
    let back: Handle<TopOpeBRepDSEdgeVertexInterference> = ev.cast();
    assert_eq!(back.get().map(|i| i.parameter()), Some(0.5));
    let cp: Handle<TopOpeBRepDSCurvePointInterference> = ev.cast();
    assert!(cp.is_null());
}

#[test]
fn interferences_are_modified_through_any_handle() -> Result<(), Failure> {
    let t = TopOpeBRepDSTransition::default();
    let fe = Handle::new(TopOpeBRepDSFaceEdgeInterference::new(
        &t,
        1,
        3,
        false,
        TopOpeBRepDSConfig::DiffOriented,
    ));
    let base: Handle<TopOpeBRepDSInterference> = fe.clone().cast();
    base.get()
        .ok_or_else(|| Failure::new("null"))?
        .set_support(9);
    fe.get()
        .ok_or_else(|| Failure::new("null"))?
        .change_transition(|t| t.set_index(6));
    let f = fe.get().ok_or_else(|| Failure::new("null"))?;
    assert_eq!((f.support(), f.transition().index()?), (9, 6));
    let other: Handle<TopOpeBRepDSInterference> = Handle::new(TopOpeBRepDSInterference::new(
        &t,
        TopOpeBRepDSKind::Face,
        9,
        TopOpeBRepDSKind::Curve,
        3,
    ));
    assert!(f.has_same_support(&other)?);
    assert!(!f.has_same_geometry(&other)?);
    Ok(())
}

#[test]
fn a_surface_curve_copy_has_no_pcurve() -> Result<(), Failure> {
    let t = TopOpeBRepDSTransition::default();
    let i: Handle<TopOpeBRepDSInterference> = Handle::new(TopOpeBRepDSInterference::new(
        &t,
        TopOpeBRepDSKind::Surface,
        1,
        TopOpeBRepDSKind::Curve,
        2,
    ));
    let sc = TopOpeBRepDSSurfaceCurveInterference::from_interference(&i)?;
    assert!(sc.pcurve().is_null());
    assert_eq!((sc.support(), sc.geometry()), (1, 2));
    Ok(())
}
