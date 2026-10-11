extern crate rustycad_tkernel;
extern crate rustycad_tkservice;
extern crate rustycad_tkv3d;
#[path = "../../../crates/tkv3d/tests/support/view_input_buffer_fixture.rs"]
mod fixture;
fn send_sync<T: Send + Sync>() {}
fn main() {
    send_sync::<rustycad_tkv3d::ais::AisViewInputBuffer>();
    let events = fixture::seeded();
    let mut acknowledged = std::thread::spawn(move || {
        assert_eq!(
            fixture::snapshot(&events),
            fixture::native_values("pending")
        );
        let mut next = events.clone();
        next.reset();
        next
    })
    .join()
    .unwrap();
    assert_eq!(
        fixture::snapshot(&acknowledged),
        fixture::native_values("reset")
    );
    acknowledged.reset();
    assert_eq!(
        fixture::snapshot(&acknowledged),
        fixture::native_values("reset_twice")
    );
}
