//! Native differential for two connected planar supports with one shared convex edge.
use rustycad_tkbrep::{
    brep_adaptor::BRepAdaptorSurface,
    topexp::{self, Explorer},
    topods::{Builder, Edge, Face, Shape, Shell},
    toptools::IndexedDataMapOfShapeListOfShape,
};
use rustycad_tkfillet::chfi_ds::TypeOfConcavity;
use rustycad_tkg3d::{adaptor3d::Adaptor3dSurface, top_abs::ShapeEnum};
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::geom_abs::{JoinType, SurfaceType};
use rustycad_tkoffset::brep_offset::{
    BRepOffsetAnalyse, BRepOffsetError, BRepOffsetMakeOffset, BRepOffsetMode,
};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder};
use rustycad_tktopalgo::{
    brep_builder_api::BRepBuilderAPIMakeShape, brep_check::BRepCheckAnalyzer, brep_gprop::BRepGProp,
};

#[test]
fn adjacent_planes_gain_the_native_rounded_shared_edge_image()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let (input, faces, shared_edge) = adjacent_box_faces()?;
    let mut maker = maker_for_offset(&input, 0.5);

    maker.make_offset_shape()?;

    assert!(maker.is_done());
    assert_eq!(maker.error(), Some(BRepOffsetError::NoError));
    assert_eq!(count(maker.shape(), ShapeEnum::Face)?, 3);
    assert_eq!(count(maker.shape(), ShapeEnum::Edge)?, 12);
    assert!(BRepCheckAnalyzer::new(maker.shape(), true, false, false)?.is_valid()?);
    assert!(faces.iter().all(|face| {
        maker
            .offset_faces_from_shapes()
            .has_image(&face.clone().into_shape())
    }));
    assert!(!maker.offset_edges_from_shapes().has_image(&shared_edge));
    let face_images = [
        maker.generated(&faces[0].clone().into_shape())?,
        maker.generated(&faces[1].clone().into_shape())?,
    ];
    assert!(face_images.iter().all(|images| images.len() == 1));
    assert!(
        faces
            .iter()
            .all(|face| maker.modified(&face.clone().into_shape()).is_empty())
    );
    let edge_generated = maker.generated(&shared_edge)?;
    assert_eq!(edge_generated.len(), 1);
    assert_eq!(edge_generated[0].shape_type()?, ShapeEnum::Face);
    assert_free_edge_history(&mut maker, &faces[0], &shared_edge)?;
    assert_shared_vertex_history(&mut maker, &shared_edge)?;
    assert_surface_manifest(maker.shape())?;
    assert_native_area(maker.shape())?;
    Ok(())
}

#[test]
fn negative_adjacent_plane_offset_fails_closed_until_boundaries_are_trimmed()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let (input, _, _) = adjacent_box_faces()?;
    let mut maker = maker_for_offset(&input, -0.5);
    let failure = maker.make_offset_shape().unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_NotImplemented");
    assert!(failure.message().contains("Some(Convex)"));
    assert!(failure.message().contains("Concave"));
    assert!(!maker.is_done());
    Ok(())
}

#[test]
fn cylinder_and_planar_cap_gain_the_native_toroidal_join()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let (input, faces, shared_edge) = cylinder_cap_shell()?;
    let mut maker = maker_for_offset(&input, 0.5);

    maker.make_offset_shape()?;

    assert!(maker.is_done());
    assert_eq!(maker.error(), Some(BRepOffsetError::NoError));
    assert_eq!(count(maker.shape(), ShapeEnum::Face)?, 3);
    assert_eq!(count(maker.shape(), ShapeEnum::Edge)?, 9);
    assert_eq!(count(maker.shape(), ShapeEnum::Shell)?, 1);
    assert_eq!(maker.shape().shape_type()?, ShapeEnum::Shell);
    assert_total_area(maker.shape(), 176.596_428_261_508_4)?;
    assert!(faces.iter().all(|face| {
        maker
            .offset_faces_from_shapes()
            .has_image(&face.clone().into_shape())
    }));
    assert!(!maker.offset_edges_from_shapes().has_image(&shared_edge));
    let generated = maker.generated(&shared_edge)?;
    assert_eq!(generated.len(), 1);
    assert_eq!(generated[0].shape_type()?, ShapeEnum::Face);
    assert_shared_vertex_history(&mut maker, &shared_edge)?;
    assert_cylinder_plane_interval(&input, &shared_edge)?;
    assert_mixed_support_manifest(maker.shape())?;
    assert!(BRepCheckAnalyzer::new(maker.shape(), true, false, false)?.is_valid()?);
    Ok(())
}

#[test]
fn cylindrical_side_with_both_caps_builds_two_native_toroidal_joins()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let (input, faces) = cylinder_closed_shell()?;
    let mut maker = maker_for_offset(&input, 0.5);

    maker.make_offset_shape()?;

    assert!(maker.is_done());
    assert_eq!(maker.error(), Some(BRepOffsetError::NoError));
    assert_eq!(count(maker.shape(), ShapeEnum::Face)?, 5);
    assert_eq!(count(maker.shape(), ShapeEnum::Edge)?, 14);
    assert_eq!(count(maker.shape(), ShapeEnum::Shell)?, 1);
    assert_total_area(maker.shape(), 221.245_965_072_245_5)?;
    assert!(faces.iter().all(|face| {
        maker
            .offset_faces_from_shapes()
            .has_image(&face.clone().into_shape())
    }));
    assert_generated_connectors(&mut maker, &faces)?;
    assert!(BRepCheckAnalyzer::new(maker.shape(), true, false, false)?.is_valid()?);
    assert_surface_type_counts(
        maker.shape(),
        [
            (SurfaceType::Plane, 2),
            (SurfaceType::Cylinder, 1),
            (SurfaceType::Torus, 2),
        ],
    )?;
    Ok(())
}

fn maker_for_offset(input: &Shape, offset: f64) -> BRepOffsetMakeOffset {
    let mut maker = BRepOffsetMakeOffset::new();
    maker.initialize(
        input,
        offset,
        1.0e-7,
        BRepOffsetMode::Skin,
        false,
        false,
        JoinType::Arc,
        false,
        false,
    );
    maker
}

fn assert_cylinder_plane_interval(
    input: &Shape,
    shared_edge: &Shape,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let offset: f64 = 0.5;
    let tolerance: f64 = 1.0e-7;
    let angle = 4.0 * (tolerance / (offset.abs() * 0.5 + 1.0e-7)).min(1.0).asin();
    let analysis = BRepOffsetAnalyse::with_shape(input, angle)?;
    let edge = Edge::unchecked(shared_edge.clone());
    let intervals = analysis.type_of(&edge)?;
    assert_eq!(intervals.len(), 1);
    assert_eq!(intervals[0].get_type()?, TypeOfConcavity::Convex);
    Ok(())
}

fn assert_total_area(
    shape: &Shape,
    expected: f64,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let mut properties = GProps::default();
    BRepGProp::surface_properties(shape, &mut properties, false, false)?;
    assert!((properties.mass() - expected).abs() < 1.0e-9);
    Ok(())
}

fn assert_surface_type_counts(
    shape: &Shape,
    expected: [(SurfaceType, usize); 3],
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?;
    let mut surface_types = Vec::new();
    for face in faces {
        let face = Face::unchecked(face);
        surface_types.push(BRepAdaptorSurface::with_face(&face, false)?.get_type()?);
    }
    for (kind, expected_count) in expected {
        let actual_count = surface_types
            .iter()
            .filter(|actual| **actual == kind)
            .count();
        assert_eq!(actual_count, expected_count, "surface kind {kind:?}");
    }
    Ok(())
}

fn assert_generated_connectors(
    maker: &mut BRepOffsetMakeOffset,
    faces: &[Face],
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let mut tested = Vec::new();
    for (index, face) in faces.iter().enumerate() {
        let shape = face.clone().into_shape();
        for candidate in Explorer::new(&shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if tested.iter().any(|edge: &Shape| edge.is_same(&candidate)) {
                continue;
            }
            if !has_distinct_face_ancestor(faces, index, &candidate)? {
                continue;
            }
            tested.push(candidate.clone());
            assert!(!maker.offset_edges_from_shapes().has_image(&candidate));
            let generated = maker.generated(&candidate)?;
            assert_eq!(generated.len(), 1);
            assert_eq!(generated[0].shape_type()?, ShapeEnum::Face);
        }
    }
    assert_eq!(tested.len(), 2);
    Ok(())
}

fn has_distinct_face_ancestor(
    faces: &[Face],
    index: usize,
    edge: &Shape,
) -> Result<bool, rustycad_tkernel::standard::Failure> {
    for (other_index, other) in faces.iter().enumerate() {
        if other_index == index {
            continue;
        }
        let shape = other.clone().into_shape();
        if Explorer::new(&shape, ShapeEnum::Edge, ShapeEnum::Shape)?
            .any(|candidate| candidate.is_same(edge))
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn assert_free_edge_history(
    maker: &mut BRepOffsetMakeOffset,
    face: &Face,
    shared: &Shape,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let free_edge = Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )?
    .find(|edge| !edge.is_same(shared))
    .ok_or_else(|| {
        rustycad_tkernel::standard::Failure::with_message(
            "Standard_NoSuchObject",
            "box face fixture had no free edge",
        )
    })?;
    assert!(maker.offset_edges_from_shapes().has_image(&free_edge));
    let generated = maker.generated(&free_edge)?;
    assert_eq!(generated.len(), 1);
    assert_eq!(generated[0].shape_type()?, ShapeEnum::Edge);
    Ok(())
}

fn assert_shared_vertex_history(
    maker: &mut BRepOffsetMakeOffset,
    shared: &Shape,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let edge = Edge::unchecked(shared.clone());
    let (first, last) = topexp::vertices(&edge, false);
    for vertex in [first, last] {
        let generated = maker.generated(&vertex.into_shape())?;
        assert_eq!(generated.len(), 1);
        assert_eq!(generated[0].shape_type()?, ShapeEnum::Vertex);
    }
    Ok(())
}

fn adjacent_box_faces() -> Result<(Shape, [Face; 2], Shape), rustycad_tkernel::standard::Failure> {
    let mut box_maker = BRepPrimAPIMakeBox::from_sizes(4.0, 5.0, 6.0)?;
    let box_shape = box_maker.shape()?.clone();
    let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(&box_shape, ShapeEnum::Edge, ShapeEnum::Face, &mut ancestors)?;
    let (edge, faces) = first_two_face_edge(&ancestors)?;
    let input = shell_with_faces(&faces)?;
    Ok((input, faces, edge))
}

fn cylinder_cap_shell() -> Result<(Shape, [Face; 2], Shape), rustycad_tkernel::standard::Failure> {
    let mut cylinder_maker = BRepPrimAPIMakeCylinder::new(3.0, 6.0);
    let cylinder = cylinder_maker.shape()?.clone();
    let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(&cylinder, ShapeEnum::Edge, ShapeEnum::Face, &mut ancestors)?;
    let (edge, faces) = first_cylinder_plane_edge(&ancestors)?;
    let input = shell_with_faces(&faces)?;
    Ok((input, faces, edge))
}

fn cylinder_closed_shell() -> Result<(Shape, Vec<Face>), rustycad_tkernel::standard::Failure> {
    let mut cylinder_maker = BRepPrimAPIMakeCylinder::new(3.0, 6.0);
    let cylinder = cylinder_maker.shape()?.clone();
    let faces = Explorer::new(&cylinder, ShapeEnum::Face, ShapeEnum::Shape)?
        .map(Face::unchecked)
        .collect::<Vec<_>>();
    let builder = Builder;
    let mut shell = Shell::default();
    builder.make_shell(&mut shell);
    let mut input = shell.into_shape();
    for face in &faces {
        builder.add(&mut input, &face.clone().into_shape())?;
    }
    Ok((input, faces))
}

fn first_cylinder_plane_edge(
    ancestors: &IndexedDataMapOfShapeListOfShape,
) -> Result<(Shape, [Face; 2]), rustycad_tkernel::standard::Failure> {
    for index in 1..=ancestors.extent() {
        let parents = ancestors.find_from_index(index)?;
        if parents.len() != 2 {
            continue;
        }
        let faces = [
            Face::unchecked(parents.first().unwrap().clone()),
            Face::unchecked(parents.last().unwrap().clone()),
        ];
        if has_cylinder_and_plane(&faces)? {
            let ordered = order_cylinder_before_plane(faces)?;
            return Ok((ancestors.find_key(index)?.clone(), ordered));
        }
    }
    Err(rustycad_tkernel::standard::Failure::with_message(
        "Standard_NoSuchObject",
        "cylinder fixture had no edge between a cylinder and plane face",
    ))
}

fn has_cylinder_and_plane(faces: &[Face; 2]) -> Result<bool, rustycad_tkernel::standard::Failure> {
    let types = [
        BRepAdaptorSurface::with_face(&faces[0], false)?.get_type()?,
        BRepAdaptorSurface::with_face(&faces[1], false)?.get_type()?,
    ];
    Ok(types.contains(&SurfaceType::Cylinder) && types.contains(&SurfaceType::Plane))
}

fn order_cylinder_before_plane(
    faces: [Face; 2],
) -> Result<[Face; 2], rustycad_tkernel::standard::Failure> {
    if BRepAdaptorSurface::with_face(&faces[0], false)?.get_type()? == SurfaceType::Cylinder {
        return Ok(faces);
    }
    Ok([faces[1].clone(), faces[0].clone()])
}

fn first_two_face_edge(
    ancestors: &IndexedDataMapOfShapeListOfShape,
) -> Result<(Shape, [Face; 2]), rustycad_tkernel::standard::Failure> {
    for index in 1..=ancestors.extent() {
        let parents = ancestors.find_from_index(index)?;
        if parents.len() != 2 {
            continue;
        }
        return Ok((
            ancestors.find_key(index)?.clone(),
            [
                Face::unchecked(parents.first().unwrap().clone()),
                Face::unchecked(parents.last().unwrap().clone()),
            ],
        ));
    }
    Err(rustycad_tkernel::standard::Failure::with_message(
        "Standard_NoSuchObject",
        "box fixture had no edge with two face ancestors",
    ))
}

fn shell_with_faces(faces: &[Face; 2]) -> Result<Shape, rustycad_tkernel::standard::Failure> {
    let builder = Builder;
    let mut shell = Shell::default();
    builder.make_shell(&mut shell);
    let mut shell_shape = shell.into_shape();
    for face in faces {
        builder.add(&mut shell_shape, &face.clone().into_shape())?;
    }
    Ok(shell_shape)
}

fn count(shape: &Shape, kind: ShapeEnum) -> Result<usize, rustycad_tkernel::standard::Failure> {
    Ok(Explorer::new(shape, kind, ShapeEnum::Shape)?.count())
}

fn assert_native_area(shape: &Shape) -> Result<(), rustycad_tkernel::standard::Failure> {
    let mut properties = GProps::default();
    BRepGProp::surface_properties(shape, &mut properties, false, false)?;
    assert!((properties.mass() - 58.712_388_980_384_69).abs() < 1.0e-9);
    Ok(())
}

fn assert_surface_manifest(shape: &Shape) -> Result<(), rustycad_tkernel::standard::Failure> {
    let mut surfaces = Vec::new();
    for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        let face = Face::unchecked(face);
        let kind = BRepAdaptorSurface::with_face(&face, false)?.get_type()?;
        let mut properties = GProps::default();
        BRepGProp::surface_properties(&face.clone().into_shape(), &mut properties, false, false)?;
        surfaces.push((kind, properties.mass()));
    }
    assert_surface_kinds(&surfaces);
    assert_surface_areas(&mut surfaces);
    Ok(())
}

fn assert_surface_kinds(surfaces: &[(SurfaceType, f64)]) {
    assert_eq!(
        surfaces
            .iter()
            .filter(|(kind, _)| *kind == SurfaceType::Plane)
            .count(),
        2
    );
    assert_eq!(
        surfaces
            .iter()
            .filter(|(kind, _)| *kind == SurfaceType::Cylinder)
            .count(),
        1
    );
}

fn assert_surface_areas(surfaces: &mut [(SurfaceType, f64)]) {
    surfaces.sort_by(|first, second| first.1.total_cmp(&second.1));
    let expected = [4.712_388_980_384_69, 24.0, 30.0];
    for ((_, actual), expected) in surfaces.iter().zip(expected) {
        assert!((actual - expected).abs() < 1.0e-9);
    }
}

fn assert_mixed_support_manifest(shape: &Shape) -> Result<(), rustycad_tkernel::standard::Failure> {
    let mut surfaces = Vec::new();
    for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        let face = Face::unchecked(face);
        let kind = BRepAdaptorSurface::with_face(&face, false)?.get_type()?;
        let mut properties = GProps::default();
        BRepGProp::surface_properties(&face.clone().into_shape(), &mut properties, false, false)?;
        surfaces.push((kind, properties.mass()));
    }
    eprintln!("mixed support surfaces {surfaces:?}");
    assert_eq!(
        surfaces
            .iter()
            .filter(|(kind, _)| *kind == SurfaceType::Plane)
            .count(),
        1
    );
    assert_eq!(
        surfaces
            .iter()
            .filter(|(kind, _)| *kind == SurfaceType::Cylinder)
            .count(),
        1
    );
    assert_eq!(
        surfaces
            .iter()
            .filter(|(kind, _)| *kind == SurfaceType::Torus)
            .count(),
        1
    );
    surfaces.sort_by(|first, second| first.1.total_cmp(&second.1));
    let expected = [
        16.375_202_928_428_937,
        28.274_333_882_308_145,
        131.946_891_450_771_32,
    ];
    for ((_, actual), expected) in surfaces.iter().zip(expected) {
        assert!((actual - expected).abs() < 1.0e-8);
    }
    Ok(())
}
