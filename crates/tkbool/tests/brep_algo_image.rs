//! Differential behavior tests for `BRepAlgo_Image` root and image links.
use rustycad_tkbool::brep_algo::BRepAlgoImage;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Compound, Shape, Vertex},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;

#[test]
fn direct_and_terminal_images_preserve_root_chain_order() {
    let source = vertex(0.0);
    let middle = vertex(1.0);
    let image = vertex(2.0);
    let mut links = BRepAlgoImage::new();
    links.set_root(&source);
    links.bind(&source, &middle).unwrap();
    links.set_root(&middle);
    links.bind(&middle, &image).unwrap();

    assert_eq!(links.image(&source), vec![middle.clone()]);
    assert_eq!(links.last_image(&source), vec![image.clone()]);
    assert!(links.root(&image).unwrap().is_same(&source));
    assert!(links.image_from(&image).unwrap().is_same(&middle));
    assert_eq!(links.roots(), &[source, middle]);
}

#[test]
fn bind_add_remove_and_clear_match_image_map_state() {
    let source = vertex(0.0);
    let first = vertex(1.0);
    let second = vertex(2.0);
    let mut links = BRepAlgoImage::new();
    links.bind(&source, &first).unwrap();
    links.add(&source, &second).unwrap();
    assert_eq!(links.image(&source), vec![first.clone(), second.clone()]);

    links.remove(&first).unwrap();
    assert_eq!(links.image(&source), vec![second.clone()]);
    links.remove(&second).unwrap();
    assert!(!links.has_image(&source));
    assert_eq!(links.image(&source), vec![source.clone()]);

    links.set_root(&source);
    links.clear();
    assert!(links.roots().is_empty());
    assert!(!links.is_image(&source));
}

#[test]
fn bind_many_and_add_many_keep_image_insertion_order() {
    let source = vertex(0.0);
    let first = vertex(1.0);
    let second = vertex(2.0);
    let third = vertex(3.0);
    let mut links = BRepAlgoImage::new();
    links
        .bind_many(&source, &[first.clone(), second.clone()])
        .unwrap();
    links
        .add_many(&source, std::slice::from_ref(&third))
        .unwrap();

    assert_eq!(links.image(&source), vec![first, second, third]);
}

#[test]
fn remove_root_and_replace_root_update_roots_and_links() {
    let old = vertex(0.0);
    let new = vertex(1.0);
    let image = vertex(2.0);
    let mut links = BRepAlgoImage::new();
    links.set_root(&old);
    links.bind(&old, &image).unwrap();
    links.replace_root(&old, &new).unwrap();
    assert_eq!(links.roots(), std::slice::from_ref(&new));
    assert!(links.image_from(&image).unwrap().is_same(&new));

    links.remove_root(&new);
    assert!(links.roots().is_empty());
    assert!(!links.has_image(&new));
    assert!(!links.is_image(&image));
}

#[test]
fn duplicate_bindings_report_construction_errors_without_mutation() {
    let source = vertex(0.0);
    let first = vertex(1.0);
    let second = vertex(2.0);
    let mut links = BRepAlgoImage::new();
    links.bind(&source, &first).unwrap();

    assert_eq!(
        links.bind(&source, &second).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    assert_eq!(
        links
            .add(&vertex(3.0), &second)
            .unwrap_err()
            .exception_type(),
        "Standard_ConstructionError"
    );
    assert_eq!(links.image(&source), vec![first]);
}

#[test]
fn shared_image_rebinds_its_direct_generator_but_keeps_all_source_links() {
    let first_source = vertex(0.0);
    let second_source = vertex(1.0);
    let image = vertex(2.0);
    let mut links = BRepAlgoImage::new();
    links.bind(&first_source, &image).unwrap();

    links.bind(&second_source, &image).unwrap();
    assert_eq!(links.image(&second_source), vec![image.clone()]);
    assert!(links.image(&first_source).contains(&image));
    assert!(links.image_from(&image).unwrap().is_same(&second_source));
    assert!(links.root(&image).unwrap().is_same(&second_source));
}

#[test]
fn compact_replaces_intermediate_links_with_terminal_images() {
    let source = vertex(0.0);
    let middle = vertex(1.0);
    let image = vertex(2.0);
    let mut links = BRepAlgoImage::new();
    links.set_root(&source);
    links.bind(&source, &middle).unwrap();
    links.bind(&middle, &image).unwrap();

    links.compact().unwrap();

    assert_eq!(links.image(&source), vec![image.clone()]);
    assert_eq!(links.image_from(&image).unwrap(), source);
}

#[test]
fn filter_removes_images_missing_from_the_selected_shape() {
    let source_a = vertex(0.0);
    let source_b = vertex(1.0);
    let image_a = vertex(2.0);
    let image_b = vertex(3.0);
    let mut links = BRepAlgoImage::new();
    links.bind(&source_a, &image_a).unwrap();
    links.bind(&source_b, &image_b).unwrap();
    let mut selected = Compound::default();
    BRepBuilder::default().make_compound(&mut selected);
    BRepBuilder::default().add(&mut selected, &image_a).unwrap();

    links
        .filter(&selected.clone().into_shape(), ShapeEnum::Vertex)
        .unwrap();

    assert!(links.has_image(&source_a));
    assert!(!links.has_image(&source_b));
}

fn vertex(x: f64) -> Shape {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, Point3::new(x, 0.0, 0.0), 1.0e-7)
        .unwrap();
    vertex.into_shape()
}
