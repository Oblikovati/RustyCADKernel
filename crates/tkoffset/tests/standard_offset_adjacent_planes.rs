//! Native differential for two connected planar supports with one shared convex edge.
use rustycad_tkbrep::{
    brep_adaptor::BRepAdaptorSurface,
    topexp::{self, Explorer},
    topods::{Builder, Edge, Face, Shape, Shell},
    toptools::IndexedDataMapOfShapeListOfShape,
};
use rustycad_tkg3d::{adaptor3d::Adaptor3dSurface, top_abs::ShapeEnum};
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::geom_abs::{JoinType, SurfaceType};
use rustycad_tkoffset::brep_offset::{BRepOffsetError, BRepOffsetMakeOffset, BRepOffsetMode};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::{
    brep_builder_api::BRepBuilderAPIMakeShape, brep_check::BRepCheckAnalyzer, brep_gprop::BRepGProp,
};

#[test]
fn adjacent_planes_gain_the_native_rounded_shared_edge_image()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let (input, faces, shared_edge) = adjacent_box_faces()?;
    let mut maker = maker_for(&input);

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

fn maker_for(input: &Shape) -> BRepOffsetMakeOffset {
    let mut maker = BRepOffsetMakeOffset::new();
    maker.initialize(
        input,
        0.5,
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
