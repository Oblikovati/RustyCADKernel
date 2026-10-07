//! `V3d_Trihedron::compute`: the origin sphere, the axes and the labels of the trihedron.
use super::V3dTrihedron;
use super::trihedron::{
    CIRCLE_SEGMENTS_NB, CYLINDER_LENGTH, INTERNAL_SCALE_FACTOR, TrihedronState,
};
use crate::prs3d::{Prs3dArrow, Prs3dText, Prs3dToolSphere};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkmath::gp::{Axis1, Point3, Transform3, ox, oy, oz};
use rustycad_tkservice::graphic3d::{
    Graphic3dArrayOfPolylines, Graphic3dArrayOfSegments, Graphic3dGroup,
    Graphic3dGroupMembers as _, Graphic3dStructureDisplay as _, Graphic3dStructureMembers as _,
    Graphic3dStructureSource,
};
use std::f64::consts::PI;

impl V3dTrihedron {
    /// `compute`: rebuilds the origin, the three axes and the labels.
    pub(super) fn compute(&self) -> Result<(), Failure> {
        let state = {
            let mut state = self.state.write();
            state.to_compute = false;
            state.clone()
        };
        let Some(structure) = state.structure.get() else {
            return Ok(());
        };
        structure.graphic_clear(false);
        let geometry = TrihedronGeometry::of(&state);
        let mut group_iter = 0usize;
        self.compute_origin(&state.structure, &mut group_iter, &geometry, &state)?;
        self.compute_axes(&state.structure, &mut group_iter, &geometry, &state)?;
        let label_group = add_group(&state.structure, &mut group_iter)?;
        let r = geometry.rayon;
        let points = [
            Point3::new(geometry.scale + 2.0 * r, 0.0, -r),
            Point3::new(r, geometry.scale + 3.0 * r, 2.0 * r),
            Point3::new(-2.0 * r, 0.5 * r, geometry.scale + 3.0 * r),
        ];
        for ((label, aspect), point) in state.labels.iter().zip(&self.text_aspects).zip(&points) {
            let label = ExtendedString::new(&String::from_utf8_lossy(label.as_bytes()));
            Prs3dText::draw(&label_group, aspect, &label, point);
        }
        Ok(())
    }
    fn compute_origin(
        &self,
        structure: &Handle<dyn Graphic3dStructureSource>,
        group_iter: &mut usize,
        g: &TrihedronGeometry,
        state: &TrihedronState,
    ) -> Result<(), Failure> {
        let group = add_group(structure, group_iter)?;
        let Some(sphere_group) = group.get() else {
            return Ok(());
        };
        sphere_group.set_closed(!state.is_wireframe);
        let aspect = self.sphere_shading_aspect.aspect().cast();
        if state.is_wireframe {
            let circle = Graphic3dArrayOfPolylines::new(CIRCLE_SEGMENTS_NB + 2, 0, 0, 0)?;
            let segment = 2.0 * PI / f64::from(CIRCLE_SEGMENTS_NB);
            for index in (0..=CIRCLE_SEGMENTS_NB).rev() {
                let angle = f64::from(index) * segment;
                circle.add_vertex_f64(g.rayon * angle.sin(), g.rayon * angle.cos(), 0.0)?;
            }
            let last = f64::from(CIRCLE_SEGMENTS_NB) * segment;
            circle.add_vertex_f64(g.rayon * last.sin(), g.rayon * last.cos(), 0.0)?;
            sphere_group.set_group_primitives_aspect(&aspect);
            sphere_group.add_primitive_array(&Handle::new(circle).cast(), true);
        } else {
            let sphere = Prs3dToolSphere::create(
                g.sphere_radius,
                state.nb_facettes,
                state.nb_facettes,
                &Transform3::default(),
            )?;
            sphere_group.set_group_primitives_aspect(&aspect);
            sphere_group.add_primitive_array(&sphere.cast(), true);
        }
        Ok(())
    }
    fn compute_axes(
        &self,
        structure: &Handle<dyn Graphic3dStructureSource>,
        group_iter: &mut usize,
        g: &TrihedronGeometry,
        state: &TrihedronState,
    ) -> Result<(), Failure> {
        let axes: [Axis1; 3] = [ox(), oy(), oz()];
        for (index, axis) in axes.iter().enumerate() {
            let group = add_group(structure, group_iter)?;
            let Some(axis_group) = group.get() else {
                continue;
            };
            axis_group.set_closed(!state.is_wireframe);
            let aspect = self.arrow_shading_aspects[index].aspect().cast();
            if state.is_wireframe {
                let array = Graphic3dArrayOfSegments::new(2, 0, 0)?;
                array.add_vertex(0.0, 0.0, 0.0)?;
                let end = axis.direction().xyz() * g.cylinder_length;
                array.add_vertex_f64(end.x(), end.y(), end.z())?;
                axis_group.set_group_primitives_aspect(&aspect);
                axis_group.add_primitive_array(&Handle::new(array).cast(), true);
            }
            let tube = if state.is_wireframe {
                0.0
            } else {
                g.cylinder_radius
            };
            let triangles = Prs3dArrow::draw_shaded(
                axis,
                tube,
                g.cylinder_length + g.cone_length,
                g.cone_radius,
                g.cone_length,
                state.nb_facettes,
            )?;
            axis_group.set_group_primitives_aspect(&aspect);
            axis_group.add_primitive_array(&triangles.cast(), true);
        }
        Ok(())
    }
}

/// The sizes of `compute`, in the native operation order.
struct TrihedronGeometry {
    scale: f64,
    cylinder_length: f64,
    cylinder_radius: f64,
    cone_radius: f64,
    cone_length: f64,
    sphere_radius: f64,
    rayon: f64,
}

impl TrihedronGeometry {
    fn of(state: &TrihedronState) -> Self {
        let scale = state.scale * state.ratio * INTERNAL_SCALE_FACTOR;
        let cylinder_length = scale * f64::from(CYLINDER_LENGTH);
        let cylinder_radius = scale * state.diameter;
        Self {
            scale,
            cylinder_length,
            cylinder_radius,
            cone_radius: if state.is_wireframe {
                cylinder_radius
            } else {
                cylinder_radius * 2.0
            },
            cone_length: scale * (1.0 - f64::from(CYLINDER_LENGTH)),
            sphere_radius: cylinder_radius * 2.0,
            rayon: scale / 30.0,
        }
    }
}

/// `addGroup`: the existing group at the iterator, or a new one.
fn add_group(
    structure: &Handle<dyn Graphic3dStructureSource>,
    group_iter: &mut usize,
) -> Result<Handle<dyn Graphic3dGroup>, Failure> {
    let Some(value) = structure.get() else {
        return Ok(Handle::default());
    };
    let groups = value.groups();
    let index = *group_iter;
    *group_iter += 1;
    if let Some(group) = groups.get(index) {
        return Ok(group.clone());
    }
    value.new_group()
}
