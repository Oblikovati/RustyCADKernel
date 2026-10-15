//! Native differential for two connected planar supports with one shared convex edge.
use rustycad_tkbrep::{
    brep_adaptor::BRepAdaptorSurface,
    topexp::Explorer,
    topods::{Builder, Face, Shape, Shell},
};
use rustycad_tkg3d::{adaptor3d::Adaptor3dSurface, top_abs::ShapeEnum};
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::geom_abs::{JoinType, SurfaceType};
use rustycad_tkoffset::brep_offset::{
    BRepOffsetError, BRepOffsetMakeOffset, BRepOffsetMode, MakeOffsetOptions,
};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeCylinder;
use rustycad_tktopalgo::{
    brep_builder_api::BRepBuilderAPIMakeShape, brep_check::BRepCheckAnalyzer, brep_gprop::BRepGProp,
};

#[test]
fn cylindrical_side_with_both_caps_builds_two_native_toroidal_joins()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let (input, faces) = cylinder_closed_shell()?;
    let mut maker = maker_for_offset(&input, 0.5);

    maker.make_offset_shape(&rustycad_tkernel::message::ProgressRange::default())?;

    assert!(maker.is_done());
    assert_eq!(maker.error(), BRepOffsetError::NoError);
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
        MakeOffsetOptions {
            mode: BRepOffsetMode::Skin,
            intersection: false,
            self_inter: false,
            join: JoinType::Arc,
            thickening: false,
            remove_int_edges: false,
        },
    );
    maker
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
            let generated = maker
                .generated(&candidate)?
                .iter()
                .cloned()
                .collect::<Vec<_>>();
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

fn count(shape: &Shape, kind: ShapeEnum) -> Result<usize, rustycad_tkernel::standard::Failure> {
    Ok(Explorer::new(shape, kind, ShapeEnum::Shape)?.count())
}
