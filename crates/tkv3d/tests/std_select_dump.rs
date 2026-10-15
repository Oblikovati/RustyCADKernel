//! StdSelect_BRepOwner::DumpJson and StdSelect_Shape::DumpJson against the text native OCCT
//! 3d097a0 prints for a null shape (captured from the native classes at depths -1, 1 and 0).
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkv3d::prs_mgr::PrsMgrPresentableObject;
use rustycad_tkv3d::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkv3d::std_select::{StdSelectBRepOwner, StdSelectBRepOwnerMembers, StdSelectShape};

fn owner_dump(depth: i32) -> String {
    let owner = StdSelectBRepOwner::with_shape(&Shape::default(), 5, false);
    owner.set_hilight_mode(3);
    let mut stream = DumpStream::default();
    owner.dump_json(&mut stream, depth).unwrap();
    String::from_utf8(stream.bytes().to_vec()).unwrap()
}

#[test]
fn brep_owner_dump_matches_native_text() {
    assert_eq!(
        owner_dump(-1),
        "\"className\": \"StdSelect_BRepOwner\", \"Shape\": {\"className\": \"TopoDS_Shape\", \
         \"Location\": {\"className\": \"TopLoc_Location\", \"Transformation\": {\"Location\": \
         [0, 0, 0], \"Matrix\": [1, 0, 0, 0, 1, 0, 0, 0, 1], \"shape\": 0, \"scale\": 1}, \
         \"IsIdentity\": 1}, \"Orient\": 3}, \"CurMode\": 3"
    );
    assert_eq!(
        owner_dump(1),
        "\"className\": \"StdSelect_BRepOwner\", \"Shape\": {\"className\": \"TopoDS_Shape\", \
         \"Orient\": 3}, \"CurMode\": 3"
    );
    assert_eq!(
        owner_dump(0),
        "\"className\": \"StdSelect_BRepOwner\", \"CurMode\": 3"
    );
}

#[test]
fn shape_dump_matches_native_text() {
    let shape = StdSelectShape::new(&Shape::default(), &Handle::default());
    let mut stream = DumpStream::default();
    shape.dump_json(&mut stream, 1).unwrap();
    assert_eq!(
        String::from_utf8(stream.bytes().to_vec()).unwrap(),
        "\"className\": \"StdSelect_Shape\", \"className\": \"PrsMgr_PresentableObject\", \
         \"Parent\": \"\", \"TypeOfPresentation3d\": 0, \"CurrentFacingModel\": 0, \"OwnWidth\": 0, \
         \"hasOwnColor\": 0, \"hasOwnMaterial\": 0, \"InfiniteState\": 0, \"IsMutable\": 0, \
         \"HasOwnPresentations\": 1, \"ToPropagateVisualState\": 1, \"sh\": {\"className\": \
         \"TopoDS_Shape\", \"Orient\": 3}"
    );
}
