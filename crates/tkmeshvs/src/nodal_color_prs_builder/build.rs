// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_NodalColorPrsBuilder::Build`: one triangle array of the colored elements and one
//! segment array of their edges, in two groups.
use super::MeshVSNodalColorPrsBuilder;
use super::texture::{create_texture, get_nearest_pow2};
use super::volume::{MeshVSNodalColorScale, scaled, texel};
use crate::MeshVSDrawerAttribute as Key;
use crate::data_source::{MeshVSDataSource, MeshVSPackedMap, MeshVSTopologyHandle};
use crate::drawer::MeshVSDrawerSource;
use crate::flags::*;
use crate::mesh_prs_builder::draw_arrays_copy_fill as copy_fill;
use crate::mesh_prs_builder::{MeshVSMeshPrsBuilder, fan_index, node_xyz};
use crate::prs_builder::{MeshVSPrsBuilderMembers, coords_of, parent_hidden_elems, remaining_ids};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Direction3, Point3, resolution};
use rustycad_tkservice::aspect::{AspectInteriorStyle, AspectTypeOfLine};
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkv3d::prs3d::Prs3dShadingAspect;

/// The two plastic materials of the colored elements: black specular and emissive colors,
/// half ambient and diffuse reflection when reflected.
pub(crate) fn color_materials(is_reflect: bool) -> [Graphic3dMaterialAspect; 2] {
    std::array::from_fn(|_| {
        let mut material = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Plastified);
        material.set_specular_color(Color::named(NamedColor::BLACK));
        material.set_emissive_color(Color::named(NamedColor::BLACK));
        if is_reflect {
            let half = Color::from_vec3(rustycad_tkernel::ncollection::Vec3::new(0.5, 0.5, 0.5))
                .expect("valid gray");
            material.set_ambient_color(half);
            material.set_diffuse_color(half);
        }
        material
    })
}

struct NodalSettings {
    max_nodes: i32,
    is_reflect: bool,
    is_smooth: bool,
    use_texture: bool,
    scale: MeshVSNodalColorScale,
}

/// The native `MeshVS_NodalColorPrsBuilder::Build` body.
pub(super) fn build_nodal_colors(
    builder: &MeshVSNodalColorPrsBuilder,
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
    if !drawer.get_integer(Key::MaxFaceNodes as i32, &mut max_nodes) || max_nodes <= 0 {
        return Ok(());
    }
    let mut coords = coords_of(max_nodes)?;
    if (display_mode & builder.get_flags()) == 0 || !is_element || !has_color_data(builder) {
        return Ok(());
    }
    let remaining = remaining_ids(ids, &parent_hidden_elems(builder), ids_to_exclude);
    let settings = nodal_settings(builder, drawer, max_nodes);
    // The native polygon array sized by the volume primitives is never drawn; its counting
    // calls of Get3DGeom are kept.
    count_volume_polygons(source, &remaining);
    let (triangles, edges) = new_arrays(source, &remaining, &mut coords, &settings)?;
    for key in remaining.iter() {
        add_element(
            builder,
            source,
            key,
            &mut coords,
            &settings,
            (&triangles, &edges),
            ids_to_exclude,
        )?;
    }
    add_groups(builder, prs, drawer, &settings, &triangles, &edges)
}

fn has_color_data(builder: &MeshVSNodalColorPrsBuilder) -> bool {
    let fields = builder.fields.read();
    if fields.use_texture {
        return fields.texture_coords.extent() > 0 && fields.texture_color_map.length() > 0;
    }
    fields.node_colors.extent() > 0
}

fn nodal_settings(
    builder: &MeshVSNodalColorPrsBuilder,
    drawer: &dyn MeshVSDrawerSource,
    max_nodes: i32,
) -> NodalSettings {
    let (mut is_reflect, mut is_smooth) = (false, false);
    drawer.get_boolean(Key::ColorReflection as i32, &mut is_reflect);
    drawer.get_boolean(Key::SmoothShading as i32, &mut is_smooth);
    let fields = builder.fields.read();
    let nb_colors = if fields.use_texture {
        fields.texture_color_map.length()
    } else {
        0
    };
    let nb_tex_colors = if fields.use_texture {
        get_nearest_pow2(nb_colors)
    } else {
        0
    };
    NodalSettings {
        max_nodes,
        is_reflect,
        is_smooth,
        use_texture: fields.use_texture,
        scale: MeshVSNodalColorScale {
            nb_colors,
            nb_tex_colors,
            color_ratio: 1.0,
        },
    }
}

fn count_volume_polygons(source: &dyn MeshVSDataSource, ids: &MeshVSPackedMap) {
    let (mut vertices, mut bounds) = (0, 0);
    for key in ids.iter() {
        let (mut nb_nodes, mut topo) = (0, MeshVSTopologyHandle::default());
        if source.get_3d_geom(key, &mut nb_nodes, &mut topo) {
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

fn new_arrays(
    source: &dyn MeshVSDataSource,
    ids: &MeshVSPackedMap,
    coords: &mut Array1<'_, f64>,
    settings: &NodalSettings,
) -> Result<
    (
        Handle<Graphic3dArrayOfTriangles>,
        Handle<Graphic3dArrayOfSegments>,
    ),
    Failure,
> {
    let (mut faces, mut volumes, mut edges) = (0, 0, 0);
    for key in ids.iter() {
        let (mut nb_nodes, mut kind) = (0, MESHVS_ET_NONE);
        if !source.get_geom(key, true, coords, &mut nb_nodes, &mut kind) {
            continue;
        }
        if kind == MESHVS_ET_FACE {
            edges += nb_nodes;
            faces += nb_nodes - 2;
            continue;
        }
        let mut topo = MeshVSTopologyHandle::default();
        if kind == MESHVS_ET_VOLUME && source.get_3d_geom(key, &mut nb_nodes, &mut topo) {
            for face in topo
                .get()
                .iter()
                .flat_map(|t| t.array1().iter().map(|f| f.length()).collect::<Vec<_>>())
            {
                edges += face;
                volumes += face - 2;
            }
        }
    }
    let texture = settings.use_texture;
    let triangles = Graphic3dArrayOfTriangles::with_options(
        (faces + volumes) * 3,
        0,
        texture || settings.is_reflect,
        !texture,
        texture,
    )?;
    Ok((
        Handle::new(triangles),
        Handle::new(Graphic3dArrayOfSegments::new(edges * 2, 0, 0)?),
    ))
}

/// Every node of the element has a color (or a texture coordinate).
fn all_nodes_colored(
    builder: &MeshVSNodalColorPrsBuilder,
    nodes: &Array1<'_, i32>,
    nb_nodes: i32,
) -> bool {
    let fields = builder.fields.read();
    (1..=nb_nodes).all(|k| {
        let node = nodes.value(k).copied().unwrap_or_default();
        if fields.use_texture {
            fields.texture_coords.is_bound(&node)
        } else {
            fields.node_colors.is_bound(&node)
        }
    })
}

#[allow(clippy::too_many_arguments)]
fn add_element(
    builder: &MeshVSNodalColorPrsBuilder,
    source: &dyn MeshVSDataSource,
    key: i32,
    coords: &mut Array1<'_, f64>,
    settings: &NodalSettings,
    (triangles, edges): (
        &Handle<Graphic3dArrayOfTriangles>,
        &Handle<Graphic3dArrayOfSegments>,
    ),
    ids_to_exclude: &mut MeshVSPackedMap,
) -> Result<(), Failure> {
    let (mut nb_nodes, mut kind) = (0, MESHVS_ET_NONE);
    if !source.get_geom(key, true, coords, &mut nb_nodes, &mut kind) {
        return Ok(());
    }
    let mut nodes =
        Array1::<i32>::with_bounds(1, nb_nodes).map_err(|e| e.into_failure("MeshVS node ids"))?;
    if !source.get_nodes_by_element(key, &mut nodes, &mut nb_nodes)
        || !all_nodes_colored(builder, &nodes, nb_nodes)
    {
        return Ok(());
    }
    let mut normals = Handle::<HArray1<f64>>::default();
    let has_normals = settings.is_reflect
        && source.get_normals_by_element(key, settings.is_smooth, settings.max_nodes, &mut normals);
    if kind == MESHVS_ET_FACE {
        let face = ColoredFace {
            key,
            nb_nodes,
            coords,
            nodes: &nodes,
            normals: has_normals.then_some(&normals),
        };
        add_colored_face(builder, &face, settings, triangles)?;
        add_border(coords, nb_nodes, edges)?;
    } else if kind == MESHVS_ET_VOLUME {
        let mut topo = MeshVSTopologyHandle::default();
        if !source.get_3d_geom(key, &mut nb_nodes, &mut topo) {
            return Ok(());
        }
        builder.add_volume_prs(
            &topo,
            &nodes,
            coords,
            &triangles.clone().cast(),
            settings.is_reflect,
            &settings.scale,
        )?;
        builder.add_volume_prs(
            &topo,
            &nodes,
            coords,
            &edges.clone().cast(),
            settings.is_reflect,
            &settings.scale,
        )?;
    } else {
        return Ok(());
    }
    if builder.is_excluding_on() {
        ids_to_exclude.add(key);
    }
    Ok(())
}

struct ColoredFace<'a, 'c> {
    key: i32,
    nb_nodes: i32,
    coords: &'a Array1<'c, f64>,
    nodes: &'a Array1<'static, i32>,
    normals: Option<&'a Handle<HArray1<f64>>>,
}

/// The fan triangles of a face with the node colors or texels and the element normals.
fn add_colored_face(
    builder: &MeshVSNodalColorPrsBuilder,
    face: &ColoredFace<'_, '_>,
    settings: &NodalSettings,
    triangles: &Graphic3dArrayOfTriangles,
) -> Result<(), Failure> {
    let _ = face.key;
    for node in 0..face.nb_nodes - 2 {
        for sub in 0..3 {
            let at = fan_index(node, sub);
            let p = node_xyz(face.coords, at + 1)?;
            let point = Point3::new(p[0], p[1], p[2]);
            let normal = vertex_normal(face.normals, at)?;
            let node_id = face
                .nodes
                .value(at + 1)
                .copied()
                .map_err(|e| e.into_failure("MeshVS node ids"))?;
            add_colored_vertex(builder, triangles, &point, &normal, node_id, settings)?;
        }
    }
    Ok(())
}

/// The element normal of a fan vertex when long enough, else +Z.
fn vertex_normal(normals: Option<&Handle<HArray1<f64>>>, at: i32) -> Result<Direction3, Failure> {
    let default = Direction3::new(0.0, 0.0, 1.0)?;
    let Some(normals) = normals.and_then(Handle::get) else {
        return Ok(default);
    };
    let array = normals.array1();
    let value = |i: i32| {
        array
            .value(3 * at + i)
            .copied()
            .map_err(|e| e.into_failure("MeshVS normals"))
    };
    let test = [value(1)?, value(2)?, value(3)?];
    if test.iter().map(|c| c * c).sum::<f64>() > resolution() * resolution() {
        return Direction3::new(test[0], test[1], test[2]);
    }
    Ok(default)
}

fn add_colored_vertex(
    builder: &MeshVSNodalColorPrsBuilder,
    triangles: &Graphic3dArrayOfTriangles,
    point: &Point3,
    normal: &Direction3,
    node_id: i32,
    settings: &NodalSettings,
) -> Result<(), Failure> {
    // The gp_Vec normal converts to gp_Dir again in AddVertex.
    let normal = Direction3::new(normal.x(), normal.y(), normal.z())?;
    if settings.use_texture {
        let coord = *builder.fields.read().texture_coords.find(&node_id)?;
        let texel = texel(
            coord,
            settings.scale.nb_colors,
            settings.scale.nb_tex_colors,
        );
        triangles.add_vertex_pnt_normal_texel(point, &normal, &texel)?;
        return Ok(());
    }
    let mut color = Color::default();
    builder.get_color(node_id, &mut color);
    if settings.is_reflect {
        triangles.add_vertex_pnt_normal_color(
            point,
            &normal,
            scaled(&color, settings.scale.color_ratio)?,
        )?;
    } else {
        triangles.add_vertex_pnt_color(point, color)?;
    }
    Ok(())
}

/// The border segments of a face.
fn add_border(
    coords: &Array1<'_, f64>,
    nb_nodes: i32,
    edges: &Graphic3dArrayOfSegments,
) -> Result<(), Failure> {
    for node in 0..nb_nodes {
        for index in [node, (node + 1) % nb_nodes] {
            let p = node_xyz(coords, index + 1)?;
            edges.add_vertex_f64(p[0], p[1], p[2])?;
        }
    }
    Ok(())
}

/// The drawer appearance of the edges: color, type and width.
struct EdgeLook {
    color: Color,
    kind: AspectTypeOfLine,
    width: f64,
    show: bool,
}

fn edge_look(drawer: &dyn MeshVSDrawerSource) -> Result<EdgeLook, Failure> {
    let (mut interior, mut color, mut line) =
        (Color::default(), Color::default(), Color::default());
    let (mut width, mut show) = (1.0, true);
    drawer.get_color(Key::InteriorColor as i32, &mut interior);
    drawer.get_color(Key::EdgeColor as i32, &mut color);
    drawer.get_color(Key::BeamColor as i32, &mut line);
    drawer.get_double(Key::EdgeWidth as i32, &mut width);
    drawer.get_boolean(Key::ShowEdges as i32, &mut show);
    let mut kind = AspectTypeOfLine::Solid;
    let mut value = AspectTypeOfLine::Solid as i32;
    if drawer.get_integer(Key::EdgeType as i32, &mut value) {
        kind = line_type(value)?;
    }
    Ok(EdgeLook {
        color,
        kind,
        width,
        show,
    })
}

/// `(Aspect_TypeOfLine)value`: a value outside the enumeration is reported.
pub(crate) fn line_type(value: i32) -> Result<AspectTypeOfLine, Failure> {
    AspectTypeOfLine::from_native_value(value).ok_or_else(|| {
        Failure::with_type(
            "Standard_ProgramError",
            &format!("MeshVS edge type={value}: expected native Aspect_TypeOfLine range"),
            "",
        )
    })
}

/// The fill aspect: the textured shading aspect of the mesh drawer, or a white solid fill.
fn fill_aspect(
    builder: &MeshVSNodalColorPrsBuilder,
    settings: &NodalSettings,
    look: &EdgeLook,
) -> Result<Handle<Graphic3dAspectFillArea3d>, Failure> {
    let [front, back] = color_materials(settings.is_reflect);
    if !settings.use_texture {
        return Ok(Handle::new(Graphic3dAspectFillArea3d::new(
            AspectInteriorStyle::Solid,
            Color::named(NamedColor::WHITE),
            look.color,
            look.kind,
            look.width,
            front,
            back,
        )?));
    }
    let parent = builder.parent_mesh();
    let Some(drawer) = parent.get().map(|mesh| mesh.attributes()) else {
        return Ok(Handle::default());
    };
    drawer.set_shading_aspect(Handle::new(Prs3dShadingAspect::new()?));
    let aspect = drawer.shading_aspect().aspect();
    let Some(fill) = aspect.get() else {
        return Ok(Handle::default());
    };
    fill.set_front_material(front);
    fill.set_back_material(back);
    let texture = create_texture(&builder.fields.read());
    if texture.is_null() {
        return Ok(Handle::default());
    }
    fill.set_texture_map_on(true);
    fill.set_texture_map(texture.cast());
    fill.set_interior_color(Color::named(NamedColor::WHITE));
    Ok(aspect)
}

fn add_groups(
    builder: &MeshVSNodalColorPrsBuilder,
    prs: &Handle<dyn Graphic3dStructureSource>,
    drawer: &dyn MeshVSDrawerSource,
    settings: &NodalSettings,
    triangles: &Handle<Graphic3dArrayOfTriangles>,
    edges: &Handle<Graphic3dArrayOfSegments>,
) -> Result<(), Failure> {
    let look = edge_look(drawer)?;
    let aspect = fill_aspect(builder, settings, &look)?;
    let Some(fill) = aspect.get() else {
        return Ok(());
    };
    fill.set_distinguish(false);
    fill.set_draw_edges(false);
    if !settings.is_reflect {
        fill.set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    }
    let line = Graphic3dAspectLine3d::new(look.color, look.kind, look.width)?;
    let group = prs.new_group()?;
    let mut suppress = false;
    drawer.get_boolean(Key::SupressBackFaces as i32, &mut suppress);
    group.set_closed(suppress);
    group.set_primitives_aspect(&aspect.clone().cast::<dyn Graphic3dAspectsSource>());
    group.add_primitive_array(&triangles.clone().cast(), true);
    if look.show {
        let edge_group = prs.new_group()?;
        let copy = copy_fill(fill);
        copy.set_texture_map_on(false);
        edge_group.set_primitives_aspect(&Handle::new(copy).cast::<dyn Graphic3dAspectsSource>());
        edge_group.set_primitives_aspect(&Handle::new(line).cast::<dyn Graphic3dAspectsSource>());
        edge_group.add_primitive_array(&edges.clone().cast(), true);
    }
    Ok(())
}
