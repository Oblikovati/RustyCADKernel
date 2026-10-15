// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_ElementalColorPrsBuilder` at OCCT 3d097a0: elements colored by one color (faces,
//! volumes and links) or by a front and back color pair (faces), grouped by color.
//!
//! Native quirk kept: `Build` moves every pair of equal colors into the one-color map
//! (a const method editing its maps through a cast).
mod build;
mod build_pairs;

use crate::data_source::{MeshVSDataSource, MeshVSPackedMap};
use crate::flags::MESHVS_DMF_ELEMENTAL_COLOR_DATA_PRS;
use crate::mesh::MeshVSMeshSource;
use crate::prs_builder::{MeshVSPrsBuilder, MeshVSPrsBuilderBase, MeshVSPrsBuilderMembers};
use crate::selection_policy::MESHVS_BP_ELEM_COLOR;
use crate::two_colors::{MeshVSTwoColors, bind_two_colors, extract_colors};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;

/// The color maps of `MeshVS_ElementalColorPrsBuilder`.
#[derive(Default)]
pub struct MeshVSElementalColorMaps {
    /// `myElemColorMap1`.
    pub colors1: DataMap<i32, Color>,
    /// `myElemColorMap2`.
    pub colors2: DataMap<i32, MeshVSTwoColors>,
}

/// The builder of element colors; `MeshVSElementalColorPrsBuilder::with_defaults(&mesh)`.
pub struct MeshVSElementalColorPrsBuilder {
    transient: StandardTransient,
    base: MeshVSPrsBuilderBase,
    /// The color maps; `builder.maps.read().colors1.extent()`.
    pub maps: SharedState<MeshVSElementalColorMaps>,
}
rustycad_tkernel::impl_transient!(MeshVSElementalColorPrsBuilder, "MeshVS_ElementalColorPrsBuilder", dyn MeshVSPrsBuilder, transient,
    interfaces: [dyn MeshVSPrsBuilder]);

fn set_or_bind<V>(map: &mut DataMap<i32, V>, id: i32, value: V) {
    match map.change_find(&id) {
        Ok(slot) => *slot = value,
        Err(_) => {
            map.bind(id, value);
        }
    }
}

impl MeshVSElementalColorPrsBuilder {
    /// `MeshVS_ElementalColorPrsBuilder(Parent, Flags, DS, Id, Priority)`: excluding.
    pub fn new(
        parent: &Handle<dyn MeshVSMeshSource>,
        flags: i32,
        data_source: &Handle<dyn MeshVSDataSource>,
        id: i32,
        priority: i32,
    ) -> Self {
        let builder = Self {
            transient: StandardTransient::default(),
            base: MeshVSPrsBuilderBase::new(parent, flags, data_source, id, priority),
            maps: SharedState::default(),
        };
        builder.set_excluding(true);
        builder
    }
    /// The native default arguments: `MeshVS_DMF_ElementalColorDataPrs`, no data source, a
    /// free id and `MeshVS_BP_ElemColor`. Example: `MeshVSElementalColorPrsBuilder::with_defaults(&mesh)`.
    pub fn with_defaults(parent: &Handle<dyn MeshVSMeshSource>) -> Self {
        Self::new(
            parent,
            MESHVS_DMF_ELEMENTAL_COLOR_DATA_PRS,
            &Handle::default(),
            -1,
            MESHVS_BP_ELEM_COLOR,
        )
    }
    /// `GetColors1`: a copy; `builder.get_colors1().extent()`.
    pub fn get_colors1(&self) -> DataMap<i32, Color> {
        let mut copy = DataMap::default();
        copy.assign(&self.maps.read().colors1);
        copy
    }
    /// `SetColors1`; `builder.set_colors1(&map)`.
    pub fn set_colors1(&self, map: &DataMap<i32, Color>) {
        self.maps.write().colors1.assign(map);
    }
    /// `HasColors1`; `builder.has_colors1()`.
    pub fn has_colors1(&self) -> bool {
        self.maps.read().colors1.extent() > 0
    }
    /// `GetColor1`: the color of an element, the output kept without.
    pub fn get_color1(&self, id: i32, color: &mut Color) -> bool {
        match self.maps.read().colors1.seek(&id) {
            Some(found) => {
                *color = *found;
                true
            }
            None => false,
        }
    }
    /// `SetColor1`: binds or replaces the color of an element.
    pub fn set_color1(&self, id: i32, color: &Color) {
        set_or_bind(&mut self.maps.write().colors1, id, *color);
    }
    /// `GetColors2`: a copy; `builder.get_colors2().extent()`.
    pub fn get_colors2(&self) -> DataMap<i32, MeshVSTwoColors> {
        let mut copy = DataMap::default();
        copy.assign(&self.maps.read().colors2);
        copy
    }
    /// `SetColors2`; `builder.set_colors2(&map)`.
    pub fn set_colors2(&self, map: &DataMap<i32, MeshVSTwoColors>) {
        self.maps.write().colors2.assign(map);
    }
    /// `HasColors2`; `builder.has_colors2()`.
    pub fn has_colors2(&self) -> bool {
        self.maps.read().colors2.extent() > 0
    }
    /// `GetColor2(ID, MeshVS_TwoColors&)`: the color pair of an element, the output kept without.
    pub fn get_color2(&self, id: i32, colors: &mut MeshVSTwoColors) -> bool {
        match self.maps.read().colors2.seek(&id) {
            Some(found) => {
                *colors = *found;
                true
            }
            None => false,
        }
    }
    /// `GetColor2(ID, Color1, Color2)`: the decoded pair, the outputs kept without.
    pub fn get_color2_colors(&self, id: i32, first: &mut Color, second: &mut Color) -> bool {
        let mut pair = bind_two_colors(Color::default(), Color::default());
        let found = self.get_color2(id, &mut pair);
        if found {
            extract_colors(&mut pair, first, second);
        }
        found
    }
    /// `SetColor2(ID, MeshVS_TwoColors)`: binds or replaces the pair of an element.
    pub fn set_color2(&self, id: i32, colors: &MeshVSTwoColors) {
        set_or_bind(&mut self.maps.write().colors2, id, *colors);
    }
    /// `SetColor2(ID, Color1, Color2)`: the pair of the two colors (sRGB bytes).
    pub fn set_color2_colors(&self, id: i32, first: &Color, second: &Color) {
        self.set_color2(id, &bind_two_colors(*first, *second));
    }
}

impl MeshVSPrsBuilder for MeshVSElementalColorPrsBuilder {
    fn prs_builder_base(&self) -> &MeshVSPrsBuilderBase {
        &self.base
    }
    fn build(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        ids: &MeshVSPackedMap,
        ids_to_exclude: &mut MeshVSPackedMap,
        is_element: bool,
        display_mode: i32,
    ) -> Result<(), Failure> {
        build::build_elemental_colors(self, prs, ids, ids_to_exclude, is_element, display_mode)
    }
}
