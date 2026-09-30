//! Direct translation of TopoDS_Iterator_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! BRepBuilderAPI_MakeVertex, BRepBuilderAPI_MakeEdge and BRepPrimAPI_MakeBox are the
//! BRep fakes of `support` (TKTopAlgo and TKPrim are not ported).
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Builder, Compound, Shape, ShapeIterator, Wire};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Point3, Transform3};
use rustycad_tkmath::toploc::Location;
use support::{make_box, make_edge, make_vertex};

fn gp_Pnt(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

/// `TopoDS_Iterator anIt(S)`: the native defaults cumOri and cumLoc are true.
fn iterator(shape: &Shape) -> ShapeIterator {
    ShapeIterator::new(shape, true, true)
}

/// A compound made with `TopoDS_Builder::MakeCompound`.
fn new_compound(aBuilder: &Builder) -> Compound {
    let mut aCompound = Compound::default();
    aBuilder.make_compound(&mut aCompound);
    aCompound
}

//=================================================================================================
// Test OCC30708_1: Initialize with null shape
//=================================================================================================

#[test]
fn TopoDS_Iterator_Test__OCC30708_1_InitializeWithNullShape() {
    let mut it = ShapeIterator::default();
    let empty = Shape::default();

    // Should not throw exception when initializing with null shape
    it.initialize(&empty, true, true);

    // More() should return false on null shape
    assert!(!it.more());
}

//=================================================================================================
// Test iterator on empty compound
//=================================================================================================

#[test]
fn TopoDS_Iterator_Test__EmptyCompound() {
    let aBuilder = Builder;
    let aCompound = new_compound(&aBuilder);

    let anIt = iterator(&aCompound);
    assert!(!anIt.more(), "Empty compound should have no children");
}

//=================================================================================================
// Test iterator on compound with vertices
//=================================================================================================

#[test]
fn TopoDS_Iterator_Test__CompoundWithVertices() {
    let aBuilder = Builder;
    let mut aCompound = new_compound(&aBuilder);

    // Add 5 vertices
    for i in 0..5 {
        let aV = make_vertex(gp_Pnt(f64::from(i), 0.0, 0.0));
        aBuilder.add(&mut aCompound, &aV).unwrap();
    }

    // Iterate and count
    let mut aCount = 0;
    let mut anIt = iterator(&aCompound);
    while anIt.more() {
        assert_eq!(
            anIt.value().unwrap().shape_type().unwrap(),
            ShapeEnum::Vertex
        );
        aCount += 1;
        anIt.next();
    }

    assert_eq!(aCount, 5, "Should iterate over 5 vertices");
}

//=================================================================================================
// Test iterator on wire with edges
//=================================================================================================

#[test]
fn TopoDS_Iterator_Test__WireWithEdges() {
    let aBuilder = Builder;
    let mut aWire = Wire::default();
    aBuilder.make_wire(&mut aWire);

    // Create a rectangular wire (4 edges)
    let aE1 = make_edge(gp_Pnt(0.0, 0.0, 0.0), gp_Pnt(1.0, 0.0, 0.0));
    let aE2 = make_edge(gp_Pnt(1.0, 0.0, 0.0), gp_Pnt(1.0, 1.0, 0.0));
    let aE3 = make_edge(gp_Pnt(1.0, 1.0, 0.0), gp_Pnt(0.0, 1.0, 0.0));
    let aE4 = make_edge(gp_Pnt(0.0, 1.0, 0.0), gp_Pnt(0.0, 0.0, 0.0));

    aBuilder.add(&mut aWire, &aE1).unwrap();
    aBuilder.add(&mut aWire, &aE2).unwrap();
    aBuilder.add(&mut aWire, &aE3).unwrap();
    aBuilder.add(&mut aWire, &aE4).unwrap();

    // Iterate and verify
    let mut aCount = 0;
    let mut anIt = iterator(&aWire);
    while anIt.more() {
        assert_eq!(anIt.value().unwrap().shape_type().unwrap(), ShapeEnum::Edge);
        aCount += 1;
        anIt.next();
    }

    assert_eq!(aCount, 4, "Wire should have 4 edges");
}

//=================================================================================================
// Test iterator on edge with vertices
//=================================================================================================

#[test]
fn TopoDS_Iterator_Test__EdgeWithVertices() {
    let anEdge = make_edge(gp_Pnt(0.0, 0.0, 0.0), gp_Pnt(1.0, 1.0, 1.0));

    let mut aCount = 0;
    let mut anIt = iterator(&anEdge);
    while anIt.more() {
        assert_eq!(
            anIt.value().unwrap().shape_type().unwrap(),
            ShapeEnum::Vertex
        );
        aCount += 1;
        anIt.next();
    }

    assert_eq!(aCount, 2, "Edge should have 2 vertices");
}

//=================================================================================================
// Test iterator on vertex (no children)
//=================================================================================================

#[test]
fn TopoDS_Iterator_Test__VertexNoChildren() {
    let aVertex = make_vertex(gp_Pnt(0.0, 0.0, 0.0));

    let anIt = iterator(&aVertex);
    assert!(!anIt.more(), "Vertex should have no children");
}

//=================================================================================================
// Test iterator cumulative orientation
//=================================================================================================

#[test]
fn TopoDS_Iterator_Test__CumulativeOrientation() {
    let aBuilder = Builder;
    let mut aCompound = new_compound(&aBuilder);

    let aV = make_vertex(gp_Pnt(0.0, 0.0, 0.0));
    aBuilder.add(&mut aCompound, &aV).unwrap();

    // Reverse the compound
    aCompound.reverse();

    // With cumOri=true (default), child orientation should be composed
    let anIt1 = ShapeIterator::new(&aCompound, true, true);
    assert!(anIt1.more());
    let anOri1: Orientation = anIt1.value().unwrap().orientation();

    // With cumOri=false, child orientation should be original
    let anIt2 = ShapeIterator::new(&aCompound, false, true);
    assert!(anIt2.more());
    let anOri2: Orientation = anIt2.value().unwrap().orientation();

    // Orientations should differ when compound is reversed
    assert_ne!(
        anOri1, anOri2,
        "Cumulative orientation should affect result"
    );
}

//=================================================================================================
// Test iterator re-initialization
//=================================================================================================

#[test]
fn TopoDS_Iterator_Test__ReInitialization() {
    let aBuilder = Builder;
    let mut aCompound1 = new_compound(&aBuilder);
    let mut aCompound2 = new_compound(&aBuilder);

    // Add different number of vertices
    for i in 0..3 {
        aBuilder
            .add(
                &mut aCompound1,
                &make_vertex(gp_Pnt(f64::from(i), 0.0, 0.0)),
            )
            .unwrap();
    }
    for i in 0..5 {
        aBuilder
            .add(
                &mut aCompound2,
                &make_vertex(gp_Pnt(f64::from(i), 1.0, 0.0)),
            )
            .unwrap();
    }

    let mut anIt = iterator(&aCompound1);

    // Count first compound
    let mut aCount1 = 0;
    while anIt.more() {
        aCount1 += 1;
        anIt.next();
    }
    assert_eq!(aCount1, 3);

    // Re-initialize with second compound
    anIt.initialize(&aCompound2, true, true);

    // Count second compound
    let mut aCount2 = 0;
    while anIt.more() {
        aCount2 += 1;
        anIt.next();
    }
    assert_eq!(aCount2, 5);
}

//=================================================================================================
// Test iterator on box solid (shell -> faces)
//=================================================================================================

#[test]
fn TopoDS_Iterator_Test__BoxSolidIteration() {
    // Create a box
    let aBox: Shape = make_box(1.0, 2.0, 3.0).into_shape();

    // Iterate over solid's children (should be 1 shell)
    let mut aShellCount = 0;
    let mut anIt = iterator(&aBox);
    while anIt.more() {
        assert_eq!(
            anIt.value().unwrap().shape_type().unwrap(),
            ShapeEnum::Shell
        );
        aShellCount += 1;

        // Iterate over shell's children (should be 6 faces)
        let mut aFaceCount = 0;
        let mut anIt2 = iterator(anIt.value().unwrap());
        while anIt2.more() {
            assert_eq!(
                anIt2.value().unwrap().shape_type().unwrap(),
                ShapeEnum::Face
            );
            aFaceCount += 1;
            anIt2.next();
        }
        assert_eq!(aFaceCount, 6, "Box shell should have 6 faces");
        anIt.next();
    }

    assert_eq!(aShellCount, 1, "Box solid should have 1 shell");
}

//=================================================================================================
// Test iterator with many children (performance/stress test)
//=================================================================================================

#[test]
fn TopoDS_Iterator_Test__ManyChildren() {
    let aBuilder = Builder;
    let mut aCompound = new_compound(&aBuilder);

    let aNbVertices = 1000;

    // Add many vertices
    for i in 0..aNbVertices {
        aBuilder
            .add(&mut aCompound, &make_vertex(gp_Pnt(f64::from(i), 0.0, 0.0)))
            .unwrap();
    }

    // Iterate and count
    let mut aCount = 0;
    let mut anIt = iterator(&aCompound);
    while anIt.more() {
        aCount += 1;
        anIt.next();
    }

    assert_eq!(aCount, aNbVertices, "Should iterate over all vertices");
}

//=================================================================================================
// Test default constructor followed by Initialize
//=================================================================================================

#[test]
fn TopoDS_Iterator_Test__DefaultConstructorThenInitialize() {
    let aBuilder = Builder;
    let mut aCompound = new_compound(&aBuilder);
    aBuilder
        .add(&mut aCompound, &make_vertex(gp_Pnt(0.0, 0.0, 0.0)))
        .unwrap();

    // Default construct, then initialize
    let mut anIt = ShapeIterator::default();
    assert!(
        !anIt.more(),
        "Default constructed iterator should have More()=false"
    );

    anIt.initialize(&aCompound, true, true);
    assert!(anIt.more(), "After Initialize, More() should be true");

    assert_eq!(
        anIt.value().unwrap().shape_type().unwrap(),
        ShapeEnum::Vertex
    );
}

//=================================================================================================
// Test Value() throws when not More() (only in debug builds with exceptions enabled)
//=================================================================================================

#[test]
fn TopoDS_Iterator_Test__ValueThrowsWhenNotMore() {
    let aBuilder = Builder;
    let aCompound = new_compound(&aBuilder);

    let anIt = iterator(&aCompound);
    assert!(!anIt.more());

    // Value() should throw Standard_NoSuchObject when !More()
    let aFailure = anIt.value().unwrap_err();
    assert_eq!(aFailure.exception_type(), "Standard_NoSuchObject");
}

//=================================================================================================
// Test iterator with cumulative location
//=================================================================================================

#[test]
fn TopoDS_Iterator_Test__CumulativeLocation() {
    let aBuilder = Builder;
    let mut aCompound = new_compound(&aBuilder);

    let aV = make_vertex(gp_Pnt(0.0, 0.0, 0.0));
    aBuilder.add(&mut aCompound, &aV).unwrap();

    // Apply a translation to the compound
    let aTrsf = Transform3::translation([10.0, 20.0, 30.0]);
    let aLoc = Location::new(aTrsf);
    aCompound.set_location(&aLoc, false).unwrap();

    // With cumLoc=true (default), child should have composed location
    let anIt1 = ShapeIterator::new(&aCompound, true, true);
    assert!(anIt1.more());
    let aChildLoc1 = anIt1.value().unwrap().location().clone();

    // With cumLoc=false, child should have original location
    let anIt2 = ShapeIterator::new(&aCompound, true, false);
    assert!(anIt2.more());
    let aChildLoc2 = anIt2.value().unwrap().location().clone();

    // Locations should differ
    assert_ne!(
        aChildLoc1.is_identity(),
        aChildLoc2.is_identity(),
        "Cumulative location should affect child location"
    );
}

//=================================================================================================
// Test iterator on nested compounds
//=================================================================================================

#[test]
fn TopoDS_Iterator_Test__NestedCompounds() {
    let aBuilder = Builder;
    let mut aOuter = new_compound(&aBuilder);
    let mut aInner1 = new_compound(&aBuilder);
    let mut aInner2 = new_compound(&aBuilder);

    // Add vertices to inner compounds
    aBuilder
        .add(&mut aInner1, &make_vertex(gp_Pnt(0.0, 0.0, 0.0)))
        .unwrap();
    aBuilder
        .add(&mut aInner1, &make_vertex(gp_Pnt(1.0, 0.0, 0.0)))
        .unwrap();
    aBuilder
        .add(&mut aInner2, &make_vertex(gp_Pnt(2.0, 0.0, 0.0)))
        .unwrap();

    // Add inner compounds to outer
    aBuilder.add(&mut aOuter, &aInner1).unwrap();
    aBuilder.add(&mut aOuter, &aInner2).unwrap();

    // Outer should have 2 children (both compounds)
    let mut aOuterCount = 0;
    let mut anIt = iterator(&aOuter);
    while anIt.more() {
        assert_eq!(
            anIt.value().unwrap().shape_type().unwrap(),
            ShapeEnum::Compound
        );
        aOuterCount += 1;
        anIt.next();
    }
    assert_eq!(aOuterCount, 2);

    // First inner should have 2 children
    let anIt = iterator(&aOuter);
    let mut aInner1Count = 0;
    let mut anIt2 = iterator(anIt.value().unwrap());
    while anIt2.more() {
        aInner1Count += 1;
        anIt2.next();
    }
    assert_eq!(aInner1Count, 2);
}

//=================================================================================================
// Test iterator on face with wires (from box)
//=================================================================================================

#[test]
fn TopoDS_Iterator_Test__FaceWithWires() {
    let aBox: Shape = make_box(1.0, 1.0, 1.0).into_shape();

    // Get first face
    let anExp = Explorer::new(&aBox, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    assert!(anExp.more());
    let aFace = topods::face(anExp.current()).unwrap();

    // Iterate over face's children (should be wires)
    let mut aWireCount = 0;
    let mut anIt = iterator(&aFace);
    while anIt.more() {
        assert_eq!(anIt.value().unwrap().shape_type().unwrap(), ShapeEnum::Wire);
        aWireCount += 1;
        anIt.next();
    }

    assert!(aWireCount >= 1, "Face should have at least 1 wire");
}

//=================================================================================================
// Test iterator preserves shape identity
//=================================================================================================

#[test]
fn TopoDS_Iterator_Test__ShapeIdentity() {
    let aBuilder = Builder;
    let mut aCompound = new_compound(&aBuilder);

    let aV1 = make_vertex(gp_Pnt(1.0, 0.0, 0.0));
    let aV2 = make_vertex(gp_Pnt(2.0, 0.0, 0.0));

    aBuilder.add(&mut aCompound, &aV1).unwrap();
    aBuilder.add(&mut aCompound, &aV2).unwrap();

    // Iterate and check that TShape pointers match
    let mut anIt = iterator(&aCompound);
    let aValue = anIt.value().unwrap().tshape().clone();
    assert!(aValue == *aV1.tshape() || aValue == *aV2.tshape());
    anIt.next();
    let aValue = anIt.value().unwrap().tshape().clone();
    assert!(aValue == *aV1.tshape() || aValue == *aV2.tshape());
}

//=================================================================================================
// Test iterator on shell with faces
//=================================================================================================

#[test]
fn TopoDS_Iterator_Test__ShellWithFaces() {
    let aBox: Shape = make_box(1.0, 1.0, 1.0).into_shape();

    // Get shell from box
    let anExp = Explorer::new(&aBox, ShapeEnum::Shell, ShapeEnum::Shape).unwrap();
    assert!(anExp.more());
    let aShell = topods::shell(anExp.current()).unwrap();

    // Iterate over shell's faces
    let mut aFaceCount = 0;
    let mut anIt = iterator(&aShell);
    while anIt.more() {
        assert_eq!(anIt.value().unwrap().shape_type().unwrap(), ShapeEnum::Face);
        aFaceCount += 1;
        anIt.next();
    }

    assert_eq!(aFaceCount, 6, "Box shell should have 6 faces");
}
