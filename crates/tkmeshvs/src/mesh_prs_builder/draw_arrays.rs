// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_MeshPrsBuilder::DrawArrays`: the groups of the opaque polygons, the edges, the
//! links and the transparent polygons (drawn last).
use super::MeshVSMeshPrsBuilderSource;
use crate::MeshVSDrawerAttribute as Key;
use crate::prs_builder::MeshVSPrsBuilderMembers;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::aspect::AspectTypeOfLine;
use rustycad_tkservice::graphic3d::*;

/// The arrays and flags of `DrawArrays`.
pub struct DrawArraysInput {
    /// `thePolygons`: the face triangles.
    pub polygons: Handle<Graphic3dArrayOfPrimitives>,
    /// `theLines`: the edges.
    pub lines: Handle<Graphic3dArrayOfPrimitives>,
    /// `theLinkLines`: the links.
    pub link_lines: Handle<Graphic3dArrayOfPrimitives>,
    /// `theVolumesInShad`: the volume triangles.
    pub volumes_in_shad: Handle<Graphic3dArrayOfPrimitives>,
    /// `IsPolygonsEdgesOff`.
    pub is_polygons_edges_off: bool,
    /// `IsSelected`.
    pub is_selected: bool,
}

fn has_items(array: &Handle<Graphic3dArrayOfPrimitives>) -> bool {
    array.get().is_some_and(|a| a.item_number() > 0)
}

/// A copy of a fill aspect (the native copy constructor shares the pointed attributes).
pub(crate) fn copy_fill(fill: &Graphic3dAspectFillArea3d) -> Graphic3dAspectFillArea3d {
    let copy = Graphic3dAspectFillArea3d::default();
    *copy.aspects_base().fields.write() = fill.aspects_base().fields.read().clone();
    copy
}

/// The native `MeshVS_MeshPrsBuilder::DrawArrays` body; nothing without fill aspect.
/// Example: `meshvs_mesh_prs_draw_arrays(&*builder, &prs, &input, &fill, &line)?`.
pub fn meshvs_mesh_prs_draw_arrays<B: MeshVSMeshPrsBuilderSource + ?Sized>(
    builder: &B,
    prs: &Handle<dyn Graphic3dStructureSource>,
    input: &DrawArraysInput,
    fill: &Handle<Graphic3dAspectFillArea3d>,
    line: &Handle<Graphic3dAspectLine3d>,
) -> Result<(), Failure> {
    let Some(fill_value) = fill.get() else {
        return Ok(());
    };
    let is_polygons = has_items(&input.polygons) || has_items(&input.volumes_in_shad);
    let edge_color = fill_value.edge_color();
    let width = f64::from(fill_value.line_width());
    let mut suppress_back_faces = false;
    let drawer = builder.get_drawer();
    if let Some(drawer) = drawer.get() {
        drawer.get_boolean(Key::SupressBackFaces as i32, &mut suppress_back_faces);
    }
    let transparent = fill_value.front_material().transparency() >= 0.01;
    if is_polygons && !transparent {
        add_polygon_group(prs, input, fill_value, suppress_back_faces)?;
    }
    if has_items(&input.lines) && !input.is_polygons_edges_off {
        let group = prs.new_group()?;
        if input.is_selected {
            group.set_primitives_aspect(&line.clone().cast::<dyn Graphic3dAspectsSource>());
        } else {
            group.set_primitives_aspect(&fill.clone().cast::<dyn Graphic3dAspectsSource>());
            let edge_line = Graphic3dAspectLine3d::new(edge_color, AspectTypeOfLine::Solid, width)?;
            group.set_primitives_aspect(
                &Handle::new(edge_line).cast::<dyn Graphic3dAspectsSource>(),
            );
        }
        group.add_primitive_array(&input.lines, true);
    }
    if has_items(&input.link_lines) {
        let group = prs.new_group()?;
        if !input.is_selected {
            group.set_primitives_aspect(&fill.clone().cast::<dyn Graphic3dAspectsSource>());
        }
        group.set_primitives_aspect(&line.clone().cast::<dyn Graphic3dAspectsSource>());
        group.add_primitive_array(&input.link_lines, true);
    }
    if is_polygons && transparent {
        add_polygon_group(prs, input, fill_value, suppress_back_faces)?;
    }
    Ok(())
}

fn add_polygon_group(
    prs: &Handle<dyn Graphic3dStructureSource>,
    input: &DrawArraysInput,
    fill: &Graphic3dAspectFillArea3d,
    suppress_back_faces: bool,
) -> Result<(), Failure> {
    let group = prs.new_group()?;
    group.set_closed(suppress_back_faces);
    let aspect = copy_fill(fill);
    aspect.set_draw_edges(false);
    aspect.set_distinguish(fill.interior_color() != fill.back_interior_color());
    group.set_primitives_aspect(&Handle::new(aspect).cast::<dyn Graphic3dAspectsSource>());
    if has_items(&input.polygons) {
        group.add_primitive_array(&input.polygons, true);
    }
    if has_items(&input.volumes_in_shad) {
        group.add_primitive_array(&input.volumes_in_shad, true);
    }
    Ok(())
}
