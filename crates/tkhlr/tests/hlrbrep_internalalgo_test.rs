//! Direct translation of HLRBRep_InternalAlgo_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::Handle;
use rustycad_tkhlr::hlr_brep::InternalAlgo;

#[test]
fn HLRBRep_InternalAlgoTest__EmptyAlgorithmOperationsAreNoOps() {
    let anAlgorithm: Handle<InternalAlgo> = Handle::new(InternalAlgo::new().unwrap());
    let anAlgorithm = anAlgorithm.get().unwrap();

    assert_eq!(anAlgorithm.nb_shapes(), 0);
    assert!(anAlgorithm.update().is_ok());
    assert!(anAlgorithm.select().is_ok());
    assert!(anAlgorithm.show_all().is_ok());
    assert!(anAlgorithm.hide_all().is_ok());
    assert!(anAlgorithm.partial_hide().is_ok());
    assert!(anAlgorithm.hide().is_ok());
    assert!(anAlgorithm.data_structure().is_null());
}
