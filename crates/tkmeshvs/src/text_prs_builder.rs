// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_TextPrsBuilder` at OCCT 3d097a0: the texts of nodes (at the node) and elements (at
//! the mean of their nodes), each also marked by a gray point.
use crate::MeshVSDrawerAttribute as Key;
use crate::data_source::{MeshVSDataSource, MeshVSPackedMap};
use crate::drawer::MeshVSDrawerSource;
use crate::flags::*;
use crate::mesh::MeshVSMeshSource;
use crate::mesh_prs_builder::node_xyz;
use crate::prs_builder::{
    MeshVSPrsBuilder, MeshVSPrsBuilderBase, MeshVSPrsBuilderMembers, coords_of,
    parent_hidden_elems, remaining_ids,
};
use crate::selection_policy::MESHVS_BP_TEXT;
use crate::tool::text::{font_aspect, terminated_font, text_display, text_style};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::aspect::AspectTypeOfMarker;
use rustycad_tkservice::graphic3d::*;

/// The node and element texts of `MeshVS_TextPrsBuilder`.
#[derive(Default)]
pub struct MeshVSTextMaps {
    /// `myNodeTextMap`.
    pub nodes: DataMap<i32, AsciiString>,
    /// `myElemTextMap`.
    pub elements: DataMap<i32, AsciiString>,
}

impl MeshVSTextMaps {
    fn of(&self, is_element: bool) -> &DataMap<i32, AsciiString> {
        if is_element {
            &self.elements
        } else {
            &self.nodes
        }
    }
    fn of_mut(&mut self, is_element: bool) -> &mut DataMap<i32, AsciiString> {
        if is_element {
            &mut self.elements
        } else {
            &mut self.nodes
        }
    }
}

/// The builder of mesh texts; `MeshVSTextPrsBuilder::new(&mesh, 10.0, &color, MESHVS_DMF_TEXT_DATA_PRS, &Handle::default(), -1, MESHVS_BP_TEXT)`.
pub struct MeshVSTextPrsBuilder {
    transient: StandardTransient,
    base: MeshVSPrsBuilderBase,
    /// The text maps; `builder.texts.read().nodes.extent()`.
    pub texts: SharedState<MeshVSTextMaps>,
}
rustycad_tkernel::impl_transient!(MeshVSTextPrsBuilder, "MeshVS_TextPrsBuilder", dyn MeshVSPrsBuilder, transient,
    interfaces: [dyn MeshVSPrsBuilder]);

impl MeshVSTextPrsBuilder {
    /// `MeshVS_TextPrsBuilder(Parent, Height, Color, Flags, DS, Id, Priority)`: the height and
    /// color go into the drawer of the builder (natively the mesh drawer, the builder having none).
    pub fn new(
        parent: &Handle<dyn MeshVSMeshSource>,
        height: f64,
        color: &Color,
        flags: i32,
        data_source: &Handle<dyn MeshVSDataSource>,
        id: i32,
        priority: i32,
    ) -> Self {
        let builder = Self {
            transient: StandardTransient::default(),
            base: MeshVSPrsBuilderBase::new(parent, flags, data_source, id, priority),
            texts: SharedState::default(),
        };
        if let Some(drawer) = builder.get_drawer().get() {
            drawer.set_double(Key::TextHeight as i32, height);
            drawer.set_color(Key::TextColor as i32, color);
        }
        builder
    }
    /// The native default arguments: `MeshVS_DMF_TextDataPrs`, no data source, a free id and
    /// `MeshVS_BP_Text`. Example: `MeshVSTextPrsBuilder::with_defaults(&mesh, 10.0, &color)`.
    pub fn with_defaults(
        parent: &Handle<dyn MeshVSMeshSource>,
        height: f64,
        color: &Color,
    ) -> Self {
        Self::new(
            parent,
            height,
            color,
            MESHVS_DMF_TEXT_DATA_PRS,
            &Handle::default(),
            -1,
            MESHVS_BP_TEXT,
        )
    }
    /// `GetTexts`: a copy of the node or element texts; `builder.get_texts(true)`.
    pub fn get_texts(&self, is_element: bool) -> DataMap<i32, AsciiString> {
        let mut copy = DataMap::default();
        copy.assign(self.texts.read().of(is_element));
        copy
    }
    /// `SetTexts`; `builder.set_texts(false, &map)`.
    pub fn set_texts(&self, is_element: bool, map: &DataMap<i32, AsciiString>) {
        self.texts.write().of_mut(is_element).assign(map);
    }
    /// `HasTexts`; `builder.has_texts(true)`.
    pub fn has_texts(&self, is_element: bool) -> bool {
        self.texts.read().of(is_element).extent() > 0
    }
    /// `GetText`: the text of an id, the output kept without; `builder.get_text(false, 3, &mut s)`.
    pub fn get_text(&self, is_element: bool, id: i32, text: &mut AsciiString) -> bool {
        match self.texts.read().of(is_element).seek(&id) {
            Some(found) => {
                *text = found.clone();
                true
            }
            None => false,
        }
    }
    /// `SetText`: binds or replaces the text of an id; `builder.set_text(false, 3, &text)`.
    pub fn set_text(&self, is_element: bool, id: i32, text: &AsciiString) {
        let mut texts = self.texts.write();
        let map = texts.of_mut(is_element);
        match map.change_find(&id) {
            Ok(slot) => *slot = text.clone(),
            Err(_) => {
                map.bind(id, text.clone());
            }
        }
    }
}

/// The text aspect of the drawer: yellow bold annotation text in the ASCII mono font by default.
fn text_aspect(drawer: &dyn MeshVSDrawerSource) -> Result<Graphic3dAspectText3d, Failure> {
    let mut color = Color::named(NamedColor::YELLOW);
    let (mut expansion, mut space) = (1.0, 0.0);
    drawer.get_color(Key::TextColor as i32, &mut color);
    drawer.get_double(Key::TextExpansionFactor as i32, &mut expansion);
    drawer.get_double(Key::TextSpace as i32, &mut space);
    let mut font = AsciiString::new("Courier");
    drawer.get_ascii_string(Key::TextFont as i32, &mut font);
    let mut values = [1, 0, 1]; // Aspect_TOST_ANNOTATION, Aspect_TODT_NORMAL, Font_FA_Bold
    for (value, key) in
        values
            .iter_mut()
            .zip([Key::TextStyle, Key::TextDisplayType, Key::TextFontAspect])
    {
        drawer.get_integer(key as i32, value);
    }
    let aspect = Graphic3dAspectText3d::new(
        color,
        None,
        expansion,
        space,
        text_style(values[0])?,
        text_display(values[1])?,
    );
    aspect.set_font(&terminated_font(&font));
    aspect.set_text_font_aspect(font_aspect(values[2])?);
    Ok(aspect)
}

impl MeshVSPrsBuilder for MeshVSTextPrsBuilder {
    fn prs_builder_base(&self) -> &MeshVSPrsBuilderBase {
        &self.base
    }
    /// `Build`: one text group, then one marker group of the text points.
    fn build(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        ids: &MeshVSPackedMap,
        ids_to_exclude: &mut MeshVSPackedMap,
        is_element: bool,
        display_mode: i32,
    ) -> Result<(), Failure> {
        let (source, drawer) = (self.get_data_source(), self.get_drawer());
        let (Some(source), Some(drawer)) = (source.get(), drawer.get()) else {
            return Ok(());
        };
        if !self.has_texts(is_element) || (display_mode & self.get_flags()) == 0 {
            return Ok(());
        }
        let (mut max_nodes, mut height) = (0, 0.0);
        if !drawer.get_integer(Key::MaxFaceNodes as i32, &mut max_nodes)
            || max_nodes <= 0
            || !drawer.get_double(Key::TextHeight as i32, &mut height)
        {
            return Ok(());
        }
        let text_group = prs.new_group()?;
        text_group.set_group_primitives_aspect(
            &Handle::new(text_aspect(drawer)?).cast::<dyn Graphic3dAspectsSource>(),
        );
        let hidden = if is_element {
            parent_hidden_elems(self)
        } else {
            Handle::default()
        };
        let remaining = remaining_ids(ids, &hidden, ids_to_exclude);
        let mut placed = TextPlacement {
            source,
            coords: coords_of(max_nodes)?,
            points: Vec::new(),
            custom: MeshVSPackedMap::default(),
        };
        for key in remaining.iter() {
            let mut text = AsciiString::default();
            if !self.get_text(is_element, key, &mut text) {
                continue;
            }
            let Some(position) = placed.position(self, key, is_element, ids_to_exclude)? else {
                continue;
            };
            let label = Handle::new(Graphic3dText::new(height as f32));
            label.set_text_ascii(&text);
            label.set_position(&position);
            text_group.add_text(&label, true);
        }
        add_text_markers(prs, &placed.points)?;
        if !placed.custom.is_empty() {
            self.custom_build(prs, &placed.custom, ids_to_exclude, display_mode)?;
        }
        Ok(())
    }
}

/// The text positions of one `Build` call.
pub(crate) struct TextPlacement<'a> {
    pub source: &'a dyn MeshVSDataSource,
    pub coords: rustycad_tkernel::ncollection::Array1<'static, f64>,
    /// The float positions of the texts, in order (`aPnts`).
    pub points: Vec<[f32; 3]>,
    pub custom: MeshVSPackedMap,
}

impl TextPlacement<'_> {
    /// The node position, or the mean of an element's nodes (excluded when the builder
    /// excludes); `None` for an unknown geometry or a custom element.
    pub fn position<B: MeshVSPrsBuilder + ?Sized>(
        &mut self,
        builder: &B,
        key: i32,
        is_element: bool,
        ids_to_exclude: &mut MeshVSPackedMap,
    ) -> Result<Option<Point3>, Failure> {
        let (mut nb_nodes, mut kind) = (0, MESHVS_ET_NONE);
        if !self
            .source
            .get_geom(key, is_element, &mut self.coords, &mut nb_nodes, &mut kind)
        {
            return Ok(None);
        }
        let p = match kind {
            MESHVS_ET_NODE => node_xyz(&self.coords, 1)?,
            MESHVS_ET_LINK | MESHVS_ET_FACE | MESHVS_ET_VOLUME => {
                if is_element && builder.is_excluding_on() {
                    ids_to_exclude.add(key);
                }
                element_mean(&self.coords, nb_nodes)?
            }
            _ => {
                self.custom.add(key);
                return Ok(None);
            }
        };
        self.points.push(p.map(|c| c as f32));
        Ok(Some(Point3::new(p[0], p[1], p[2])))
    }
}

/// The mean of the element nodes (sums in node order, divided by the count).
pub(crate) fn element_mean(
    coords: &rustycad_tkernel::ncollection::Array1<'_, f64>,
    nb_nodes: i32,
) -> Result<[f64; 3], Failure> {
    let mut sum = [0.0; 3];
    for i in 1..=nb_nodes {
        let p = node_xyz(coords, i)?;
        for axis in 0..3 {
            sum[axis] += p[axis];
        }
    }
    Ok(sum.map(|c| c / f64::from(nb_nodes)))
}

fn add_text_markers(
    prs: &Handle<dyn Graphic3dStructureSource>,
    points: &[[f32; 3]],
) -> Result<(), Failure> {
    if points.is_empty() {
        return Ok(());
    }
    let group = prs.new_group()?;
    let array = Handle::new(Graphic3dArrayOfPoints::new(points.len() as i32, 0)?);
    for p in points {
        array.add_vertex(p[0], p[1], p[2])?;
    }
    let marker = Graphic3dAspectMarker3d::new(
        AspectTypeOfMarker::Point,
        Color::named(NamedColor::GRAY),
        1.0,
    )?;
    group.set_group_primitives_aspect(&Handle::new(marker).cast::<dyn Graphic3dAspectsSource>());
    group.add_primitive_array(&array.cast(), true);
    Ok(())
}
