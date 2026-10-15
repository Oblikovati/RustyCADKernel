// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_VectorPrsBuilder::Build`, `DrawVector` and `calculateArrow`.
use super::MeshVSVectorPrsBuilder;
use crate::MeshVSDrawerAttribute as Key;
use crate::data_source::MeshVSPackedMap;
use crate::flags::*;
use crate::mesh_prs_builder::node_xyz;
use crate::prs_builder::{
    MeshVSPrsBuilder, MeshVSPrsBuilderMembers, coords_of, parent_hidden_elems, remaining_ids,
};
use crate::text_prs_builder::element_mean;
use rustycad_tkernel::precision::confusion;
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Axis3, Direction3, Point3, Transform3, Vector3, Xyz};
use rustycad_tkservice::aspect::AspectTypeOfLine;
use rustycad_tkservice::graphic3d::*;

/// `calculateArrow`: the eight points of an arrow head of height `length * arrow_part` below
/// the origin, returned with that height. Example: `calculate_arrow(&mut points, 10.0, 0.1)`.
pub fn calculate_arrow(points: &mut [Point3; 8], length: f64, arrow_part: f64) -> f64 {
    let h = length * arrow_part;
    let w = h / 5.;
    *points = [
        Point3::new(0., 0., 0.),
        Point3::new(0., -w, -h),
        Point3::new(w * 0.866, -w * 0.5, -h),
        Point3::new(w * 0.866, w * 0.5, -h),
        Point3::new(0., w, -h),
        Point3::new(-w * 0.866, w * 0.5, -h),
        Point3::new(-w * 0.866, -w * 0.5, -h),
        Point3::new(0., -w, -h),
    ];
    h
}

fn transformed(trsf: &Transform3, point: Point3) -> Point3 {
    let mut xyz = point.xyz();
    trsf.transforms(&mut xyz);
    Point3::from(xyz)
}

/// The three arrays of `Build`: lines, simple arrow lines and arrow heads.
pub(super) struct VectorArrays {
    pub lines: Handle<Graphic3dArrayOfSegments>,
    pub arrow_lines: Handle<Graphic3dArrayOfSegments>,
    pub triangles: Handle<Graphic3dArrayOfSegments>,
}

impl MeshVSVectorPrsBuilder {
    /// `DrawVector`: a line of the length (at least the simple minimum in the simple mode)
    /// along +Z moved by the transformation, then its arrow head or simple arrow line.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_vector(
        &self,
        trsf: &Transform3,
        length: f64,
        max_length: f64,
        arrow_points: &[Point3],
        lines: &Graphic3dArrayOfPrimitives,
        arrow_lines: &Graphic3dArrayOfPrimitives,
        triangles: &Graphic3dArrayOfPrimitives,
    ) -> Result<(), Failure> {
        let fields = self.fields.read();
        let (simple, start, end) = (
            fields.is_simple_prs,
            fields.simple_start_prm,
            fields.simple_end_prm,
        );
        drop(fields);
        let min_length = max_length * (1.0 - start);
        let local = if !simple || length > min_length {
            length
        } else {
            min_length
        };
        lines.add_vertex_pnt(&transformed(trsf, Point3::new(0., 0., 0.)))?;
        lines.add_vertex_pnt(&transformed(trsf, Point3::new(0., 0., local)))?;
        if simple {
            let end_pos = local - max_length * (1.0 - end);
            let arrow_length = max_length * (end - start);
            arrow_lines.add_vertex_pnt(&transformed(
                trsf,
                Point3::new(0., 0., end_pos - arrow_length),
            ))?;
            arrow_lines.add_vertex_pnt(&transformed(trsf, Point3::new(0., 0., end_pos)))?;
            return Ok(());
        }
        if arrow_points.len() < 8 {
            return Ok(());
        }
        let offset = triangles.vertex_number() + 1;
        for point in &arrow_points[..8] {
            let lifted = Point3::from(point.xyz() + Xyz::new(0., 0., local));
            triangles.add_vertex_pnt(&transformed(trsf, lifted))?;
        }
        for index in 1..=6 {
            for vertex in [offset, offset + index, offset + index, offset + index + 1] {
                triangles.add_edge(vertex)?;
            }
        }
        Ok(())
    }
}

/// The length scale `k` and offset `b` of the magnitudes.
fn length_scale(min_value: f64, max_value: f64, max_len: f64) -> (f64, f64) {
    if max_value - min_value > confusion() {
        let k = 0.8 * max_len / (max_value - min_value);
        return (k, max_len - k * max_value);
    }
    (0.0, max_len)
}

/// `MeshVS_DA_MaxFaceNodes`, `MeshVS_DA_VectorMaxLength` and `MeshVS_DA_VectorArrowPart`, all positive.
fn vector_settings(drawer: &dyn crate::drawer::MeshVSDrawerSource) -> Option<(i32, f64, f64)> {
    let (mut max_nodes, mut max_len, mut arrow_part) = (0, 0.0, 0.1);
    let ok = drawer.get_integer(Key::MaxFaceNodes as i32, &mut max_nodes)
        && max_nodes > 0
        && drawer.get_double(Key::VectorMaxLength as i32, &mut max_len)
        && max_len > 0.0
        && drawer.get_double(Key::VectorArrowPart as i32, &mut arrow_part)
        && arrow_part > 0.0;
    ok.then_some((max_nodes, max_len, arrow_part))
}

/// The native `MeshVS_VectorPrsBuilder::Build` body.
pub(super) fn build_vectors(
    builder: &MeshVSVectorPrsBuilder,
    prs: &Handle<dyn Graphic3dStructureSource>,
    ids: &MeshVSPackedMap,
    ids_to_exclude: &mut MeshVSPackedMap,
    is_element: bool,
    display_mode: i32,
) -> Result<(), Failure> {
    let (drawer, source) = (builder.get_drawer(), builder.get_data_source());
    let (Some(drawer), Some(source)) = (drawer.get(), source.get()) else {
        return Ok(());
    };
    if !builder.has_vectors(is_element) || (display_mode & builder.get_flags()) == 0 {
        return Ok(());
    }
    let Some((max_nodes, max_len, arrow_part)) = vector_settings(drawer) else {
        return Ok(());
    };
    let mut coords = coords_of(max_nodes)?;
    let nb_vectors = builder.fields.read().of(is_element).extent() as i32;
    if nb_vectors <= 0 {
        return Ok(());
    }
    let arrays = VectorArrays {
        lines: Handle::new(Graphic3dArrayOfSegments::new(nb_vectors * 2, 0, 0)?),
        arrow_lines: Handle::new(Graphic3dArrayOfSegments::new(nb_vectors * 2, 0, 0)?),
        triangles: Handle::new(Graphic3dArrayOfSegments::new(
            nb_vectors * 8,
            nb_vectors * 12 * 2,
            0,
        )?),
    };
    let mut arrow = [Point3::default(); 8];
    let min_length = calculate_arrow(&mut arrow, max_len, arrow_part);
    let (mut min_value, mut max_value) = (0.0, 0.0);
    builder.get_min_max_vector_value(is_element, &mut min_value, &mut max_value);
    let (k, b) = length_scale(min_value, max_value, max_len);
    let hidden = if is_element {
        parent_hidden_elems(builder)
    } else {
        Handle::default()
    };
    let remaining = remaining_ids(ids, &hidden, ids_to_exclude);
    let mut custom = MeshVSPackedMap::default();
    let from = Axis3::from_normal(Point3::new(0., 0., 0.), Direction3::new(0., 0., 1.)?)?;
    for key in remaining.iter() {
        let mut vector = Vector3::new(0., 0., 0.);
        if !builder.get_vector(is_element, key, &mut vector) {
            continue;
        }
        let value = vector.magnitude();
        if value.abs() < confusion() {
            continue;
        }
        let (mut nb_nodes, mut kind) = (0, MESHVS_ET_NONE);
        if !source.get_geom(key, is_element, &mut coords, &mut nb_nodes, &mut kind) {
            continue;
        }
        let position = match kind {
            MESHVS_ET_NODE => node_xyz(&coords, 1)?,
            MESHVS_ET_LINK | MESHVS_ET_FACE | MESHVS_ET_VOLUME => {
                if is_element && builder.is_excluding_on() {
                    ids_to_exclude.add(key);
                }
                element_mean(&coords, nb_nodes)?
            }
            _ => {
                custom.add(key);
                continue;
            }
        };
        let to = Axis3::from_normal(
            Point3::new(position[0], position[1], position[2]),
            Direction3::new(vector.x(), vector.y(), vector.z())?,
        )?;
        let mut trsf = Transform3::default();
        trsf.set_displacement(from, to);
        let length = (k * value.abs() + b).max(min_length);
        builder.draw_vector(
            &trsf,
            length,
            max_len,
            &arrow,
            &arrays.lines,
            &arrays.arrow_lines,
            &arrays.triangles,
        )?;
    }
    add_vector_group(builder, prs, drawer, &arrays)?;
    if !custom.is_empty() {
        builder.custom_build(prs, &custom, ids_to_exclude, display_mode)?;
    }
    Ok(())
}

fn line_aspect(color: Color, width: f64) -> Result<Handle<dyn Graphic3dAspectsSource>, Failure> {
    Ok(Handle::new(Graphic3dAspectLine3d::new(
        color,
        AspectTypeOfLine::Solid,
        width,
    )?)
    .cast())
}

fn add_vector_group(
    builder: &MeshVSVectorPrsBuilder,
    prs: &Handle<dyn Graphic3dStructureSource>,
    drawer: &dyn crate::drawer::MeshVSDrawerSource,
    arrays: &VectorArrays,
) -> Result<(), Failure> {
    let group = prs.new_group()?;
    let mut color = Color::default();
    drawer.get_color(Key::VectorColor as i32, &mut color);
    group.set_primitives_aspect(&line_aspect(color, 1.5)?);
    group.add_primitive_array(&arrays.lines.clone().cast(), true);
    let (simple, width) = {
        let fields = builder.fields.read();
        (fields.is_simple_prs, fields.simple_width_prm)
    };
    if !simple {
        group.set_primitives_aspect(&line_aspect(color, width)?);
        group.add_primitive_array(&arrays.triangles.clone().cast(), true);
    } else {
        group.set_primitives_aspect(&line_aspect(color, width * 1.5)?);
        group.add_primitive_array(&arrays.arrow_lines.clone().cast(), true);
    }
    Ok(())
}
