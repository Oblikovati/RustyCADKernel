//! Unit tests of the XSAlgo package.
use super::*;
use crate::interface::InterfaceStatic;
use crate::interface::test_norm::fake_entity;
use crate::transfer::{TransferFinder, TransferFinderProcess, TransferTransientProcess};
use crate::transfer_brep::{TransferBRep, TransferBRepShapeBinder};
use rustycad_tkbrep::topods::{Builder, Compound, Shape};
use rustycad_tkernel::{
    message::ProgressRange,
    standard::{Handle, StandardTransient},
    tcollection::AsciiString,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendMsgRegistrator,
    shape_process::{ShapeProcessOperationsFlags, ShapeProcessShapeMap},
};

fn compound() -> Shape {
    let mut c = Compound::default();
    Builder.make_compound(&mut c);
    c.into_shape()
}

fn value(map: &XSAlgoParameterMap, key: &str) -> String {
    map.seek(&AsciiString::new(key))
        .map(|v| String::from_utf8_lossy(v.as_bytes()).into_owned())
        .unwrap_or_default()
}

#[test]
fn parameters_from_shape_fix_parameters() {
    let mut map = XSAlgoParameterMap::default();
    XSAlgoShapeProcessor::set_parameter(
        "FixShape.FixFaceMode",
        &AsciiString::new("1"),
        false,
        &mut map,
    );
    XSAlgoShapeProcessor::fill_parameter_map(&DEShapeFixParameters::default(), false, &mut map);
    assert_eq!(map.extent(), 54);
    assert_eq!(value(&map, "FixShape.FixFaceMode"), "1", "not replaced");
    assert_eq!(value(&map, "FixShape.FixFreeShellMode"), "-1");
    assert_eq!(value(&map, "FixShape.CreateOpenSolidMode"), "0");
    assert_eq!(value(&map, "DetalizationLevel"), "7");
    assert_eq!(value(&map, "NonManifold"), "0");
    assert_eq!(
        value(&map, "FixShape.MaxTailAngle"),
        "0",
        "native quirk: a fix mode"
    );
    assert_eq!(value(&map, "FixShape.Tolerance3d"), "1e-06");
    let mut target = map.clone();
    let mut extra = XSAlgoParameterMap::default();
    extra.bind(
        AsciiString::new("FixShape.FixFaceMode"),
        AsciiString::new("5"),
    );
    extra.bind(AsciiString::new("Extra"), AsciiString::new("x"));
    XSAlgoShapeProcessor::set_shape_fix_parameters(
        &DEShapeFixParameters::default(),
        &extra,
        &mut target,
    );
    assert_eq!(
        (
            value(&target, "FixShape.FixFaceMode"),
            value(&target, "Extra")
        ),
        ("-1".to_owned(), "x".to_owned())
    );
    XSAlgoShapeProcessor::set_parameter_mode("M", DEShapeFixMode::Fix, true, &mut target);
    XSAlgoShapeProcessor::set_parameter_real("R", 2.5, true, &mut target);
    assert_eq!(
        (value(&target, "M"), value(&target, "R")),
        ("1".to_owned(), "2.5".to_owned())
    );
}

#[test]
fn actors_take_shape_fix_parameters() {
    use crate::transfer::{TransferActorOfTransientProcess, TransferActorOfTransientProcessOps};
    let actor = TransferActorOfTransientProcess::new();
    actor.shape_processing().set_shape_fix_parameters_from(
        &DEShapeFixParameters::default(),
        &XSAlgoParameterMap::default(),
    );
    assert_eq!(actor.shape_processing().shape_fix_parameters().extent(), 54);
}

#[test]
fn processor_runs_and_merges() {
    let mut processor = XSAlgoShapeProcessor::from_shape_fix(&DEShapeFixParameters::default());
    let progress = ProgressRange::default();
    assert!(
        processor
            .process_shape(
                &Shape::default(),
                ShapeProcessOperationsFlags::default(),
                &progress
            )
            .unwrap()
            .is_null()
    );
    let shape = compound();
    let result = processor
        .process_shape(&shape, ShapeProcessOperationsFlags::default(), &progress)
        .unwrap();
    assert!(result == shape && processor.get_context().is_some());
    let context = processor.get_context();
    let live = context.get().unwrap();
    assert_eq!(live.get_detalisation(), ShapeEnum::Vertex);
    let tp = TransferTransientProcess::new(10);
    processor.merge_transfer_info(&tp, 1).unwrap();
    processor
        .merge_transfer_info_finder(&TransferFinderProcess::new(10))
        .unwrap();
    assert!(
        XSAlgoShapeProcessor::read_processing_data(
            &AsciiString::new("no.such.static"),
            &AsciiString::new("no.such.scope")
        )
        .is_ok()
    );
}

#[test]
fn merge_modified_shapes() {
    let (old, new) = (compound(), compound());
    let mut map = ShapeProcessShapeMap::default();
    map.bind(old.clone(), new.clone());
    let none = Handle::<ShapeExtendMsgRegistrator>::default();
    let tp = TransferTransientProcess::new(10);
    let ent = fake_entity("a", &[]);
    TransferBRep::set_shape_result(&tp, &ent, &old).unwrap();
    XSAlgoShapeProcessor::merge_shape_transfer_info(&tp, &map, 0, &none).unwrap();
    assert!(TransferBRep::shape_result_of(&tp, &ent).unwrap() == new);
    let fp = TransferFinderProcess::new(10);
    XSAlgoShapeProcessor::merge_shape_transfer_info_finder(&fp, &map, &none).unwrap();
    let bound = TransferBRep::result_from_shape(&fp, &old);
    assert!(
        bound
            .cast::<TransferBRepShapeBinder>()
            .get()
            .is_some_and(|b| b.result() == new)
    );
    XSAlgoShapeProcessor::merge_shape_transfer_info_finder(&fp, &map, &none).unwrap();
    assert!(
        bound.next_result().is_some(),
        "a second merge chains the result binder"
    );
    assert!(
        XSAlgoShapeProcessor::merge_shape_transfer_info(&Handle::default(), &map, 1, &none)
            .is_err()
    );
    assert!(
        XSAlgoShapeProcessor::merge_shape_transfer_info(
            &Handle::default(),
            &ShapeProcessShapeMap::default(),
            1,
            &none
        )
        .is_ok()
    );
    let finder: Handle<dyn TransferFinder> = TransferBRep::shape_mapper(&fp, &old).unwrap().cast();
    assert!(fp.map_index(&finder) > 0);
}

#[test]
fn check_pcurve_on_a_planar_face() {
    use rustycad_tkbrep::topexp::Explorer;
    use rustycad_tkbrep::topods;
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;
    let face = BRepBuilderAPIMakeFace::from_plane_bounds(Default::default(), (0.0, 1.0, 0.0, 2.0))
        .unwrap()
        .face()
        .unwrap();
    let explorer = Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    let edge = topods::edge(explorer.current()).unwrap();
    assert!(XSAlgoShapeProcessor::check_pcurve(&edge, &face, 1e-7, false).unwrap());
    // the ends of the p-curve on the surface match the 3D ends: a large precision passes too
    assert!(XSAlgoShapeProcessor::check_pcurve(&edge, &face, 1.0, false).unwrap());
    let other = BRepBuilderAPIMakeFace::from_plane_bounds(Default::default(), (5.0, 6.0, 5.0, 6.0))
        .unwrap()
        .face()
        .unwrap();
    // BRep_Tool projects the edge on another plane: the p-curve exists and is kept
    assert!(
        XSAlgoShapeProcessor::check_pcurve(&edge, &other, 1e-7, false).unwrap(),
        "a projected p-curve on another plane"
    );
}

#[test]
fn algo_container_and_statics() {
    XSAlgo::init().unwrap();
    XSAlgo::init().unwrap();
    let container = XSAlgo::algo_container();
    assert!(container.is_some());
    assert_eq!(
        InterfaceStatic::c_val("xstep.cascade.unit").as_bytes(),
        b"MM"
    );
    assert_eq!(InterfaceStatic::i_val("read.stdsameparameter.mode"), 0);
    container.prepare_for_transfer();
    assert_eq!(xs_algo_units().get_cascade_length_unit_default(), 1.0);
    let shape = compound();
    let args = XSAlgoProcessShapeArgs {
        prec: 1e-7,
        max_tol: 1.0,
        rscfile: "no.such.rsc",
        pseq: "read.no.seq",
        non_manifold: false,
        detailing_level: ShapeEnum::Vertex,
    };
    let mut info: Handle<StandardTransient> = Handle::default();
    let progress = ProgressRange::default();
    let result = container
        .process_shape(&shape, &args, &mut info, &progress)
        .unwrap();
    assert!(
        result == shape && info.is_some(),
        "the default ShapeFix keeps a valid empty compound"
    );
    let tp = TransferTransientProcess::new(10);
    container.merge_transfer_info(&tp, &info, 1).unwrap();
    container
        .merge_transfer_info_finder(&TransferFinderProcess::new(10), &info)
        .unwrap();
    container
        .merge_transfer_info(&tp, &Handle::default(), 1)
        .unwrap();
    let write_args = XSAlgoProcessShapeArgs {
        pseq: "write.no.seq",
        ..args
    };
    let mut write_info: Handle<StandardTransient> = Handle::default();
    assert!(
        container
            .process_shape(&shape, &write_args, &mut write_info, &progress)
            .unwrap()
            == shape
    );
    assert!(
        container
            .process_shape(&Shape::default(), &write_args, &mut write_info, &progress)
            .unwrap()
            .is_null()
    );
    XSAlgo::set_algo_container(&Handle::new(XSAlgoAlgoContainer::new()).cast());
}
