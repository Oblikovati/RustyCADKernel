//! Real meshed solids prove closure routing, own-aspect checks and NaN alpha comparisons.
use super::{colored_shape_grouping_tests::dispatch_group_row, *};
use crate::prs3d::{Prs3dDrawer, Prs3dShadingAspect};
use rustycad_tkbrep::{topexp::Explorer, topods::Shape};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmesh::brep_mesh::BRepMeshIncrementalMesh;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkservice::graphic3d::*;
use std::fmt::Write;
#[test]
fn actual_meshed_shell_closure_replays_hidden_front_back_and_nan_policy() {
    let mut rows = String::new();
    for scenario in 0..9 {
        closed_dispatch_row(&mut rows, scenario);
    }
    assert_eq!(
        rows,
        include_str!("../../tests/fixtures/colored-shape-compute/native-closed-dispatch.txt")
    );
}
fn closed_dispatch_row(rows: &mut String, scenario: usize) {
    let mut builder = BRepPrimAPIMakeBox::from_sizes(2., 3., 4.).unwrap();
    let shape = builder.solid().unwrap().into_shape();
    if scenario != 0 {
        BRepMeshIncrementalMesh::with_shape(&shape, 0.1, false, 0.5, false).unwrap();
    }
    let parent = Handle::new(Prs3dDrawer::new());
    let base = Handle::new(AisColoredDrawer::new(&parent));
    let custom = Handle::new(AisColoredDrawer::new(&parent));
    let styles = closed_dispatch_styles(&shape, &custom, scenario);
    let opened: AisColoredOpenedCompounds = std::array::from_fn(|_| Default::default());
    let closed = AisColoredDrawerCompounds::default();
    let overridden = ais_colored_shape_dispatch_colors(
        &base,
        &shape,
        &styles,
        ShapeEnum::Compound,
        false,
        &opened,
        &closed,
    )
    .unwrap();
    write!(rows, "closed {scenario} {}", i32::from(overridden)).unwrap();
    dispatch_group_row(rows, &opened[4], &base, &custom);
    dispatch_group_row(rows, &closed, &base, &custom);
    rows.push('\n');
}
fn closed_dispatch_styles(
    shape: &Shape,
    custom: &Handle<AisColoredDrawer>,
    scenario: usize,
) -> AisShapeDrawerMap {
    let styles = AisShapeDrawerMap::default();
    if scenario >= 2 {
        let faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
        styles.bind(faces.current(), custom).unwrap();
    }
    if scenario == 2 {
        custom.set_hidden(true);
    }
    if scenario >= 3 {
        configure_closed_dispatch_alpha(custom, scenario);
    }
    styles
}
fn configure_closed_dispatch_alpha(custom: &AisColoredDrawer, scenario: usize) {
    custom.set_shading_aspect(Handle::new(Prs3dShadingAspect::new().unwrap()));
    let aspect = custom.shading_aspect().aspect();
    let mode = match scenario {
        3 => Graphic3dAlphaMode::Opaque,
        4 => Graphic3dAlphaMode::Blend,
        8 => Graphic3dAlphaMode::Mask,
        _ => Graphic3dAlphaMode::BlendAuto,
    };
    aspect.set_alpha_mode(mode, 0.5);
    let mut front = aspect.front_material();
    let mut back = aspect.back_material();
    front
        .set_alpha(if scenario <= 4 {
            0.5
        } else if scenario == 7 {
            f32::NAN
        } else {
            1.
        })
        .unwrap();
    back.set_alpha(0.5).unwrap();
    aspect.set_front_material(front);
    aspect.set_back_material(back);
    aspect.set_distinguish(scenario == 6);
}
