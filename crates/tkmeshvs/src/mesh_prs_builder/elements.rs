// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_MeshPrsBuilder::BuildElements`: the triangles of faces and volumes, their edges
//! and the links, in the display mode the builder flags select.
use super::elements_faces::add_face;
use super::{MeshVSMeshPrsBuilder, MeshVSMeshPrsBuilderSource};
use crate::MeshVSDrawerAttribute as Key;
use crate::MeshVSTool;
use crate::data_source::{MeshVSDataSource, MeshVSPackedMap, MeshVSTopologyHandle};
use crate::drawer::MeshVSDrawerSource;
use crate::flags::*;
use crate::node_pairs::MeshVSTwoNodes;
use crate::prs_builder::{MeshVSPrsBuilderMembers, coords_of, parent_hidden_elems, remaining_ids};
use rustycad_tkernel::ncollection::{Array1, Map};
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::*;

/// The display flags and drawer values of one `BuildElements` call.
pub(super) struct ElementSettings {
    pub max_nodes: i32,
    pub shrink_coef: f64,
    pub is_shading: bool,
    pub is_shrink: bool,
    pub has_select: bool,
    pub is_reflect: bool,
    pub is_smooth: bool,
    pub overlap_control: bool,
    pub show_edges: bool,
}

/// The arrays of one `BuildElements` call; the segments are null without edges.
pub(super) struct ElementArrays {
    pub volume_triangles: Handle<Graphic3dArrayOfTriangles>,
    pub face_triangles: Handle<Graphic3dArrayOfTriangles>,
    pub link_segments: Handle<Graphic3dArrayOfSegments>,
    pub edge_segments: Handle<Graphic3dArrayOfSegments>,
}

fn drawer_bool(drawer: &dyn MeshVSDrawerSource, key: Key, default: bool) -> bool {
    let mut value = default;
    drawer.get_boolean(key as i32, &mut value);
    value
}

/// The real display mode of the builder flags (a deformed mode becomes its plain mode).
fn plain_mode(display_mode: i32, flags: i32) -> i32 {
    let mut mode = display_mode & flags;
    if (mode & MESHVS_DMF_DEFORMED_MASK) != 0 {
        mode /= MESHVS_DMF_DEFORMED_PRS_WIRE_FRAME;
    }
    mode & MESHVS_DMF_OCC_MASK
}

fn element_settings(
    drawer: &dyn MeshVSDrawerSource,
    display_mode: i32,
    flags: i32,
    max_nodes: i32,
) -> ElementSettings {
    let mode = plain_mode(display_mode, flags);
    let mut shrink_coef = 0.0;
    drawer.get_double(Key::ShrinkCoeff as i32, &mut shrink_coef);
    let has_hilight = (display_mode & MESHVS_DMF_HILIGHT_PRS) != 0;
    let has_select = (display_mode & MESHVS_DMF_SELECTION_PRS) != 0;
    let is_wire_frame = mode == MESHVS_DMF_WIRE_FRAME;
    let is_shading = mode == MESHVS_DMF_SHADING || has_hilight;
    let allow_overlap = drawer_bool(drawer, Key::IsAllowOverlapped, false);
    ElementSettings {
        max_nodes,
        shrink_coef,
        is_shading,
        is_shrink: mode == MESHVS_DMF_SHRINK && !has_hilight,
        has_select,
        is_reflect: drawer_bool(drawer, Key::Reflection, false) && !has_hilight,
        is_smooth: drawer_bool(drawer, Key::SmoothShading, false),
        overlap_control: !allow_overlap
            && (is_wire_frame || mode == MESHVS_DMF_SHADING || has_hilight)
            && !has_select,
        show_edges: is_wire_frame || drawer_bool(drawer, Key::ShowEdges, true),
    }
}

/// The native `MeshVS_MeshPrsBuilder::BuildElements` body.
/// Example: `meshvs_mesh_prs_build_elements(&*builder, &prs, &ids, &mut excluded, 1)?`.
pub fn meshvs_mesh_prs_build_elements<B: MeshVSMeshPrsBuilderSource + ?Sized>(
    builder: &B,
    prs: &Handle<dyn Graphic3dStructureSource>,
    ids: &MeshVSPackedMap,
    ids_to_exclude: &mut MeshVSPackedMap,
    display_mode: i32,
) -> Result<(), Failure> {
    let source = builder.get_data_source();
    let drawer = builder.get_drawer();
    let (Some(source), Some(drawer_value)) = (source.get(), drawer.get()) else {
        return Ok(());
    };
    let mut max_nodes = 0;
    if !drawer_value.get_integer(Key::MaxFaceNodes as i32, &mut max_nodes) || max_nodes <= 0 {
        return Ok(());
    }
    let settings = element_settings(drawer_value, display_mode, builder.get_flags(), max_nodes);
    let mut material = Graphic3dMaterialAspect::default();
    drawer_value.get_material(Key::FrontMaterial as i32, &mut material);
    let fill = MeshVSTool::create_aspect_fill_area3d_with_material(
        &builder.get_drawer(),
        &material,
        true,
    )?;
    let fill_value = fill.get().ok_or_else(null_fill)?;
    if !settings.is_reflect {
        fill_value.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    }
    let beam = MeshVSTool::create_aspect_line3d(&builder.get_drawer(), true)?;
    let remaining = remaining_ids(ids, &parent_hidden_elems(builder), ids_to_exclude);
    let mut coords = coords_of(max_nodes)?;
    let arrays = new_arrays(source, &remaining, &mut coords, &settings)?;
    let link_nodes = overlap_links(source, &remaining, &settings)?;
    let mut custom = MeshVSPackedMap::default();
    let pass = ElementPass {
        builder_excludes: builder.is_excluding_on(),
        settings: &settings,
        arrays: &arrays,
        link_nodes: &link_nodes,
    };
    pass.run(
        builder,
        source,
        &remaining,
        &mut coords,
        ids_to_exclude,
        &mut custom,
    )?;
    let old_edge_color = fill_value.edge_color();
    if settings.is_shrink {
        fill_value.set_edge_color(Color::named(NamedColor::BLACK));
    }
    let draw = super::draw_arrays::DrawArraysInput {
        polygons: arrays.face_triangles.clone().cast(),
        lines: arrays.edge_segments.clone().cast(),
        link_lines: arrays.link_segments.clone().cast(),
        volumes_in_shad: arrays.volume_triangles.clone().cast(),
        is_polygons_edges_off: !settings.show_edges,
        is_selected: settings.has_select,
    };
    super::draw_arrays::meshvs_mesh_prs_draw_arrays(builder, prs, &draw, &fill, &beam)?;
    if !custom.is_empty() {
        builder.custom_build(prs, &custom, ids_to_exclude, display_mode)?;
    }
    if settings.is_shrink {
        fill_value.set_edge_color(old_edge_color);
    }
    Ok(())
}

fn null_fill() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "MeshVS_MeshPrsBuilder fill aspect=null: expected the drawer fill keys (native null dereference)",
        "",
    )
}

/// The first pass: the number of triangles and segments to allocate.
fn new_arrays(
    source: &dyn MeshVSDataSource,
    ids: &MeshVSPackedMap,
    coords: &mut Array1<'_, f64>,
    settings: &ElementSettings,
) -> Result<ElementArrays, Failure> {
    let mut counts = [0; 4]; // faces, volumes, edges, links
    let mut topo = MeshVSTopologyHandle::default();
    for key in ids.iter() {
        let (mut nb_nodes, mut kind) = (0, MESHVS_ET_NONE);
        if !source.get_geom(key, true, coords, &mut nb_nodes, &mut kind) {
            continue;
        }
        count_element(
            source,
            key,
            kind,
            nb_nodes,
            &mut topo,
            settings,
            &mut counts,
        );
    }
    let [faces, volumes, edges, links] = counts;
    let reflect = settings.is_reflect;
    let segments = |count: i32| -> Result<Handle<Graphic3dArrayOfSegments>, Failure> {
        if !settings.show_edges {
            return Ok(Handle::default());
        }
        Ok(Handle::new(Graphic3dArrayOfSegments::new(count * 2, 0, 0)?))
    };
    Ok(ElementArrays {
        volume_triangles: Handle::new(Graphic3dArrayOfTriangles::with_options(
            volumes * 3,
            0,
            reflect,
            false,
            false,
        )?),
        face_triangles: Handle::new(Graphic3dArrayOfTriangles::with_options(
            faces * 3,
            0,
            reflect,
            false,
            false,
        )?),
        link_segments: segments(links)?,
        edge_segments: segments(edges)?,
    })
}

#[allow(clippy::too_many_arguments)]
fn count_element(
    source: &dyn MeshVSDataSource,
    key: i32,
    kind: MeshVSEntityType,
    nb_nodes: i32,
    topo: &mut MeshVSTopologyHandle,
    settings: &ElementSettings,
    counts: &mut [i32; 4],
) {
    let solid = (settings.is_shading || settings.is_shrink) && !settings.has_select;
    if kind == MESHVS_ET_VOLUME {
        let mut nb = nb_nodes;
        if !source.get_3d_geom(key, &mut nb, topo) {
            return;
        }
        for face in topo
            .get()
            .iter()
            .flat_map(|t| t.array1().iter().map(|f| f.length()).collect::<Vec<_>>())
        {
            if settings.show_edges {
                counts[2] += face;
            }
            if solid {
                counts[1] += face - 2;
            }
        }
    } else if kind == MESHVS_ET_LINK {
        if settings.show_edges {
            counts[3] += 1;
        }
    } else if kind == MESHVS_ET_FACE {
        if settings.show_edges {
            counts[2] += nb_nodes;
        }
        if (!settings.overlap_control || settings.is_shading) && solid {
            counts[0] += nb_nodes - 2;
        }
    }
}

/// The links of two nodes, whose face edges are not drawn again (overlap control).
fn overlap_links(
    source: &dyn MeshVSDataSource,
    ids: &MeshVSPackedMap,
    settings: &ElementSettings,
) -> Result<Map<MeshVSTwoNodes>, Failure> {
    let mut links = Map::<MeshVSTwoNodes>::default();
    if !(settings.show_edges && settings.overlap_control) {
        return Ok(links);
    }
    let mut nodes = node_ids(settings.max_nodes)?;
    for key in ids.iter() {
        let mut kind = MESHVS_ET_NONE;
        if !source.get_geom_type(key, true, &mut kind) || kind != MESHVS_ET_LINK {
            continue;
        }
        let mut nb_nodes = 0;
        if source.get_nodes_by_element(key, &mut nodes, &mut nb_nodes) && nb_nodes == 2 {
            links.add(MeshVSTwoNodes::new(nodes[1], nodes[2]));
        }
    }
    Ok(links)
}

/// A one-based node id array of `max_nodes` values.
pub(super) fn node_ids(max_nodes: i32) -> Result<Array1<'static, i32>, Failure> {
    Array1::with_bounds(1, max_nodes).map_err(|error| error.into_failure("MeshVS node ids"))
}

/// The second pass over the elements.
pub(super) struct ElementPass<'a> {
    pub builder_excludes: bool,
    pub settings: &'a ElementSettings,
    pub arrays: &'a ElementArrays,
    pub link_nodes: &'a Map<MeshVSTwoNodes>,
}

impl ElementPass<'_> {
    fn run<B: MeshVSMeshPrsBuilderSource + ?Sized>(
        &self,
        builder: &B,
        source: &dyn MeshVSDataSource,
        ids: &MeshVSPackedMap,
        coords: &mut Array1<'_, f64>,
        ids_to_exclude: &mut MeshVSPackedMap,
        custom: &mut MeshVSPackedMap,
    ) -> Result<(), Failure> {
        let mut nodes = node_ids(self.settings.max_nodes)?;
        let mut segment_map = Map::default();
        for key in ids.iter() {
            let (mut nb_nodes, mut kind) = (0, MESHVS_ET_NONE);
            if !source.get_geom(key, true, coords, &mut nb_nodes, &mut kind)
                || !source.get_nodes_by_element(key, &mut nodes, &mut nb_nodes)
            {
                continue;
            }
            let known = [MESHVS_ET_VOLUME, MESHVS_ET_LINK, MESHVS_ET_FACE].contains(&kind);
            if !known {
                custom.add(key);
                continue;
            }
            if self.builder_excludes {
                ids_to_exclude.add(key);
            }
            let element = Element {
                key,
                nb_nodes,
                coords,
                nodes: &nodes,
            };
            match kind {
                MESHVS_ET_VOLUME => self.add_volume(source, &element)?,
                MESHVS_ET_LINK => self.add_link(&element)?,
                _ => add_face(builder, self, &element, &mut segment_map)?,
            }
        }
        Ok(())
    }
    fn add_volume(
        &self,
        source: &dyn MeshVSDataSource,
        element: &Element<'_, '_>,
    ) -> Result<(), Failure> {
        let mut topo = MeshVSTopologyHandle::default();
        let mut nb_nodes = element.nb_nodes;
        if !source.get_3d_geom(element.key, &mut nb_nodes, &mut topo) {
            return Ok(());
        }
        let s = self.settings;
        if s.show_edges {
            MeshVSMeshPrsBuilder::add_volume_prs(
                &topo,
                element.coords,
                nb_nodes,
                &self.arrays.edge_segments.clone().cast(),
                s.is_reflect,
                s.is_shrink,
                s.has_select,
                s.shrink_coef,
            )?;
        }
        if (s.is_shading || s.is_shrink) && !s.has_select {
            MeshVSMeshPrsBuilder::add_volume_prs(
                &topo,
                element.coords,
                nb_nodes,
                &self.arrays.volume_triangles.clone().cast(),
                s.is_reflect,
                s.is_shrink,
                s.has_select,
                s.shrink_coef,
            )?;
        }
        Ok(())
    }
    fn add_link(&self, element: &Element<'_, '_>) -> Result<(), Failure> {
        let s = self.settings;
        match self.arrays.link_segments.get() {
            Some(links) if s.show_edges => MeshVSMeshPrsBuilder::add_link_prs(
                element.coords,
                links,
                s.is_shrink || s.has_select,
                s.shrink_coef,
            ),
            _ => Ok(()),
        }
    }
}

/// One element of the second pass: its key, node count, coordinates and node ids.
pub(super) struct Element<'c, 'a> {
    pub key: i32,
    pub nb_nodes: i32,
    pub coords: &'c Array1<'a, f64>,
    pub nodes: &'c Array1<'static, i32>,
}
