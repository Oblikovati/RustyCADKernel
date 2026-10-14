// SPDX-License-Identifier: GPL-2.0-only
use super::{MeshVSDrawer, storage::*};
use rustycad_tkernel::{quantity::Color, tcollection::AsciiString};
use rustycad_tkservice::graphic3d::Graphic3dMaterialAspect;
impl MeshVSDrawer {
    /// Store a typed appearance value; `drawer.set_integer(7, value)`.
    pub fn set_integer(&self, key: i32, value: i32) {
        set_channel(&self.maps.integers, key, &value);
    }
    /// Leave output untouched on a missing key; `drawer.get_integer(7,&mut output)`.
    pub fn get_integer(&self, key: i32, output: &mut i32) -> bool {
        get_channel(&self.maps.integers, key, output)
    }
    /// Remove only this typed key; `drawer.remove_integer(7)`.
    pub fn remove_integer(&self, key: i32) -> bool {
        remove_channel(&self.maps.integers, key)
    }
    /// Store a typed appearance value; `drawer.set_double(7, value)`.
    pub fn set_double(&self, key: i32, value: f64) {
        set_channel(&self.maps.doubles, key, &value);
    }
    /// Leave output untouched on a missing key; `drawer.get_double(7,&mut output)`.
    pub fn get_double(&self, key: i32, output: &mut f64) -> bool {
        get_channel(&self.maps.doubles, key, output)
    }
    /// Remove only this typed key; `drawer.remove_double(7)`.
    pub fn remove_double(&self, key: i32) -> bool {
        remove_channel(&self.maps.doubles, key)
    }
    /// Store a typed appearance value; `drawer.set_boolean(7, value)`.
    pub fn set_boolean(&self, key: i32, value: bool) {
        set_channel(&self.maps.booleans, key, &value);
    }
    /// Leave output untouched on a missing key; `drawer.get_boolean(7,&mut output)`.
    pub fn get_boolean(&self, key: i32, output: &mut bool) -> bool {
        get_channel(&self.maps.booleans, key, output)
    }
    /// Remove only this typed key; `drawer.remove_boolean(7)`.
    pub fn remove_boolean(&self, key: i32) -> bool {
        remove_channel(&self.maps.booleans, key)
    }
    /// Store a typed appearance value; `drawer.set_color(7, value)`.
    pub fn set_color(&self, key: i32, value: &Color) {
        set_channel(&self.maps.colors, key, value);
    }
    /// Leave output untouched on a missing key; `drawer.get_color(7,&mut output)`.
    pub fn get_color(&self, key: i32, output: &mut Color) -> bool {
        get_channel(&self.maps.colors, key, output)
    }
    /// Remove only this typed key; `drawer.remove_color(7)`.
    pub fn remove_color(&self, key: i32) -> bool {
        remove_channel(&self.maps.colors, key)
    }
    /// Store a typed appearance value; `drawer.set_material(7, value)`.
    pub fn set_material(&self, key: i32, value: &Graphic3dMaterialAspect) {
        set_channel(&self.maps.materials, key, value);
    }
    /// Leave output untouched on a missing key; `drawer.get_material(7,&mut output)`.
    pub fn get_material(&self, key: i32, output: &mut Graphic3dMaterialAspect) -> bool {
        get_channel(&self.maps.materials, key, output)
    }
    /// Remove only this typed key; `drawer.remove_material(7)`.
    pub fn remove_material(&self, key: i32) -> bool {
        remove_channel(&self.maps.materials, key)
    }
    /// Store a typed appearance value; `drawer.set_ascii_string(7, value)`.
    pub fn set_ascii_string(&self, key: i32, value: &AsciiString) {
        set_channel(&self.maps.strings, key, value);
    }
    /// Leave output untouched on a missing key; `drawer.get_ascii_string(7,&mut output)`.
    pub fn get_ascii_string(&self, key: i32, output: &mut AsciiString) -> bool {
        get_channel(&self.maps.strings, key, output)
    }
    /// Remove only this typed key; `drawer.remove_ascii_string(7)`.
    pub fn remove_ascii_string(&self, key: i32) -> bool {
        remove_channel(&self.maps.strings, key)
    }
}
