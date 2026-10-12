//! Light-node mutation preserves native handle identity and iterator links.
use rustycad_tkernel::{
    ncollection::AllocatorList,
    standard::{Handle, raise_native_failure},
};
use rustycad_tkservice::graphic3d::Graphic3dCLight;
pub(super) fn remove_light_node(
    chain: &mut AllocatorList<Handle<Graphic3dCLight>>,
    identity: *const (),
) {
    let mut cursor = chain.cursor();
    while cursor.more() {
        if cursor
            .value(chain)
            .is_some_and(|stored| stored.object_address() == identity)
        {
            raise_native_failure(chain.remove(&mut cursor));
            return;
        }
        cursor.next(chain);
    }
}

pub(super) fn contains_light_node(
    chain: &AllocatorList<Handle<Graphic3dCLight>>,
    identity: *const (),
) -> bool {
    chain
        .iter()
        .any(|stored| stored.object_address() == identity)
}
