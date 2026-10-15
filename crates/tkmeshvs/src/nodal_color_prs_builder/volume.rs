// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_NodalColorPrsBuilder::AddVolumePrs`: the colored fan triangles of the faces of a
//! volume (an array of triangles), else its edges once each.
use super::MeshVSNodalColorPrsBuilder;
use crate::MeshVSTool;
use crate::data_source::MeshVSTopologyHandle;
use crate::data_source::normals::{coord, face_polygon};
use crate::mesh_prs_builder::{fan_index, volume_edge_map};
use rustycad_tkernel::ncollection::{Array1, Sequence};
use rustycad_tkernel::quantity::{Color, ColorSpace};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Direction3, Point2, Point3, Vector3};
use rustycad_tkservice::graphic3d::{Graphic3dArrayOfPrimitives, Graphic3dArrayOfTriangles};

/// The color parameters of a volume: texture colors and the ratio of the reflected colors.
pub struct MeshVSNodalColorScale {
    /// `theNbColors`.
    pub nb_colors: i32,
    /// `theNbTexColors`.
    pub nb_tex_colors: i32,
    /// `theColorRatio`.
    pub color_ratio: f64,
}

/// The texel of a texture coordinate: the color-map coordinate shrunk to the texel centers,
/// and the valid (0.75) or invalid (0.25) row.
pub(super) fn texel(coord: f64, nb_colors: i32, nb_tex_colors: i32) -> Point2 {
    let row = if !(0.0..=1.0).contains(&coord) {
        0.25
    } else {
        0.75
    };
    Point2::new(
        (coord * (f64::from(nb_colors) - 1.0) + 0.5) / f64::from(nb_tex_colors),
        row,
    )
}

/// A color scaled by the ratio (`Quantity_Color::SetValues` in RGB).
pub(super) fn scaled(color: &Color, ratio: f64) -> Result<Color, Failure> {
    Color::new(
        ratio * color.red(),
        ratio * color.green(),
        ratio * color.blue(),
        ColorSpace::Rgb,
    )
}

impl MeshVSNodalColorPrsBuilder {
    /// `AddVolumePrs`: `nodes` are the element node ids (one-based), the face indices are
    /// zero-based into them and into `coords`.
    #[allow(clippy::too_many_arguments)]
    pub fn add_volume_prs(
        &self,
        topo: &MeshVSTopologyHandle,
        nodes: &Array1<'_, i32>,
        coords: &Array1<'_, f64>,
        array: &Handle<Graphic3dArrayOfPrimitives>,
        is_shaded: bool,
        scale: &MeshVSNodalColorScale,
    ) -> Result<(), Failure> {
        let (Some(faces), Some(target)) = (topo.get(), array.get()) else {
            return Ok(());
        };
        let faces = faces.array1();
        if array.cast::<Graphic3dArrayOfTriangles>().is_null() {
            return add_edges(faces.iter(), coords, target);
        }
        for face in faces.iter() {
            self.add_colored_face(face, nodes, coords, target, is_shaded, scale)?;
        }
        Ok(())
    }
    fn add_colored_face(
        &self,
        face: &Sequence<i32>,
        nodes: &Array1<'_, i32>,
        coords: &Array1<'_, f64>,
        array: &Graphic3dArrayOfPrimitives,
        is_shaded: bool,
        scale: &MeshVSNodalColorScale,
    ) -> Result<(), Failure> {
        let indices: Vec<i32> = face.iter().copied().collect();
        let poly = face_polygon(indices.iter().copied(), coords)?;
        let mut norm = Vector3::new(0.0, 0.0, 1.0);
        if is_shaded && !MeshVSTool::get_average_normal(&poly, &mut norm)? {
            norm = Vector3::new(0.0, 0.0, 1.0);
        }
        let count = indices.len() as i32;
        for node in 0..count - 2 {
            for sub in 0..3 {
                let at = fan_index(node, sub);
                let point = Point3::new(poly[3 * at + 1], poly[3 * at + 2], poly[3 * at + 3]);
                let node_id = nodes
                    .value(indices[at as usize] + 1)
                    .copied()
                    .map_err(|e| e.into_failure("MeshVS node ids"))?;
                self.add_volume_vertex(array, &point, &norm, node_id, is_shaded, scale)?;
            }
        }
        Ok(())
    }
    fn add_volume_vertex(
        &self,
        array: &Graphic3dArrayOfPrimitives,
        point: &Point3,
        norm: &Vector3,
        node_id: i32,
        is_shaded: bool,
        scale: &MeshVSNodalColorScale,
    ) -> Result<(), Failure> {
        let fields = self.fields.read();
        if fields.use_texture {
            let coord = *fields.texture_coords.find(&node_id)?;
            drop(fields);
            let dir = Direction3::new(norm.x(), norm.y(), norm.z())?;
            array.add_vertex_pnt_normal_texel(
                point,
                &dir,
                &texel(coord, scale.nb_colors, scale.nb_tex_colors),
            )?;
            return Ok(());
        }
        drop(fields);
        let mut color = Color::default();
        self.get_color(node_id, &mut color);
        if is_shaded {
            let dir = Direction3::new(norm.x(), norm.y(), norm.z())?;
            array.add_vertex_pnt_normal_color(point, &dir, scaled(&color, scale.color_ratio)?)?;
        } else {
            array.add_vertex_pnt_color(point, color)?;
        }
        Ok(())
    }
}

/// Every edge of the faces once, in the native map order, unshrunk.
fn add_edges<'a>(
    faces: impl Iterator<Item = &'a Sequence<i32>>,
    coords: &Array1<'_, f64>,
    array: &Graphic3dArrayOfPrimitives,
) -> Result<(), Failure> {
    let low = coords.lower();
    for &(first, second) in volume_edge_map(faces).iter() {
        for index in [first, second] {
            let at = low + 3 * index;
            array.add_vertex_f64(
                coord(coords, at)?,
                coord(coords, at + 1)?,
                coord(coords, at + 2)?,
            )?;
        }
    }
    Ok(())
}
