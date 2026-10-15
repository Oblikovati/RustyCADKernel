//! Independent closed-form checks of the complete intersection mode of
//! BRepOffset_MakeOffset (BRepOffset_MakeOffset_1: the splits of the offset faces of planar
//! solids, with the invalid splits removed and rebuilt).
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkoffset::brep_offset::{BRepOffsetMakeOffset, BRepOffsetMode, MakeOffsetOptions};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakePrism;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use rustycad_tktopalgo::brep_lib::{BRepLibMakeFace, BRepLibMakePolygon};

/// The prism of height `h` along Z of the closed polygon of the XY points.
fn polygon_prism(points: &[(f64, f64)], h: f64) -> Result<Shape, Failure> {
    let mut poly = BRepLibMakePolygon::new();
    for &(x, y) in points {
        poly.add_point(Point3::new(x, y, 0.))?;
    }
    poly.close()?;
    let face = BRepLibMakeFace::from_wire(&poly.wire()?, true)?.face()?;
    Ok(
        BRepPrimAPIMakePrism::new(&face, Vector3::new(0., 0., h), false, true)?
            .shape()?
            .clone(),
    )
}

/// The offset shape of the complete intersection mode (intersection join).
fn complete_offset(shape: &Shape, offset: f64) -> Result<Shape, Failure> {
    let mut maker = BRepOffsetMakeOffset::new();
    let options = MakeOffsetOptions {
        mode: BRepOffsetMode::Skin,
        intersection: true,
        self_inter: false,
        join: JoinType::Intersection,
        thickening: false,
        remove_int_edges: false,
    };
    maker.initialize(shape, offset, 1e-7, options);
    maker.make_offset_shape(&ProgressRange::default())?;
    assert!(maker.is_done());
    Ok(maker.shape().clone())
}

/// The volume and the number of solids of a shape.
fn volume_and_solids(shape: &Shape) -> Result<(f64, usize), Failure> {
    let mut props = GProps::new();
    BRepGProp::volume_properties(shape, &mut props, false, false, false)?;
    let solids = Explorer::new(shape, ShapeEnum::Solid, ShapeEnum::Shape)?.count();
    Ok((props.mass(), solids))
}

/// The U prism of width 5, depth 4 and height 2 whose arms leave a gap 1 wide: the
/// outward offset by 0.6 moves both walls of the gap 0.6 into it, so the gap is closed and
/// the result is the offset box 6.2 x 5.2 x 3.2 = 103.168.
#[test]
fn closing_the_gap_of_a_u_prism_gives_the_offset_box() -> Result<(), Failure> {
    let u = polygon_prism(
        &[
            (0., 0.),
            (5., 0.),
            (5., 4.),
            (3., 4.),
            (3., 1.2),
            (2., 1.2),
            (2., 4.),
            (0., 4.),
        ],
        2.,
    )?;
    let (volume, solids) = volume_and_solids(&complete_offset(&u, 0.6)?)?;
    assert_eq!(solids, 1);
    assert!((volume - 103.168).abs() < 1e-6 * 103.168, "volume {volume}");
    Ok(())
}

/// The inward offset of the same U prism by 0.6 removes its base (1.2 deep) and leaves the
/// two arms 2 - 1.2 = 0.8 wide, 4 - 1.2 - 0.6 + 0.6 = 2.8 deep and 0.8 high, two separate
/// solids of volume 0.8 x 2.8 x 0.8 = 1.792 each.
#[test]
fn shrinking_a_u_prism_splits_it_into_its_arms() -> Result<(), Failure> {
    let u = polygon_prism(
        &[
            (0., 0.),
            (5., 0.),
            (5., 4.),
            (3., 4.),
            (3., 1.2),
            (2., 1.2),
            (2., 4.),
            (0., 4.),
        ],
        2.,
    )?;
    let (volume, solids) = volume_and_solids(&complete_offset(&u, -0.6)?)?;
    assert_eq!(solids, 2);
    assert!(
        (volume - 2. * 1.792).abs() < 1e-6 * 3.584,
        "volume {volume}"
    );
    Ok(())
}

/// The inward offset by 0.5 of the L prism of side 4, arms 1.6 wide and height 2 keeps
/// the L of arms 0.6 wide: (3 x 0.6 + 0.6 x 2.4) x 1 = 3.24.
#[test]
fn shrinking_an_l_prism_keeps_the_l() -> Result<(), Failure> {
    let l = polygon_prism(
        &[
            (0., 0.),
            (4., 0.),
            (4., 1.6),
            (1.6, 1.6),
            (1.6, 4.),
            (0., 4.),
        ],
        2.,
    )?;
    let (volume, solids) = volume_and_solids(&complete_offset(&l, -0.5)?)?;
    assert_eq!(solids, 1);
    assert!((volume - 3.24).abs() < 1e-6 * 3.24, "volume {volume}");
    Ok(())
}
