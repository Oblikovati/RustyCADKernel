// SPDX-License-Identifier: GPL-2.0-only
//! MeshVS_Drawer at OCCT 3d097a0: six independent typed appearance maps.
mod channels;
mod copying;
mod source;
mod storage;
pub use copying::MeshVSDrawerBaseType;
use rustycad_tkernel::standard::StandardTransient;
pub(crate) use source::registered_drawer;
pub use source::{MeshVSDrawerSource, meshvs_drawer_assign};
use storage::DrawerMaps;
/// Genuine native drawer state; `let drawer=MeshVSDrawer::default(); drawer.set_integer(7,42)`.
#[derive(Default)]
pub struct MeshVSDrawer {
    transient: StandardTransient,
    maps: DrawerMaps,
}
rustycad_tkernel::impl_transient!(
    MeshVSDrawer, "MeshVS_Drawer", StandardTransient, transient,
    interfaces: [dyn MeshVSDrawerSource]
);
