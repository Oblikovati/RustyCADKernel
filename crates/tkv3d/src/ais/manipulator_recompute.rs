//! `AIS_Manipulator::RecomputeTransformation`: in the flat skin, the 2D parts turned around
//! their axes to face the camera, the outer circle turned toward it, and the selections of
//! the turned parts recomputed.
use super::*;
use crate::prs_mgr::PrsMgrPresentableObject;
use crate::prs3d::Prs3dShadingAspect;
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{self, Axis1, Axis3, Direction3, Transform3, Vector3};
use rustycad_tkservice::graphic3d::*;
use std::f64::consts::FRAC_PI_2;

/// The parts turned by a flat recomputation.
#[derive(Default)]
struct Recomputed {
    translation: bool,
    rotation: bool,
    dragging: bool,
    scaling: bool,
}

fn null_part() -> Failure {
    Failure::with_message("Standard_NullObject", "AIS_Manipulator: part not computed")
}

fn group_of(group: &Handle<dyn Graphic3dGroup>) -> Result<&dyn Graphic3dGroup, Failure> {
    group.get().ok_or_else(null_part)
}

fn highlight_group(
    prs: &Handle<dyn Graphic3dStructureSource>,
) -> Result<Handle<dyn Graphic3dGroup>, Failure> {
    prs.get().ok_or_else(null_part)?.current_group()
}

impl AisManipulator {
    /// The flat-skin body of `RecomputeTransformation`.
    pub(super) fn recompute_flat_transformation(
        &self,
        camera: &Handle<Graphic3dCamera>,
    ) -> Result<(), Failure> {
        if self.fields.read().skin_mode == AisManipulatorSkin::Shaded {
            return Ok(());
        }
        let camera_dir = camera.get().ok_or_else(null_part)?.direction();
        let mut done = Recomputed::default();
        let axes = self.fields.read().axes.clone();
        for axis in axes.iter().filter(|a| a.has_dragging) {
            group_of(&axis.dragger_group)?.set_transformation(&Transform3::default());
            done.dragging = true;
        }
        for index in 0..3 {
            self.turn_axis(&axes, index, &camera_dir, &mut done)?;
        }
        if done.rotation {
            self.turn_circle(&camera_dir)?;
        }
        if done.dragging {
            for axis in &axes {
                let transformation = group_of(&axis.dragger_group)?.transformation();
                highlight_group(&axis.highlight_dragger)?.set_transformation(&transformation);
            }
        }
        let modes = [
            (done.translation, AisManipulatorMode::Translation),
            (done.rotation, AisManipulatorMode::Rotation),
            (done.dragging, AisManipulatorMode::TranslationPlane),
            (done.scaling, AisManipulatorMode::Scaling),
        ];
        for (recomputed, mode) in modes {
            if recomputed {
                self.recompute_selection(mode)?;
            }
        }
        let object = self.object()?;
        let context = object.get().ok_or_else(null_part)?.get_context();
        let context = context.get().ok_or_else(|| {
            Failure::with_message("Standard_NullObject", "AIS_Manipulator: no context")
        })?;
        context.recompute_selection_only(&self.presentable_base().this().cast())
    }

    /// The axis direction and the normal its parts turn from.
    fn axis_frame(&self, reference: &Axis1) -> Result<(Direction3, Direction3), Failure> {
        let position = self.fields.read().position;
        let (x, y) = (position.x_direction(), position.y_direction());
        let direction = reference.direction();
        if direction.x() > 0.0 {
            return Ok((y.reversed(), x));
        }
        if direction.y() > 0.0 {
            return Ok((x.crossed(y)?.reversed(), y));
        }
        Ok((x.reversed(), x.crossed(y)?))
    }

    fn turn_axis(
        &self,
        axes: &[AisManipulatorAxis; 3],
        index: usize,
        camera_dir: &Direction3,
        done: &mut Recomputed,
    ) -> Result<(), Failure> {
        let axis = &axes[index];
        let reference = axis.reference_axis;
        let (normal, axis_dir) = self.axis_frame(&reference)?;
        let camera_proj = if (axis_dir.dot(*camera_dir).abs() - 1.0).abs() <= gp::resolution() {
            *camera_dir
        } else {
            axis_dir.crossed(*camera_dir)?.crossed(axis_dir)?
        };
        let is_reversed = axis_dir.dot(*camera_dir) > 0.0;
        let mut angle = normal.angle_with_ref(camera_proj, axis_dir);
        if reference.direction().x() > 0.0 {
            angle -= FRAC_PI_2;
        }
        let reversed_dir = reference.direction().xyz().reversed();
        if axis.has_translation {
            self.turn_translator(axis, angle, is_reversed)?;
            done.translation = true;
        }
        if axis.has_rotation {
            let mut trsf = Transform3::default();
            trsf.set_rotation(reference, angle - FRAC_PI_2)?;
            group_of(&axis.rotator_group)?.set_transformation(&trsf);
            highlight_group(&axis.highlight_rotator)?.set_transformation(&trsf);
            done.rotation = true;
        }
        if axis.has_dragging && is_reversed {
            for (other_index, other) in axes.iter().enumerate() {
                let translation = if other_index == index {
                    reference.direction().xyz() * f64::from(other.axis_radius * 2.0)
                } else {
                    reversed_dir * f64::from(other.length)
                };
                let mut trsf = Transform3::default();
                trsf.set_translation(Vector3::from(translation));
                let group = group_of(&other.dragger_group)?;
                trsf *= group.transformation();
                group.set_transformation(&trsf);
            }
        }
        if axis.has_scaling {
            if reference.direction().x() > 0.0 {
                angle += FRAC_PI_2;
            }
            let mut trsf = Transform3::default();
            trsf.set_rotation(reference, angle)?;
            if is_reversed {
                let length = axis.length * 2.0 + axis.box_size + axis.indent * 4.0;
                trsf.set_translation_part(Vector3::from(reversed_dir * f64::from(length)));
            }
            group_of(&axis.scaler_group)?.set_transformation(&trsf);
            highlight_group(&axis.highlight_scaler)?.set_transformation(&trsf);
            done.scaling = true;
        }
        Ok(())
    }

    fn turn_translator(
        &self,
        axis: &AisManipulatorAxis,
        angle: f64,
        is_reversed: bool,
    ) -> Result<(), Failure> {
        let aspect = Prs3dShadingAspect::new()?;
        aspect
            .aspect()
            .set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
        let color = if is_reversed {
            let [r, g, b] = axis.color.rgb();
            Color::from_vec3(rustycad_tkernel::ncollection::Vec3::new(
                r * 0.1,
                g * 0.1,
                b * 0.1,
            ))?
        } else {
            axis.color
        };
        aspect.aspect().set_interior_color(color);
        let reference = axis.reference_axis;
        let mut trsf = Transform3::default();
        trsf.set_rotation(reference, angle)?;
        if is_reversed {
            let length = f64::from(axis.length + axis.indent * 4.0);
            trsf.set_translation_part(Vector3::from(
                reference.direction().xyz().reversed() * length,
            ));
        }
        let group = group_of(&axis.translator_group)?;
        group.set_group_primitives_aspect(&aspect.aspect().cast());
        group.set_transformation(&trsf);
        highlight_group(&axis.highlight_translator)?.set_transformation(&trsf);
        Ok(())
    }

    /// The outer circle turned toward the camera around the manipulator Z axis.
    fn turn_circle(&self, camera_dir: &Direction3) -> Result<(), Failure> {
        let (x_dir, y_dir, z_dir) = (gp::dx(), gp::dy(), gp::dz());
        let resolution = gp::resolution();
        let camera_projection = if x_dir.dot(*camera_dir).abs() <= resolution
            || y_dir.dot(*camera_dir).abs() <= resolution
        {
            *camera_dir
        } else {
            Direction3::from_xyz(
                x_dir.xyz() * x_dir.dot(*camera_dir) + y_dir.xyz() * y_dir.dot(*camera_dir),
            )?
        };
        let is_reversed = z_dir.dot(*camera_dir) > 0.0;
        let angle = FRAC_PI_2 - camera_dir.angle(&camera_projection);
        let mut rot_axis = if (camera_projection.dot(z_dir).abs() - 1.0).abs() <= resolution {
            z_dir
        } else {
            camera_projection.crossed(z_dir)?
        };
        if is_reversed {
            rot_axis.reverse();
        }
        let mut rotation = Transform3::default();
        rotation.set_rotation(Axis1::new(gp::origin(), rot_axis), angle)?;
        let position = self.fields.read().position;
        let to_system = Axis3::new(
            gp::origin(),
            position.x_direction().crossed(position.y_direction())?,
            position.x_direction(),
        )?;
        let mut from_system = Axis3::from_axis2(gp::xoy());
        from_system.transform(rotation);
        let mut trsf = Transform3::default();
        trsf.set_transformation_between(from_system, to_system);
        let group = self.fields.read().circle_group.clone();
        group_of(&group)?.set_transformation(&trsf);
        Ok(())
    }
}
