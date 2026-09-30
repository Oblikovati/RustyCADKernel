//! Direct translation of the TKBRep-only declarations of BRepTools_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. BRepPrimAPI_MakeBox is the box fake of
//! `support` (TKPrim is not ported).
//!
//! The other eight declarations call BRepGProp, BRepMesh, BRepAlgoAPI, BRepCheck,
//! BRepBndLib, BRepBuilderAPI_NurbsConvert, BRepBuilderAPI_Transform or OSD_Parallel on
//! BRepGProp results: those toolkits (TKTopAlgo, TKMesh, TKBO) are not ported, and the
//! native oracle closure cannot build this source (it needs TKBO and TKMesh). The
//! suite is not mapped in the ledger until every declaration is
//! (docs/tkbrep-progress.md, "Dependency gap").
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{self, BRepWriteOptions};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{
    self, CompSolid, Compound, Edge, Face, Shape, Shell, Solid, Vertex, Wire,
};
use rustycad_tkbrep::toptools::{MapOfShape, format_version};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg3d::top_abs::ShapeEnum;
use support::make_box;

/// The text half of `RoundTripShape`: `BRepTools::Write` with the options into a
/// string stream, then `BRepTools::Read` from its start.
fn RoundTripShape(theShape: &Shape, theWithTriangles: bool, theVersion: i32) -> Shape {
    let mut aStream = IoStream::from_bytes(Vec::new());
    let anOptions = BRepWriteOptions {
        with_triangles: theWithTriangles,
        with_normals: false,
        version: theVersion,
    };
    brep_tools::write_with_options(theShape, &mut aStream, anOptions, &ProgressRange::default())
        .unwrap();
    assert!(aStream.good());
    assert!(aStream.tellp() > 0);

    aStream.seekg(0);
    let mut aRestoredShape = Shape::default();
    brep_tools::read(
        &mut aRestoredShape,
        &mut aStream,
        &BRepBuilder::default(),
        &ProgressRange::default(),
    )
    .unwrap();
    assert!(!aRestoredShape.is_null());
    aRestoredShape
}

fn NbSubShapes(theShape: &Shape, theType: ShapeEnum) -> usize {
    Explorer::new(theShape, theType, ShapeEnum::Shape)
        .unwrap()
        .count()
}

fn MakeEmptyShape(theType: ShapeEnum) -> Shape {
    let aBuilder = BRepBuilder::default();
    match theType {
        ShapeEnum::Vertex => {
            let mut aShape = Vertex::default();
            aBuilder.make_vertex(&mut aShape);
            aShape.into_shape()
        }
        ShapeEnum::Edge => {
            let mut aShape = Edge::default();
            aBuilder.make_edge(&mut aShape).unwrap();
            aShape.into_shape()
        }
        ShapeEnum::Wire => {
            let mut aShape = Wire::default();
            aBuilder.make_wire(&mut aShape);
            aShape.into_shape()
        }
        ShapeEnum::Face => {
            let mut aShape = Face::default();
            aBuilder.make_face(&mut aShape);
            aShape.into_shape()
        }
        _ => MakeEmptyContainer(&aBuilder, theType),
    }
}

/// The container half of `MakeEmptyShape`.
fn MakeEmptyContainer(theBuilder: &BRepBuilder, theType: ShapeEnum) -> Shape {
    match theType {
        ShapeEnum::Shell => {
            let mut aShape = Shell::default();
            theBuilder.make_shell(&mut aShape);
            aShape.into_shape()
        }
        ShapeEnum::Solid => {
            let mut aShape = Solid::default();
            theBuilder.make_solid(&mut aShape);
            aShape.into_shape()
        }
        ShapeEnum::CompSolid => {
            let mut aShape = CompSolid::default();
            theBuilder.make_comp_solid(&mut aShape);
            aShape.into_shape()
        }
        ShapeEnum::Compound => {
            let mut aShape = Compound::default();
            theBuilder.make_compound(&mut aShape);
            aShape.into_shape()
        }
        _ => Shape::default(),
    }
}

// bugs/demo/bug25344_4: a BRep shape remains valid after stream save/restore.
#[test]
fn BRepTools_Test__DemoBug_25344_4_SaveRestoreShapeType() {
    let aSource = make_box(1.0, 1.0, 1.0).into_shape();
    let aRestored = RoundTripShape(&aSource, true, format_version::VERSION_2);
    assert!(!aRestored.is_null());
    assert_eq!(
        aRestored.shape_type().unwrap(),
        aSource.shape_type().unwrap()
    );
    assert_eq!(NbSubShapes(&aRestored, ShapeEnum::Solid), 1);
    assert_eq!(NbSubShapes(&aRestored, ShapeEnum::Face), 6);
}

// bugs/moddata_1/bug20616: every empty BRep topology type survives DRAW-style storage.
#[test]
fn BRepTools_Test__ModDataBug_20616_EmptyShapeSaveRestore() {
    let aTypes = [
        ShapeEnum::Vertex,
        ShapeEnum::Edge,
        ShapeEnum::Wire,
        ShapeEnum::Face,
        ShapeEnum::Shell,
        ShapeEnum::Solid,
        ShapeEnum::CompSolid,
        ShapeEnum::Compound,
    ];

    for aType in aTypes {
        let aSource = MakeEmptyShape(aType);
        assert!(!aSource.is_null());

        let mut aStream = IoStream::from_bytes(Vec::new());
        brep_tools::write(&aSource, &mut aStream, &ProgressRange::default()).unwrap();
        assert!(aStream.good());
        assert!(aStream.tellp() > 0);

        aStream.seekg(0);
        let mut aRestored = Shape::default();
        let aBuilder = BRepBuilder::default();
        brep_tools::read(
            &mut aRestored,
            &mut aStream,
            &aBuilder,
            &ProgressRange::default(),
        )
        .unwrap();
        assert!(!aRestored.is_null());
        assert_eq!(aRestored.shape_type().unwrap(), aType);
    }
}

/// The tolerance of the current explored sub-shape of `theType`.
fn ToleranceOf(theShape: &Shape, theType: ShapeEnum) -> f64 {
    match theType {
        ShapeEnum::Face => BRepTool::face_tolerance(&topods::face(theShape).unwrap()).unwrap(),
        ShapeEnum::Edge => BRepTool::edge_tolerance(&topods::edge(theShape).unwrap()).unwrap(),
        ShapeEnum::Vertex => {
            BRepTool::vertex_tolerance(&topods::vertex(theShape).unwrap()).unwrap()
        }
        _ => 0.0,
    }
}

// bugs/demo/bug23130: maxtolerance reports all box subshape counts and their
// default tolerances consistently on every supported platform.
#[test]
fn BRepTools_Test__DemoBug_23130_BoxMaxTolerance() {
    let aBox = make_box(100.0, 100.0, 100.0).into_shape();
    let aTypes = [ShapeEnum::Face, ShapeEnum::Edge, ShapeEnum::Vertex];
    let anExpectedCounts = [6, 12, 8];

    for aTypeIndex in 0..3 {
        let mut aShapeMap = MapOfShape::new(1);
        let mut aMinTolerance = 1.0e100_f64;
        let mut aMaxTolerance = -1.0_f64;
        for aCurrent in Explorer::new(&aBox, aTypes[aTypeIndex], ShapeEnum::Shape).unwrap() {
            let aTolerance = ToleranceOf(&aCurrent, aTypes[aTypeIndex]);
            aShapeMap.add(aCurrent);
            aMinTolerance = aMinTolerance.min(aTolerance);
            aMaxTolerance = aMaxTolerance.max(aTolerance);
        }

        assert_eq!(aShapeMap.extent(), anExpectedCounts[aTypeIndex]);
        assert!((aMinTolerance - 1.0e-7).abs() <= 1.0e-7);
        assert!((aMaxTolerance - 1.0e-7).abs() <= 1.0e-7);
    }
}
