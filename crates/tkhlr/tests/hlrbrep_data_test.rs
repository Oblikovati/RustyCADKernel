//! Direct translation of HLRBRep_Data_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Compound, Face, Shape};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkhlr::hlr_algo::Projector;
use rustycad_tkhlr::hlr_brep::{Algo, Data, HLRToShape, TypeOfResultingEdge};
use rustycad_tkmath::gp::{Axis2, Direction3, Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

struct HLRResultCounts {
    VisibleEdges: i32,
    HiddenEdges: i32,
}

fn makeRectangularFace(theZ: f64, theUMin: f64, theUMax: f64, theVMin: f64, theVMax: f64) -> Face {
    let plane = Plane3::new(
        Point3::new(0.0, 0.0, theZ),
        Direction3::new(0., 0., 1.).unwrap(),
    )
    .unwrap();
    BRepBuilderAPIMakeFace::from_plane_bounds(plane, (theUMin, theUMax, theVMin, theVMax))
        .unwrap()
        .face()
        .unwrap()
}

const TYPES: [TypeOfResultingEdge; 5] = [
    TypeOfResultingEdge::IsoLine,
    TypeOfResultingEdge::OutLine,
    TypeOfResultingEdge::Rg1Line,
    TypeOfResultingEdge::RgNLine,
    TypeOfResultingEdge::Sharp,
];

fn countResultEdges(theAlgorithm: &Handle<Algo>, theVisible: bool) -> i32 {
    let aResultBuilder = HLRToShape::new(theAlgorithm);
    let mut aNbEdges = 0;
    // aType from HLRBRep_IsoLine to HLRBRep_Sharp.
    for aType in TYPES {
        let aResult = aResultBuilder
            .compound_of_edges(aType, theVisible, false)
            .unwrap();
        aNbEdges += Explorer::new(&aResult, ShapeEnum::Edge, ShapeEnum::Shape)
            .unwrap()
            .count() as i32;
    }
    aNbEdges
}

fn performExactHLR(theShape: &Shape) -> Handle<Algo> {
    let anAlgorithm = Handle::new(Algo::new().unwrap());
    let a = anAlgorithm.get().unwrap();
    a.set_projector(
        &Projector::with_frame(
            Axis2::new(
                Point3::new(0., 0., 0.),
                Direction3::new(0., 0., 1.).unwrap(),
            )
            .unwrap(),
        )
        .unwrap(),
    );
    a.add(theShape, 0);
    a.update().unwrap();
    a.hide().unwrap();
    anAlgorithm
}

fn performExactHLRFaces(theFirstFace: &Face, theSecondFace: &Face) -> HLRResultCounts {
    let mut aCompound = Compound::default();
    let aBuilder = BRepBuilder::default();
    aBuilder.make_compound(&mut aCompound);
    let mut aCompound = Shape::from(aCompound);
    aBuilder
        .add(&mut aCompound, &Shape::from(theFirstFace.clone()))
        .unwrap();
    aBuilder
        .add(&mut aCompound, &Shape::from(theSecondFace.clone()))
        .unwrap();

    let anAlgorithm = performExactHLR(&aCompound);

    HLRResultCounts {
        VisibleEdges: countResultEdges(&anAlgorithm, true),
        HiddenEdges: countResultEdges(&anAlgorithm, false),
    }
}

#[test]
fn HLRBRep_DataTest__ZeroSizedStorageCanBeDestroyedRepeatedly() {
    let aData: Handle<Data> = Handle::new(Data::new(0, 0, 0).unwrap());
    let aData = aData.get().unwrap();
    assert_eq!(aData.nb_vertices(), 0);
    assert_eq!(aData.nb_edges(), 0);
    assert_eq!(aData.nb_faces(), 0);

    aData.destroy();
    aData.destroy();
}

#[test]
fn HLRBRep_DataTest__IntersectionCacheIsStableAcrossEdgeOrdering() {
    let aFrontFace = makeRectangularFace(2.0, -4.0, 4.0, -3.0, 3.0);
    let aBackFace = makeRectangularFace(0.0, -1.0, 7.0, -5.0, 1.0);

    let aForwardOrder = performExactHLRFaces(&aFrontFace, &aBackFace);
    let aReverseOrder = performExactHLRFaces(&aBackFace, &aFrontFace);

    assert_eq!(aForwardOrder.VisibleEdges, 8);
    assert_eq!(aForwardOrder.HiddenEdges, 2);
    assert_eq!(aReverseOrder.VisibleEdges, aForwardOrder.VisibleEdges);
    assert_eq!(aReverseOrder.HiddenEdges, aForwardOrder.HiddenEdges);
}

#[test]
fn HLRBRep_DataTest__RejectionBitsCrossWordBoundary() {
    let mut aFaces: Vec<Face> = Vec::with_capacity(9);
    let aBuilder = BRepBuilder::default();
    let (mut aForwardCompound, mut aReverseCompound) = (Compound::default(), Compound::default());
    aBuilder.make_compound(&mut aForwardCompound);
    aBuilder.make_compound(&mut aReverseCompound);
    let mut aForwardCompound = Shape::from(aForwardCompound);
    let mut aReverseCompound = Shape::from(aReverseCompound);

    for anIndex in 0..9 {
        let anOffset = anIndex as f64 * 0.25;
        aFaces.push(makeRectangularFace(
            anIndex as f64,
            -4.0 + anOffset,
            4.0 + anOffset,
            -3.0 + anOffset,
            3.0 + anOffset,
        ));
        aBuilder
            .add(&mut aForwardCompound, &Shape::from(aFaces[anIndex].clone()))
            .unwrap();
    }
    for anIndex in (1..=aFaces.len()).rev() {
        aBuilder
            .add(
                &mut aReverseCompound,
                &Shape::from(aFaces[anIndex - 1].clone()),
            )
            .unwrap();
    }

    let aForwardAlgorithm = performExactHLR(&aForwardCompound);
    let aReverseAlgorithm = performExactHLR(&aReverseCompound);
    assert!(!aForwardAlgorithm.get().unwrap().data_structure().is_null());
    assert!(!aReverseAlgorithm.get().unwrap().data_structure().is_null());
    assert_eq!(
        aForwardAlgorithm
            .get()
            .unwrap()
            .data_structure()
            .get()
            .unwrap()
            .nb_edges(),
        36
    );
    assert_eq!(
        aReverseAlgorithm
            .get()
            .unwrap()
            .data_structure()
            .get()
            .unwrap()
            .nb_edges(),
        36
    );
    let aForwardCounts = HLRResultCounts {
        VisibleEdges: countResultEdges(&aForwardAlgorithm, true),
        HiddenEdges: countResultEdges(&aForwardAlgorithm, false),
    };
    let aReverseCounts = HLRResultCounts {
        VisibleEdges: countResultEdges(&aReverseAlgorithm, true),
        HiddenEdges: countResultEdges(&aReverseAlgorithm, false),
    };
    assert_eq!(aForwardCounts.VisibleEdges, 36);
    assert_eq!(aForwardCounts.HiddenEdges, 16);
    assert_eq!(aReverseCounts.VisibleEdges, aForwardCounts.VisibleEdges);
    assert_eq!(aReverseCounts.HiddenEdges, aForwardCounts.HiddenEdges);
}
