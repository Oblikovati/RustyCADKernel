//! 1:1 translation of src/ModelingAlgorithms/TKTopAlgo/GTests/BRepLib_MakeWire_Test.cxx.
#![allow(non_snake_case)]

use rustycad_tkbrep::topods::Wire;
use rustycad_tktopalgo::brep_lib::BRepLibMakeWire;

#[test]
fn BRepLib_MakeWire_Test__OCC30708_2_InitializeWithNullWire() {
    let empty = Wire::default();

    // Should not throw exception when initializing with null wire
    // EXPECT_NO_THROW(BRepLib_MakeWire aWBuilder(empty));
    assert!(BRepLibMakeWire::from_wire(&empty).is_ok());
}
