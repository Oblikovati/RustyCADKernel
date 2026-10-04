//! Native BRepAlgo_AsDes association state and offset-intersection dependencies.
use rustycad_tkbool::brep_algo::BRepAlgoAsDes;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Shape, Vertex},
};
use rustycad_tkernel::{
    ncollection::List,
    standard::{Failure, RuntimeTyped},
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point3;

#[test]
fn ancestry_mutation_and_failures_match_pinned_native_state() -> Result<(), Failure> {
    let [a, b, c, d, e] = vertices()?;
    let mut associations = BRepAlgoAsDes::default();
    add_repeated_descendants(&mut associations, &a, &b, &c);
    assert_initial_state(&mut associations, &a, &b, &c, &d)?;
    replace_and_remove(&mut associations, &a, &b, &d, &e)?;
    assert_add_many(&mut associations, &a, &b, &c, &d, &e);
    Ok(())
}

#[test]
fn replace_merges_into_existing_destination_without_losing_links() -> Result<(), Failure> {
    let [a, b, _, d, e] = vertices()?;
    let mut associations = BRepAlgoAsDes::new();
    associations.add(&a, &b);
    associations.add(&e, &d);
    associations.replace(&a, &e);
    assert_eq!(associations.descendant(&e).count(), 2);
    assert_eq!(associations.ascendant(&b).count(), 1);
    assert!(associations.ascendant(&b).any(|shape| shape.is_same(&e)));
    Ok(())
}

fn add_repeated_descendants(associations: &mut BRepAlgoAsDes, a: &Shape, b: &Shape, c: &Shape) {
    associations.add(a, &b.oriented(Orientation::Reversed));
    associations.add(c, b);
    associations.add(a, b);
}

fn assert_common_descendants(associations: &BRepAlgoAsDes, a: &Shape, b: &Shape, c: &Shape) {
    let mut common = List::default();
    common.append(b.clone());
    assert!(associations.has_common_descendant(a, c, &mut common));
    assert_eq!(common.len(), 2);
    assert_eq!(common.iter().filter(|shape| shape.is_same(b)).count(), 2);
}

fn assert_initial_state(
    associations: &mut BRepAlgoAsDes,
    a: &Shape,
    b: &Shape,
    c: &Shape,
    d: &Shape,
) -> Result<(), Failure> {
    assert!(associations.has_descendant(a) && associations.has_ascendant(b));
    assert_eq!(associations.descendant(a).count(), 2);
    assert_eq!(associations.ascendant(b).count(), 3);
    assert!(associations.change_descendant(d).is_none());
    assert_common_descendants(associations, a, b, c);
    let error = associations.remove(a).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    Ok(())
}

fn replace_and_remove(
    associations: &mut BRepAlgoAsDes,
    a: &Shape,
    b: &Shape,
    d: &Shape,
    e: &Shape,
) -> Result<(), Failure> {
    associations.change_descendant(a).unwrap().append(d.clone());
    assert!(!associations.has_ascendant(d));
    associations.replace(a, e);
    assert_eq!(associations.descendant(e).count(), 3);
    assert_eq!(associations.ascendant(b).count(), 2);
    associations.remove(b)?;
    assert!(!associations.has_ascendant(b));
    assert_eq!(associations.descendant(e).count(), 2);
    Ok(())
}

fn assert_add_many(
    associations: &mut BRepAlgoAsDes,
    a: &Shape,
    b: &Shape,
    c: &Shape,
    d: &Shape,
    e: &Shape,
) {
    associations.clear();
    assert_eq!(BRepAlgoAsDes::static_type().name(), "BRepAlgo_AsDes");
    assert!(!associations.has_descendant(e));
    associations.add_many(a, &List::from_iter([b.clone(), c.clone()]));
    assert_eq!(associations.descendant(a).count(), 2);
    assert_eq!(
        associations.remove(d).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
}

fn vertices() -> Result<[Shape; 5], Failure> {
    let builder = BRepBuilder::default();
    let points = [0., 1., 2., 3., 4.];
    let mut result = Vec::new();
    for x in points {
        let mut vertex = Vertex::default();
        builder.make_vertex_at(&mut vertex, Point3::new(x, 0., 0.), 1e-7)?;
        result.push(vertex.into_shape());
    }
    result.try_into().map_err(|shapes: Vec<Shape>| {
        Failure::with_message(
            "Standard_ConstructionError",
            &format!("built {} vertices; expected 5", shapes.len()),
        )
    })
}
