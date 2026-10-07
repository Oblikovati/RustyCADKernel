//! Direct translation of HLRBRep_Data_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::Handle;
use rustycad_tkhlr::hlr_brep::Data;

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
