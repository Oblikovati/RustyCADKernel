// SPDX-License-Identifier: GPL-2.0-only
use rustycad_tkernel::{
    ncollection::DataMap, quantity::Color, standard::SharedState, tcollection::AsciiString,
};
use rustycad_tkservice::graphic3d::Graphic3dMaterialAspect;
#[derive(Default)]
pub(super) struct DrawerMaps {
    pub integers: SharedState<DataMap<i32, i32>>,
    pub doubles: SharedState<DataMap<i32, f64>>,
    pub booleans: SharedState<DataMap<i32, bool>>,
    pub colors: SharedState<DataMap<i32, Color>>,
    pub materials: SharedState<DataMap<i32, Graphic3dMaterialAspect>>,
    pub strings: SharedState<DataMap<i32, AsciiString>>,
}
pub(super) fn set_channel<T: Clone>(map: &SharedState<DataMap<i32, T>>, key: i32, value: &T) {
    let mut map = map.write();
    if map.is_bound(&key) {
        map.change_find(&key)
            .expect("bound drawer key")
            .clone_from(value);
        return;
    }
    map.bind_ref(&key, value);
}
pub(super) fn get_channel<T: Clone>(
    map: &SharedState<DataMap<i32, T>>,
    key: i32,
    output: &mut T,
) -> bool {
    let map = map.read();
    // Native miss does not default or otherwise touch the caller's output.
    map.find_into(&key, output)
}
pub(super) fn remove_channel<T>(map: &SharedState<DataMap<i32, T>>, key: i32) -> bool {
    let mut map = map.write();
    // UnBind has exactly the native miss result and leaves other typed maps unchanged.
    map.unbind(&key)
}
