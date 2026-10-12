//! Bounded worker completion guards the valid native self-alias against a recursive state lock.
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{graphic3d::*,null_driver::NullGraphicDriver};
use std::sync::mpsc;
use std::time::Duration;

#[test]
fn native_camera_self_alias_assignment_finishes_and_preserves_slot_and_counts() {
    let (sender,receiver)=mpsc::channel();
    let worker=std::thread::spawn(move ||sender.send(camera_alias_replay()).unwrap());
    let rows=receiver.recv_timeout(Duration::from_secs(3)).expect(
        "CView camera self-assignment completion=timeout; expected unlocked completion within 3 seconds",
    );
    worker.join().unwrap();
    assert_eq!(rows,include_str!("fixtures/cview-camera-self-assignment-native.txt"));
}
fn camera_alias_replay()->String {
    let driver=NullGraphicDriver::new_handle();
    let manager=Handle::new(Graphic3dStructureManager::new(&driver).unwrap());
    let view=driver.create_view(&manager).unwrap();
    let alias=view.camera_ref();
    let original=alias.clone_handle();
    let mut rows=vec![camera_alias_row("before",&*alias,&original,&*view.camera_ref())];
    view.set_camera(&*alias);
    rows.push(camera_alias_row("self",&*alias,&original,&*view.camera_ref()));
    view.set_camera(&original);
    rows.push(camera_alias_row("same-target",&*alias,&original,&*view.camera_ref()));
    rows.join("\n")+"\n"
}
fn camera_alias_row(
    phase:&str,
    alias:&dyn rustycad_tkernel::standard::HandleReference<Graphic3dCamera>,
    original:&Handle<Graphic3dCamera>,
    fresh:&dyn rustycad_tkernel::standard::HandleReference<Graphic3dCamera>,
)->String {
    let same_slot=(alias.reference_address()==fresh.reference_address())as u8;
    let same_target=(alias.object_address()==original.object_address())as u8;
    format!("{phase} {same_slot} {same_target} {}",original.ref_count())
}
