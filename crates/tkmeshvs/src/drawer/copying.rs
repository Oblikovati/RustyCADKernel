// SPDX-License-Identifier: GPL-2.0-only
//! The implicit native copy constructor copies fields in declaration order and resets object count.
use super::{MeshVSDrawer, storage::DrawerMaps};
use rustycad_tkernel::standard::{SharedState, StandardTransient};
impl Clone for MeshVSDrawer {
    fn clone(&self) -> Self {
        Self {
            transient: StandardTransient::default(),
            maps: DrawerMaps {
                integers: SharedState::new(self.maps.integers.read().clone()),
                booleans: SharedState::new(self.maps.booleans.read().clone()),
                doubles: SharedState::new(self.maps.doubles.read().clone()),
                colors: SharedState::new(self.maps.colors.read().clone()),
                materials: SharedState::new(self.maps.materials.read().clone()),
                strings: SharedState::new(self.maps.strings.read().clone()),
            },
        }
    }
}
impl MeshVSDrawer {
    /// Native class spelling without allocating a descriptor; `MeshVSDrawer::get_type_name()`.
    pub const fn get_type_name() -> &'static str {
        // Keep upstream names for API and provenance, while the port itself is Martensite.
        "MeshVS_Drawer"
    }
}
/// Native `MeshVS_Drawer::base_type`; aliases refer to the actual inherited counter base.
pub type MeshVSDrawerBaseType = StandardTransient;
