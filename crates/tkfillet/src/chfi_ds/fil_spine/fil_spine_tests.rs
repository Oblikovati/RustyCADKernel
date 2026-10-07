use super::*;
use crate::chfi_ds::shared::{lock, shared};
use crate::chfi_ds::{SpineHandle, State};
use rustycad_tkbrep::topods::Edge;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

fn segment(from: Point3, to: Point3) -> Edge {
    BRepBuilderAPIMakeEdge::from_points(from, to)
        .unwrap()
        .edge()
        .unwrap()
}

/// Three unit segments along X: abscissa [1, 2, 3].
fn loaded_spine() -> FilSpine {
    let mut spine = FilSpine::new();
    for i in 0..3 {
        let x = f64::from(i);
        spine.set_edges(&segment(
            Point3::new(x, 0., 0.),
            Point3::new(x + 1., 0., 0.),
        ));
    }
    spine.load().unwrap();
    spine
}

fn el_spine(first: f64, last: f64) -> Handle<ElSpine> {
    let curve: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
        Point3::default(),
        Direction3::default(),
    ))
    .cast();
    let el_spine = ElSpine::new();
    el_spine.set_curve(&curve).unwrap();
    el_spine.set_first_parameter(first);
    el_spine.set_last_parameter(last);
    Handle::new(el_spine)
}

#[test]
fn edge_radii_are_kept_sorted_and_overwritten_at_equal_abscissae() {
    let mut spine = loaded_spine();
    let edge1 = spine.edges(1).unwrap().clone();
    let edge2 = spine.edges(2).unwrap().clone();
    spine.set_radius_on_edge(1.0, &edge1).unwrap();
    spine.set_radius_on_edge(2.0, &edge2).unwrap();
    let xs: Vec<f64> = spine.par_and_rad().iter().map(|pair| pair.x()).collect();
    assert_eq!(xs, vec![0.0, 1.0, 2.0]);
    // The shared end abscissa 1 keeps the last radius set there, as natively.
    assert_eq!(spine.par_and_rad()[1].y(), 2.0);
    assert!(spine.is_constant_on_edge(2).unwrap());
    assert!(!spine.is_constant_on_edge(1).unwrap());
    assert_eq!(spine.radius_on_edge(&edge2).unwrap(), 2.0);
    assert!(!spine.is_constant());
    assert_eq!(
        spine.radius().unwrap_err().message(),
        "Spine is not constant"
    );
}

#[test]
fn variable_edge_radius_raises_domain_error() {
    let mut spine = loaded_spine();
    spine.set_radius_at(Xy::new(0.0, 1.0), 1).unwrap();
    spine.set_radius_at(Xy::new(1.0, 3.0), 1).unwrap();
    let failure = spine.radius_on_edge_index(1).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_DomainError");
    assert!(!spine.is_constant_on_edge(1).unwrap());
}

#[test]
fn unset_radius_on_edge_removes_its_span() {
    let mut spine = loaded_spine();
    spine.set_radius(2.0).unwrap();
    let edge3 = spine.edges(3).unwrap().clone();
    spine.set_radius_on_edge(2.0, &edge3).unwrap();
    assert_eq!(spine.par_and_rad().len(), 3);
    spine.unset_radius_on_edge(&edge3).unwrap();
    assert_eq!(spine.par_and_rad().len(), 1);
}

#[test]
fn reset_completes_the_radii_to_the_bounds() {
    let mut spine = loaded_spine();
    spine.set_radius_at(Xy::new(1.0, 4.0), 0).unwrap();
    SpineObject::reset(&mut spine, false).unwrap();
    let xs: Vec<f64> = spine.par_and_rad().iter().map(|pair| pair.x()).collect();
    assert_eq!(xs, vec![0.0, 1.0, 3.0]);
    SpineObject::reset(&mut spine, true).unwrap();
    assert!(spine.par_and_rad().is_empty());
    let failure = SpineObject::reset(&mut spine, false).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_NoSuchObject");
}

#[test]
fn constant_spine_law_is_one_constant_piece() {
    let mut spine = loaded_spine();
    spine.set_radius(2.0).unwrap();
    let els = el_spine(0.0, 3.0);
    spine.append_el_spine(&els).unwrap();
    let law = spine.law(&els).unwrap();
    let composite = law.get().unwrap();
    assert_eq!(composite.value(1.5).unwrap(), 2.0);
    assert_eq!(spine.max_rad_from_seq_and_laws().unwrap(), 2.0);
    assert!(spine.law(&el_spine(0.0, 1.0)).unwrap().is_null());
}

#[test]
fn variable_middle_edge_gets_an_interpolated_piece() {
    let mut spine = loaded_spine();
    let (edge1, edge3) = (
        spine.edges(1).unwrap().clone(),
        spine.edges(3).unwrap().clone(),
    );
    spine.set_radius_on_edge(1.0, &edge1).unwrap();
    spine.set_radius_on_edge(2.0, &edge3).unwrap();
    let els = el_spine(0.0, 3.0);
    spine.append_el_spine(&els).unwrap();
    let law = spine.law(&els).unwrap();
    let composite = law.get().unwrap();
    assert_eq!(composite.value(0.5).unwrap(), 1.0);
    assert_eq!(composite.value(2.5).unwrap(), 2.0);
    let middle = composite.value(1.5).unwrap();
    assert!(middle > 1.0 && middle < 2.0);
    spine.set_split_done(true);
    let edge2 = spine.edges(2).unwrap().clone();
    assert!(spine.change_law(&edge2, |law| law.is_some()).unwrap());
    assert!(spine.change_law(&edge1, |_| ()).is_err());
}

#[test]
fn without_radius_every_edge_reads_as_constant_zero() {
    // Natively IsConstant(IE) is true and Radius(IE) is 0 when no radius is set.
    let spine = loaded_spine();
    assert!(spine.is_constant_on_edge(2).unwrap());
    let law = spine.compute_law(&el_spine(0.0, 3.0)).unwrap();
    assert_eq!(law.get().unwrap().value(1.0).unwrap(), 0.0);
    assert!(!spine.is_constant());
}

#[test]
fn periodic_spine_without_constant_edge_interpolates_once() {
    let mut spine = loaded_spine();
    spine.set_first_status(State::Closed);
    spine.set_radius_at(Xy::new(0.0, 1.0), 0).unwrap();
    spine.set_radius_at(Xy::new(1.5, 2.0), 0).unwrap();
    let els = el_spine(0.0, 3.0);
    els.get().unwrap().set_periodic(true);
    let law = spine.compute_law(&els).unwrap();
    let composite = law.get().unwrap();
    assert!(composite.is_periodic());
    assert!((composite.value(1.5).unwrap() - 2.0).abs() < 1e-12);
}

#[test]
fn split_done_recomputes_the_law_holding_the_radius() {
    let mut spine = loaded_spine();
    spine.set_radius(1.0).unwrap();
    let els = el_spine(0.0, 3.0);
    spine.append_el_spine(&els).unwrap();
    spine.set_split_done(true);
    spine.set_radius_at(Xy::new(1.5, 3.0), 0).unwrap();
    let law = spine.law(&els).unwrap();
    assert!((law.get().unwrap().value(1.5).unwrap() - 3.0).abs() < 1e-12);
    assert_eq!(spine.max_rad_from_seq_and_laws().unwrap(), 3.0);
}

#[test]
fn a_law_radius_is_dropped_as_natively() {
    let mut spine = loaded_spine();
    spine.set_radius(1.0).unwrap();
    let law: Handle<dyn LawFunction> =
        Handle::new(rustycad_tkgeomalgo::law::LawConstant::new()).cast();
    spine.set_radius_law(&law, 0);
    assert!(spine.par_and_rad().is_empty());
}

#[test]
fn polymorphic_handle_dispatches_to_the_fillet_reset() {
    let mut fillet = loaded_spine();
    fillet.set_radius(1.0).unwrap();
    let handle: SpineHandle = shared(fillet);
    lock(&handle).reset(true).unwrap();
    assert!(
        lock(&handle)
            .as_fil_spine()
            .unwrap()
            .par_and_rad()
            .is_empty()
    );
    assert!(lock(&handle).as_chamf_spine().is_none());
    assert_eq!(lock(&handle).el_spines().len(), 0);
    assert!(el_spine(0.0, 1.0).get().unwrap().first_parameter().unwrap() == 0.0);
}
