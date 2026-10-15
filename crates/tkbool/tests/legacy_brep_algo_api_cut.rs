//! Direct translation of src/ModelingAlgorithms/TKBool/GTests/BRepAlgoAPI_Cut_Test.cxx (3d097a0).
#![allow(non_snake_case)]

use rustycad_tkbo::brep_algo_api::{BRepAlgoAPICommon, BRepAlgoAPICut};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape, Solid};
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPICopy, BRepBuilderAPIMakeShape};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_gprop::BRepGProp;

/// `EXPECT_NEAR(actual, expected, tolerance)`.
fn expect_near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} differs from {expected} by more than {tolerance}"
    );
}

/// `BRepPrimAPI_MakeBox(P, dx, dy, dz).Solid()`.
fn MakeBoxSolid(p: Point3, dx: f64, dy: f64, dz: f64) -> Solid {
    BRepPrimAPIMakeBox::at_point(p, dx, dy, dz)
        .unwrap()
        .solid()
        .unwrap()
}

/// `BRepGProp::VolumeProperties(S, props)` mass.
fn VolumeOf(shape: &Shape) -> f64 {
    let mut props = GProps::default();
    BRepGProp::volume_properties(shape, &mut props, false, false, false).unwrap();
    props.mass()
}

/// The solids of a shape, with the last one (`aCutSolid` of the native loops).
fn Solids(shape: &Shape) -> (usize, Shape) {
    let (mut count, mut last) = (0, Shape::default());
    for solid in Explorer::new(shape, ShapeEnum::Solid, ShapeEnum::Shape).unwrap() {
        let solid = topods::solid(&solid).unwrap();
        if !solid.is_null() {
            last = solid.into_shape();
            count += 1;
        }
    }
    (count, last)
}

/// The grid cells of the extended bounding box of `solid` with their volumes.
fn GridCells(solid: &Shape, the_mesh_delta: f64) -> (Vec<Shape>, Vec<f64>) {
    let a_delt = 5.0 * precision::confusion();
    let mut a_bnd_box = BoundingBox3::default();
    BRepBndLib::add(solid, &mut a_bnd_box, true).unwrap();
    let [
        mut a_xmin,
        mut a_ymin,
        mut a_zmin,
        mut a_xmax,
        mut a_ymax,
        mut a_zmax,
    ] = a_bnd_box.get().unwrap();
    a_xmin -= a_delt;
    a_ymin -= a_delt;
    a_zmin -= a_delt;
    a_xmax += a_delt;
    a_ymax += a_delt;
    a_zmax += a_delt;
    let count = |extent: f64| (((extent / the_mesh_delta) as i32).max(0)).max(1);
    let (a_nx, a_ny, a_nz) = (
        count(a_xmax - a_xmin),
        count(a_ymax - a_ymin),
        count(a_zmax - a_zmin),
    );
    let a_step_x = (a_xmax - a_xmin) / f64::from(a_nx);
    let a_step_y = (a_ymax - a_ymin) / f64::from(a_ny);
    let a_step_z = (a_zmax - a_zmin) / f64::from(a_nz);
    let (mut subvols, mut vols) = (Vec::new(), Vec::new());
    for i in 0..a_nx {
        for j in 0..a_ny {
            for k in 0..a_nz {
                let p = Point3::new(
                    a_xmin + f64::from(i) * a_step_x,
                    a_ymin + f64::from(j) * a_step_y,
                    a_zmin + f64::from(k) * a_step_z,
                );
                let cell = MakeBoxSolid(p, a_step_x, a_step_y, a_step_z).into_shape();
                vols.push(VolumeOf(&cell));
                subvols.push(cell);
            }
        }
    }
    (subvols, vols)
}

/// `runHollowBoxMeshTest(theMeshDelta)`: the ratio of the volumes of the commons of the
/// hollow box with the grid cells to its volume; -1 on failure.
fn runHollowBoxMeshTest(the_mesh_delta: f64) -> f64 {
    let a_full_solid = MakeBoxSolid(Point3::new(0., 0., 0.), 30.0, 30.0, 30.0).into_shape();
    let an_inner_solid = MakeBoxSolid(Point3::new(10., 10., 10.), 10.0, 10.0, 10.0).into_shape();
    let a_cut = BRepAlgoAPICut::from_shapes(&a_full_solid, &an_inner_solid).unwrap();
    if !a_cut.is_done() {
        return -1.0;
    }
    let (a_nb_solids, a_cut_solid) = Solids(a_cut.shape());
    if a_nb_solids != 1 {
        return -1.0;
    }
    let an_original_volume = VolumeOf(&a_cut_solid);
    if an_original_volume <= 0.0 {
        return -1.0;
    }
    let (a_subvols, a_subvol_vols) = GridCells(&a_cut_solid, the_mesh_delta);
    let mut an_accumulated = 0.0;
    for (cell, cell_volume) in a_subvols.iter().zip(&a_subvol_vols) {
        let a_copy_solid = BRepBuilderAPICopy::with_shape(&a_cut_solid, true, false)
            .unwrap()
            .shape()
            .unwrap()
            .clone();
        let a_common = BRepAlgoAPICommon::from_shapes(&a_copy_solid, cell).unwrap();
        if !a_common.is_done() {
            continue;
        }
        let (a_nb_common, a_found_solid) = Solids(a_common.shape());
        if a_nb_common == 1 {
            let a_vol = VolumeOf(&a_found_solid);
            if a_vol > 0.0 && a_vol <= *cell_volume {
                an_accumulated += a_vol;
            }
        }
    }
    an_accumulated / an_original_volume
}

/// OCC817: BRepAlgoAPI_Common result accuracy with hollow box using mesh_delta=10.
#[test]
fn hollow_box_volume_accuracy_delta10() {
    let a_ratio = runHollowBoxMeshTest(10.0);
    assert!(a_ratio >= 0.0, "Cut or Common operation failed");
    expect_near(a_ratio, 1.0, 0.001);
}

/// OCC817: BRepAlgoAPI_Common result accuracy with hollow box using mesh_delta=15.
#[test]
fn hollow_box_volume_accuracy_delta15() {
    let a_ratio = runHollowBoxMeshTest(15.0);
    assert!(a_ratio >= 0.0, "Cut or Common operation failed");
    expect_near(a_ratio, 1.0, 0.001);
}

/// OCC817: BRepAlgoAPI_Common result accuracy with hollow box using mesh_delta=30.
#[test]
fn hollow_box_volume_accuracy_delta30() {
    let a_ratio = runHollowBoxMeshTest(30.0);
    assert!(a_ratio >= 0.0, "Cut or Common operation failed");
    expect_near(a_ratio, 1.0, 0.001);
}

/// OCC817: hollow box has correct surface area and valid shape.
#[test]
fn hollow_box_surface_area_and_validity() {
    let a_full_solid = MakeBoxSolid(Point3::new(0., 0., 0.), 30.0, 30.0, 30.0).into_shape();
    let an_inner_solid = MakeBoxSolid(Point3::new(10., 10., 10.), 10.0, 10.0, 10.0).into_shape();
    let a_cut = BRepAlgoAPICut::from_shapes(&a_full_solid, &an_inner_solid).unwrap();
    assert!(a_cut.is_done());
    let mut a_cut_solid = Shape::default();
    for solid in Explorer::new(a_cut.shape(), ShapeEnum::Solid, ShapeEnum::Shape).unwrap() {
        a_cut_solid = topods::solid(&solid).unwrap().into_shape();
    }
    assert!(!a_cut_solid.is_null());
    let mut a_sprops = GProps::default();
    BRepGProp::surface_properties(&a_cut_solid, &mut a_sprops, false, false).unwrap();
    expect_near(a_sprops.mass(), 6000.0, 6.0);
    let a_checker = BRepCheckAnalyzer::new(&a_cut_solid, true, false, false).unwrap();
    assert!(a_checker.is_valid().unwrap());
}
