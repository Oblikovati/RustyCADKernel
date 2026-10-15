// SPDX-License-Identifier: GPL-2.0-only
//! The two-color faces of `MeshVS_ElementalColorPrsBuilder::Build`: one distinguished fill
//! aspect (front and back interior colors) per color pair.
use super::build::{ElementalGroups, ElementalScope};
use crate::data_source::MeshVSPackedMap;
use crate::flags::MESHVS_ET_FACE;
use crate::prs_builder::MeshVSPrsBuilderMembers;
use crate::two_colors::{MeshVSTwoColors, extract_colors};
use rustycad_tkernel::ncollection::{DataMap, Map, Vec3};
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::aspect::AspectInteriorStyle;
use rustycad_tkservice::graphic3d::*;

/// The two plastic materials of the color pairs: black specular and emissive colors, and a
/// "nondirectional" full ambient color without diffusion when not reflected (so the front
/// and back materials still differ), half ambient and diffuse colors when reflected.
pub(super) fn pair_materials(is_reflect: bool) -> [Graphic3dMaterialAspect; 2] {
    std::array::from_fn(|_| {
        let mut material = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Plastified);
        material.set_specular_color(Color::named(NamedColor::BLACK));
        material.set_emissive_color(Color::named(NamedColor::BLACK));
        let (ambient, diffuse) = if is_reflect {
            let half = Color::from_vec3(Vec3::new(0.5, 0.5, 0.5)).expect("valid gray");
            (half, half)
        } else {
            (
                Color::from_vec3(Vec3::new(1.0, 1.0, 1.0)).expect("valid white"),
                Color::named(NamedColor::BLACK),
            )
        };
        material.set_ambient_color(ambient);
        material.set_diffuse_color(diffuse);
        material
    })
}

/// The pair group aspect, then the faces of every color pair.
pub(super) fn add_pair_groups(
    scope: &mut ElementalScope<'_>,
    groups: &ElementalGroups,
    pairs: &DataMap<MeshVSTwoColors, Map<i32>>,
    ids_to_exclude: &mut MeshVSPackedMap,
) -> Result<(), Failure> {
    if pairs.is_empty() {
        return Ok(());
    }
    let look = &scope.look;
    let [front, back] = pair_materials(look.is_reflect);
    let fill = Graphic3dAspectFillArea3d::new(
        AspectInteriorStyle::Solid,
        look.interior,
        look.edge,
        look.edge_type,
        look.edge_width,
        front,
        back,
    )?;
    let pair_group = groups.pairs.get().expect("pair group");
    pair_group.set_closed(false);
    pair_group.set_group_primitives_aspect(&Handle::new(fill).cast::<dyn Graphic3dAspectsSource>());
    for (pair, elements) in pairs.items() {
        if !elements.is_empty() {
            add_pair_group(scope, groups, *pair, elements, ids_to_exclude)?;
        }
    }
    Ok(())
}

/// [faces, edges] counts of the faces of a pair.
fn count_pair(scope: &mut ElementalScope<'_>, elements: &Map<i32>) -> [i32; 2] {
    let mut counts = [0; 2];
    let ids: Vec<i32> = scope.ids.iter().filter(|k| elements.contains(k)).collect();
    for key in ids {
        if let Some((nb_nodes, MESHVS_ET_FACE)) = scope.geom(key).filter(|(n, _)| *n > 0) {
            counts[1] += if scope.look.edge_on { nb_nodes } else { 0 };
            counts[0] += nb_nodes - 2;
        }
    }
    counts
}

fn add_pair_group(
    scope: &mut ElementalScope<'_>,
    groups: &ElementalGroups,
    mut pair: MeshVSTwoColors,
    elements: &Map<i32>,
    ids_to_exclude: &mut MeshVSPackedMap,
) -> Result<(), Failure> {
    let [faces, edges] = count_pair(scope, elements);
    let look = &scope.look;
    let triangles = Handle::new(Graphic3dArrayOfTriangles::with_options(
        faces * 3,
        0,
        look.is_reflect,
        false,
        false,
    )?);
    let edge_segments = Handle::new(Graphic3dArrayOfSegments::new(edges * 2, 0, 0)?);
    let (mut interior, mut back_color) = (Color::default(), Color::default());
    extract_colors(&mut pair, &mut interior, &mut back_color);
    let [front, back] = pair_materials(look.is_reflect);
    let aspect = Graphic3dAspectFillArea3d::new(
        AspectInteriorStyle::Solid,
        interior,
        look.edge,
        look.edge_type,
        look.edge_width,
        front,
        back,
    )?;
    aspect.set_distinguish(true);
    aspect.set_interior_color(interior);
    aspect.set_back_interior_color(back_color);
    let pair_group = groups.pairs.get().expect("pair group");
    pair_group.set_primitives_aspect(&Handle::new(aspect).cast::<dyn Graphic3dAspectsSource>());
    let ids: Vec<i32> = scope.ids.iter().filter(|k| elements.contains(k)).collect();
    for key in ids {
        let Some((nb_nodes, kind)) = scope.geom(key) else {
            continue;
        };
        if scope.builder.is_excluding_on() {
            ids_to_exclude.add(key);
        }
        if kind != MESHVS_ET_FACE || nb_nodes <= 0 {
            continue;
        }
        // The normals are always asked (OCC21720), and used only when reflected.
        let normals = scope.normals(key, true);
        scope.add_face_triangles(&triangles, nb_nodes, normals.as_ref())?;
        if scope.look.edge_on {
            scope.add_face_edges(&edge_segments, nb_nodes)?;
        }
    }
    pair_group.add_primitive_array(&triangles.cast(), true);
    if let Some(edges_group) = groups.edges.get().filter(|_| scope.look.edge_on) {
        edges_group.add_primitive_array(&edge_segments.cast(), true);
    }
    Ok(())
}
