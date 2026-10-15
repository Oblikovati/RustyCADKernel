// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_ElementalColorPrsBuilder::Build`: the color groups of the one-color elements, then
//! (build_pairs.rs) the two-color faces.
use super::MeshVSElementalColorPrsBuilder;
use super::build_pairs::add_pair_groups;
use crate::MeshVSDrawerAttribute as Key;
use crate::data_source::{MeshVSDataSource, MeshVSPackedMap, MeshVSTopologyHandle};
use crate::drawer::MeshVSDrawerSource;
use crate::flags::*;
use crate::mesh_prs_builder::{MeshVSMeshPrsBuilder, fan_index, node_xyz};
use crate::nodal_color_prs_builder::{color_materials, line_type};
use crate::prs_builder::{
    MeshVSPrsBuilder, MeshVSPrsBuilderMembers, coords_of, parent_hidden_elems, remaining_ids,
};
use crate::two_colors::{MeshVSTwoColors, extract_colors};
use rustycad_tkernel::ncollection::{Array1, DataMap, HArray1, Map};
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::aspect::{AspectInteriorStyle, AspectTypeOfLine};
use rustycad_tkservice::graphic3d::*;

/// The drawer values of one `Build` call.
pub(super) struct ElementalLook {
    pub interior: Color,
    pub edge: Color,
    pub edge_type: AspectTypeOfLine,
    pub line_type: AspectTypeOfLine,
    pub edge_width: f64,
    pub line_width: f64,
    pub edge_on: bool,
    pub is_reflect: bool,
    pub is_smooth: bool,
    pub suppress_back_faces: bool,
}

fn elemental_look(drawer: &dyn MeshVSDrawerSource) -> Result<ElementalLook, Failure> {
    let (mut interior, mut edge, mut line) = (Color::default(), Color::default(), Color::default());
    drawer.get_color(Key::InteriorColor as i32, &mut interior);
    drawer.get_color(Key::EdgeColor as i32, &mut edge);
    drawer.get_color(Key::BeamColor as i32, &mut line);
    let (mut edge_width, mut line_width) = (1.0, 1.0);
    drawer.get_double(Key::EdgeWidth as i32, &mut edge_width);
    drawer.get_double(Key::BeamWidth as i32, &mut line_width);
    let mut flags = [true, false, false, false];
    for (flag, key) in flags.iter_mut().zip([
        Key::ShowEdges,
        Key::ColorReflection,
        Key::SmoothShading,
        Key::SupressBackFaces,
    ]) {
        drawer.get_boolean(key as i32, flag);
    }
    let read_type = |key: Key| -> Result<AspectTypeOfLine, Failure> {
        let mut value = AspectTypeOfLine::Solid as i32;
        if drawer.get_integer(key as i32, &mut value) {
            return line_type(value);
        }
        Ok(AspectTypeOfLine::Solid)
    };
    Ok(ElementalLook {
        interior,
        edge,
        edge_type: read_type(Key::EdgeType)?,
        line_type: read_type(Key::BeamType)?,
        edge_width,
        line_width,
        edge_on: flags[0],
        is_reflect: flags[1],
        is_smooth: flags[2],
        suppress_back_faces: flags[3],
    })
}

/// The state of one `Build` call.
pub(super) struct ElementalScope<'a> {
    pub builder: &'a MeshVSElementalColorPrsBuilder,
    pub source: &'a dyn MeshVSDataSource,
    pub coords: Array1<'static, f64>,
    pub max_nodes: i32,
    pub ids: MeshVSPackedMap,
    pub look: ElementalLook,
}

impl ElementalScope<'_> {
    /// The geometry of an element into the coordinate buffer, `None` when unknown.
    pub fn geom(&mut self, key: i32) -> Option<(i32, MeshVSEntityType)> {
        let (mut nb_nodes, mut kind) = (0, MESHVS_ET_NONE);
        self.source
            .get_geom(key, true, &mut self.coords, &mut nb_nodes, &mut kind)
            .then_some((nb_nodes, kind))
    }
    /// The element normals (only when `reflected`), as `GetNormalsByElement`.
    pub fn normals(&self, key: i32, asked: bool) -> Option<Handle<HArray1<f64>>> {
        let mut normals = Handle::default();
        (asked
            && self.source.get_normals_by_element(
                key,
                self.look.is_smooth,
                self.max_nodes,
                &mut normals,
            ))
        .then_some(normals)
    }
    /// The fan triangles of a face, with the element normals (or +Z) when reflected.
    pub fn add_face_triangles(
        &self,
        triangles: &Graphic3dArrayOfTriangles,
        nb_nodes: i32,
        normals: Option<&Handle<HArray1<f64>>>,
    ) -> Result<(), Failure> {
        for node in 0..nb_nodes - 2 {
            for sub in 0..3 {
                let at = fan_index(node, sub);
                let p = node_xyz(&self.coords, at + 1)?;
                if !self.look.is_reflect {
                    triangles.add_vertex_f64(p[0], p[1], p[2])?;
                    continue;
                }
                let normal = match normals.and_then(Handle::get) {
                    Some(array) => {
                        let array = array.array1();
                        let value = |i: i32| {
                            array
                                .value(3 * at + i)
                                .copied()
                                .map_err(|e| e.into_failure("MeshVS normals"))
                        };
                        [value(1)?, value(2)?, value(3)?]
                    }
                    None => [0.0, 0.0, 1.0],
                };
                triangles.add_vertex_normal_f64(p, normal)?;
            }
        }
        Ok(())
    }
    /// The border segments of a face.
    pub fn add_face_edges(
        &self,
        edges: &Graphic3dArrayOfSegments,
        nb_nodes: i32,
    ) -> Result<(), Failure> {
        for node in 0..nb_nodes {
            for index in [node, (node + 1) % nb_nodes] {
                let p = node_xyz(&self.coords, index + 1)?;
                edges.add_vertex_f64(p[0], p[1], p[2])?;
            }
        }
        Ok(())
    }
}

/// The groups of `Build`: two-color faces, one-color faces and volumes, links, edges.
pub(super) struct ElementalGroups {
    pub pairs: Handle<dyn Graphic3dGroup>,
    pub solids: Handle<dyn Graphic3dGroup>,
    pub links: Handle<dyn Graphic3dGroup>,
    pub edges: Handle<dyn Graphic3dGroup>,
}

/// The native `MeshVS_ElementalColorPrsBuilder::Build` body.
pub(super) fn build_elemental_colors(
    builder: &MeshVSElementalColorPrsBuilder,
    prs: &Handle<dyn Graphic3dStructureSource>,
    ids: &MeshVSPackedMap,
    ids_to_exclude: &mut MeshVSPackedMap,
    is_element: bool,
    display_mode: i32,
) -> Result<(), Failure> {
    let (source, drawer) = (builder.get_data_source(), builder.get_drawer());
    let (Some(source), Some(drawer)) = (source.get(), drawer.get()) else {
        return Ok(());
    };
    let mut max_nodes = 0;
    // The native test is `!GetInteger(...) && aMaxFaceNodes <= 0` (an uninitialised value
    // when the key is missing): a missing key returns here, a non-positive value continues.
    if !drawer.get_integer(Key::MaxFaceNodes as i32, &mut max_nodes) {
        return Ok(());
    }
    let coords = coords_of(max_nodes)?;
    let has_maps = {
        let maps = builder.maps.read();
        !maps.colors1.is_empty() || !maps.colors2.is_empty()
    };
    if (display_mode & builder.get_flags()) == 0 || !is_element || !has_maps {
        return Ok(());
    }
    let remaining = remaining_ids(ids, &parent_hidden_elems(builder), ids_to_exclude);
    move_equal_pairs(builder);
    let (singles, pairs) = sorted_colors(builder);
    let mut scope = ElementalScope {
        builder,
        source,
        coords,
        max_nodes,
        ids: remaining,
        look: elemental_look(drawer)?,
    };
    count_volume_polygons(&scope);
    let groups = new_groups(prs, &scope, !singles.is_empty(), !pairs.is_empty())?;
    for (color, elements) in singles.items() {
        if !elements.is_empty() {
            add_color_group(
                &mut scope,
                prs,
                &groups,
                *color,
                elements,
                ids_to_exclude,
                display_mode,
            )?;
        }
    }
    add_pair_groups(&mut scope, &groups, &pairs, ids_to_exclude)
}

/// STEP 0: a pair of equal colors becomes a one-color element.
fn move_equal_pairs(builder: &MeshVSElementalColorPrsBuilder) {
    let mut maps = builder.maps.write();
    let mut equal = Vec::new();
    let pairs: Vec<(i32, MeshVSTwoColors)> = maps.colors2.items().map(|(k, v)| (*k, *v)).collect();
    for (key, mut value) in pairs {
        let (mut first, mut second) = (Color::default(), Color::default());
        extract_colors(&mut value, &mut first, &mut second);
        if first == second {
            equal.push(key);
            maps.colors1.bind(key, first);
        }
    }
    for key in equal {
        maps.colors2.unbind(&key);
    }
}

/// STEPS 1 and 2: the elements of each color and of each color pair, the first equal key
/// collecting them (colors compare within `Quantity_Color::Epsilon`).
fn sorted_colors(
    builder: &MeshVSElementalColorPrsBuilder,
) -> (DataMap<Color, Map<i32>>, DataMap<MeshVSTwoColors, Map<i32>>) {
    let maps = builder.maps.read();
    let (mut singles, mut pairs) = (DataMap::default(), DataMap::default());
    for (key, color) in maps.colors1.items() {
        add_sorted(&mut singles, *color, *key);
    }
    for (key, pair) in maps.colors2.items() {
        add_sorted(&mut pairs, *pair, *key);
    }
    (singles, pairs)
}

fn add_sorted<K: PartialEq + std::fmt::Debug + rustycad_tkernel::ncollection::HashKey>(
    sorted: &mut DataMap<K, Map<i32>>,
    value: K,
    key: i32,
) {
    if let Some((_, elements)) = sorted.items_mut().find(|(color, _)| **color == value) {
        elements.add(key);
        return;
    }
    let mut elements = Map::default();
    elements.add(key);
    sorted.bind(value, elements);
}

/// The native counting of the volume polygons, whose result is unused; its Get3DGeom calls are kept.
fn count_volume_polygons(scope: &ElementalScope<'_>) {
    let (mut vertices, mut bounds) = (0, 0);
    for key in scope.ids.iter() {
        let (mut nb_nodes, mut topo) = (0, MeshVSTopologyHandle::default());
        if scope.source.get_3d_geom(key, &mut nb_nodes, &mut topo) {
            MeshVSMeshPrsBuilder::how_many_primitives(
                &topo,
                true,
                false,
                nb_nodes,
                &mut vertices,
                &mut bounds,
            );
        }
    }
}

fn new_groups(
    prs: &Handle<dyn Graphic3dStructureSource>,
    scope: &ElementalScope<'_>,
    has_singles: bool,
    has_pairs: bool,
) -> Result<ElementalGroups, Failure> {
    let look = &scope.look;
    let mut groups = ElementalGroups {
        pairs: Handle::default(),
        solids: Handle::default(),
        links: Handle::default(),
        edges: Handle::default(),
    };
    if has_pairs {
        groups.pairs = prs.new_group()?;
    }
    if has_singles {
        let [front, back] = color_materials(look.is_reflect);
        let fill = Graphic3dAspectFillArea3d::new(
            AspectInteriorStyle::Solid,
            look.interior,
            look.edge,
            look.edge_type,
            look.edge_width,
            front,
            back,
        )?;
        if !look.is_reflect {
            fill.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
        }
        groups.solids = prs.new_group()?;
        groups.links = prs.new_group()?;
        let solids = groups.solids.get().expect("new group");
        solids.set_closed(look.suppress_back_faces);
        solids.set_group_primitives_aspect(&Handle::new(fill).cast::<dyn Graphic3dAspectsSource>());
    }
    if look.edge_on {
        groups.edges = prs.new_group()?;
        let line = Graphic3dAspectLine3d::new(look.edge, look.edge_type, look.edge_width)?;
        groups
            .edges
            .get()
            .expect("new group")
            .set_group_primitives_aspect(&Handle::new(line).cast::<dyn Graphic3dAspectsSource>());
    }
    Ok(groups)
}

/// The arrays of one color: [faces, volumes, edges, links] counts.
fn count_color(scope: &mut ElementalScope<'_>, elements: &Map<i32>) -> [i32; 4] {
    let mut counts = [0; 4];
    let ids: Vec<i32> = scope.ids.iter().filter(|k| elements.contains(k)).collect();
    for key in ids {
        let Some((mut nb_nodes, kind)) = scope.geom(key) else {
            continue;
        };
        if kind == MESHVS_ET_VOLUME {
            let mut topo = MeshVSTopologyHandle::default();
            if scope.source.get_3d_geom(key, &mut nb_nodes, &mut topo) {
                for face in topo
                    .get()
                    .iter()
                    .flat_map(|t| t.array1().iter().map(|f| f.length()).collect::<Vec<_>>())
                {
                    counts[2] += if scope.look.edge_on { face } else { 0 };
                    counts[1] += face - 2;
                }
            }
        } else if kind == MESHVS_ET_LINK {
            counts[3] += if scope.look.edge_on { nb_nodes - 1 } else { 0 };
        } else if kind == MESHVS_ET_FACE {
            counts[2] += if scope.look.edge_on { nb_nodes } else { 0 };
            counts[0] += nb_nodes - 2;
        }
    }
    counts
}

#[allow(clippy::too_many_arguments)]
fn add_color_group(
    scope: &mut ElementalScope<'_>,
    prs: &Handle<dyn Graphic3dStructureSource>,
    groups: &ElementalGroups,
    color: Color,
    elements: &Map<i32>,
    ids_to_exclude: &mut MeshVSPackedMap,
    display_mode: i32,
) -> Result<(), Failure> {
    let [faces, volumes, edges, links] = count_color(scope, elements);
    let look = &scope.look;
    let triangles = Handle::new(Graphic3dArrayOfTriangles::with_options(
        (faces + volumes) * 3,
        0,
        look.is_reflect,
        false,
        false,
    )?);
    let edge_segments = Handle::new(Graphic3dArrayOfSegments::new(edges * 2, 0, 0)?);
    let link_segments = Handle::new(Graphic3dArrayOfSegments::new(links * 2, 0, 0)?);
    let [front, back] = color_materials(look.is_reflect);
    let fill = Handle::new(Graphic3dAspectFillArea3d::new(
        AspectInteriorStyle::Solid,
        color,
        look.edge,
        look.edge_type,
        look.edge_width,
        front,
        back,
    )?);
    let link_aspect = Handle::new(Graphic3dAspectLine3d::new(
        color,
        look.line_type,
        look.line_width,
    )?);
    fill.set_distinguish(false);
    fill.set_interior_color(color);
    fill.set_draw_edges(false);
    if !look.is_reflect {
        fill.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    }
    let mut drawn = ColorDrawn::default();
    let ids: Vec<i32> = scope.ids.iter().filter(|k| elements.contains(k)).collect();
    for key in ids {
        add_colored_element(
            scope,
            key,
            (&triangles, &edge_segments, &link_segments),
            &mut drawn,
            ids_to_exclude,
        )?;
    }
    let look = &scope.look;
    let fill_aspect: Handle<dyn Graphic3dAspectsSource> = fill.cast();
    if drawn.polygons {
        let solids = groups.solids.get().expect("solid group");
        solids.set_primitives_aspect(&fill_aspect);
        solids.add_primitive_array(&triangles.cast(), true);
        solids.set_closed(look.suppress_back_faces);
        if let Some(edges) = groups.edges.get().filter(|_| look.edge_on) {
            edges.add_primitive_array(&edge_segments.cast(), true);
        }
    }
    if drawn.links {
        let links = groups.links.get().expect("link group");
        links.set_primitives_aspect(&fill_aspect);
        links.set_primitives_aspect(&link_aspect.cast::<dyn Graphic3dAspectsSource>());
        links.add_primitive_array(&link_segments.cast(), true);
    }
    if !drawn.custom.is_empty() {
        scope
            .builder
            .custom_build(prs, &drawn.custom, ids_to_exclude, display_mode)?;
    }
    Ok(())
}

/// What one color drew: `IsPolyG`, `IsPolyL` and the custom elements.
#[derive(Default)]
struct ColorDrawn {
    polygons: bool,
    links: bool,
    custom: MeshVSPackedMap,
}

type ColorArrays<'a> = (
    &'a Handle<Graphic3dArrayOfTriangles>,
    &'a Handle<Graphic3dArrayOfSegments>,
    &'a Handle<Graphic3dArrayOfSegments>,
);

fn add_colored_element(
    scope: &mut ElementalScope<'_>,
    key: i32,
    (triangles, edges, links): ColorArrays<'_>,
    drawn: &mut ColorDrawn,
    ids_to_exclude: &mut MeshVSPackedMap,
) -> Result<(), Failure> {
    let Some((mut nb_nodes, kind)) = scope.geom(key) else {
        return Ok(());
    };
    if ![MESHVS_ET_FACE, MESHVS_ET_LINK, MESHVS_ET_VOLUME].contains(&kind) {
        drawn.custom.add(key);
        return Ok(());
    }
    if scope.builder.is_excluding_on() {
        ids_to_exclude.add(key);
    }
    let (reflect, edge_on) = (scope.look.is_reflect, scope.look.edge_on);
    if kind == MESHVS_ET_VOLUME {
        let mut topo = MeshVSTopologyHandle::default();
        if !scope.source.get_3d_geom(key, &mut nb_nodes, &mut topo) {
            return Ok(());
        }
        MeshVSMeshPrsBuilder::add_volume_prs(
            &topo,
            &scope.coords,
            nb_nodes,
            &triangles.clone().cast(),
            reflect,
            false,
            false,
            1.0,
        )?;
        if edge_on {
            MeshVSMeshPrsBuilder::add_volume_prs(
                &topo,
                &scope.coords,
                nb_nodes,
                &edges.clone().cast(),
                reflect,
                false,
                false,
                1.0,
            )?;
        }
        drawn.polygons = true;
    } else if kind == MESHVS_ET_FACE {
        let normals = scope.normals(key, reflect);
        scope.add_face_triangles(
            triangles.get().expect("triangles"),
            nb_nodes,
            normals.as_ref(),
        )?;
        if edge_on {
            scope.add_face_edges(edges.get().expect("edges"), nb_nodes)?;
        }
        drawn.polygons = true;
    } else {
        for node in 0..nb_nodes - 1 {
            for index in [node, node + 1] {
                let p = node_xyz(&scope.coords, index + 1)?;
                links.add_vertex_f64(p[0], p[1], p[2])?;
            }
            drawn.links = true;
        }
    }
    Ok(())
}
