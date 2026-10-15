//! `AIS_Manipulator` interactive transformation: the transformation of a pick in the active
//! mode, its application to the attached objects and the manipulator, and the placement.
use super::*;
use crate::prs_mgr::*;
use crate::prs3d::Prs3dShadingAspect;
use crate::v3d::V3dViewSource;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::extrema::ExtElC;
use rustycad_tkgeombase::int_ana::IntConicQuad;
use rustycad_tkmath::gp::{
    self, Axis2, Axis3, Direction3, Line3, Plane3, Point3, Transform3, TransformForm, Vector3,
};
use rustycad_tkmath::toploc::Datum3;
use rustycad_tkservice::aspect::{AspectInteriorStyle, AspectTypeOfFacingModel};
use rustycad_tkservice::graphic3d::*;

const BOTH: AspectTypeOfFacingModel = AspectTypeOfFacingModel::BothSide;

/// `gce_MakeDir(P1, P2)`: coincident points raise `StdFail_NotDone`.
fn make_dir(from: Point3, to: Point3) -> Result<Direction3, Failure> {
    Direction3::from_xyz(to.xyz() - from.xyz())
        .map_err(|_| Failure::with_message("StdFail_NotDone", "gce_MakeDir::Value() - no result"))
}

impl AisManipulator {
    /// `ObjectTransformation`: the transformation of a pixel in the active mode, composed
    /// into `trsf` (the first call records the start state and pick and returns true).
    /// Example: `manipulator.object_transformation(x, y, &view, &mut trsf)?`.
    pub fn object_transformation(
        &self,
        x: i32,
        y: i32,
        view: &Handle<dyn V3dViewSource>,
        trsf: &mut Transform3,
    ) -> Result<bool, Failure> {
        if !self.fields.read().has_started_transformation {
            self.record_start_state()?;
        }
        let actual = view.get().ok_or_else(|| {
            Failure::with_message("Standard_NullObject", "AIS_Manipulator: null view")
        })?;
        let [px, py, pz, dx, dy, dz] = actual.convert_with_proj(x, y)?;
        let input = Line3::new(Point3::new(px, py, pz), Direction3::new(dx, dy, dz)?);
        let mode = self.fields.read().current_mode;
        match mode {
            AisManipulatorMode::Translation | AisManipulatorMode::Scaling => {
                self.linear_transformation(&input, trsf)
            }
            AisManipulatorMode::Rotation => self.rotation_transformation(&input, trsf),
            AisManipulatorMode::TranslationPlane => self.plane_transformation(&input, trsf),
            AisManipulatorMode::None => Ok(false),
        }
    }

    fn record_start_state(&self) -> Result<(), Failure> {
        let objects = self.objects();
        let sequence = objects.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "AIS_Manipulator: no attached objects",
            )
        })?;
        let starts: Vec<Transform3> = sequence.sequence(|items| {
            items
                .iter()
                .map(|object| {
                    object
                        .get()
                        .map_or_else(Transform3::default, |o| o.local_transformation())
                })
                .collect()
        });
        let mut fields = self.fields.write();
        fields.start_trsfs = starts;
        fields.start_position = fields.position;
        Ok(())
    }

    /// The start pick of the first call, else whether the pick moved.
    fn start_or_moved(&self, pick: Point3) -> Option<bool> {
        let mut fields = self.fields.write();
        if !fields.has_started_transformation {
            fields.start_pick = pick;
            fields.has_started_transformation = true;
            return Some(true);
        }
        if pick.distance(&fields.start_pick) < precision::confusion() {
            return Some(false);
        }
        None
    }

    fn linear_transformation(&self, input: &Line3, trsf: &mut Transform3) -> Result<bool, Failure> {
        let (start, mode, position, axis_dir) = {
            let f = self.fields.read();
            let index = f.current_index as usize;
            (
                f.start_position,
                f.current_mode,
                f.position,
                f.axes[index].position.direction(),
            )
        };
        let line = Line3::new(start.location(), axis_dir);
        let extrema = ExtElC::lines(*input, line, precision::angular());
        if !extrema.is_done() || extrema.is_parallel()? || extrema.nb_ext()? != 1 {
            return Ok(false);
        }
        let new_position = *extrema.points(1)?.1.value();
        if let Some(result) = self.start_or_moved(new_position) {
            return Ok(result);
        }
        let start_pick = self.fields.read().start_pick;
        let mut new_trsf = Transform3::default();
        if mode == AisManipulatorMode::Translation {
            new_trsf.set_translation(Vector3::between(start_pick, new_position));
            *trsf *= new_trsf;
            return Ok(true);
        }
        if new_position.distance(&start.location()) < precision::confusion() {
            return Ok(false);
        }
        let coeff =
            start.location().distance(&new_position) / start.location().distance(&start_pick);
        new_trsf.set_scale(position.location(), coeff)?;
        *trsf = new_trsf;
        Ok(true)
    }

    fn plane_intersection(
        &self,
        input: &Line3,
        require_not_parallel: bool,
    ) -> Result<Option<Point3>, Failure> {
        let (start, index) = {
            let f = self.fields.read();
            (f.start_position, f.current_index)
        };
        let axis = Self::axis_of(&start, index)?;
        let plane = Plane3::new(start.location(), axis.direction())?;
        let inter = IntConicQuad::line_plane(
            *input,
            plane,
            precision::angular(),
            precision::intersection(),
            0.0,
        )?;
        if !inter.is_done()
            || (require_not_parallel && inter.is_parallel()?)
            || inter.nb_points()? < 1
        {
            return Ok(None);
        }
        Ok(Some(inter.point(1)?))
    }

    fn rotation_transformation(
        &self,
        input: &Line3,
        trsf: &mut Transform3,
    ) -> Result<bool, Failure> {
        let Some(new_position) = self.plane_intersection(input, true)? else {
            return Ok(false);
        };
        let (start, index) = {
            let f = self.fields.read();
            (f.start_position, f.current_index)
        };
        let pos_loc = start.location();
        let curr_axis = Self::axis_of(&start, index)?;
        if !self.fields.read().has_started_transformation {
            let mut fields = self.fields.write();
            fields.start_pick = new_position;
            fields.has_started_transformation = true;
            let start_axis = make_dir(pos_loc, new_position)?;
            fields.prev_state =
                start_axis.angle_with_ref(make_dir(pos_loc, new_position)?, curr_axis.direction());
            return Ok(true);
        }
        let start_pick = self.fields.read().start_pick;
        if new_position.distance(&start_pick) < precision::confusion() {
            return Ok(false);
        }
        let start_axis = if pos_loc.is_equal(&start_pick, precision::confusion()) {
            Self::axis_of(&start, (index + 1) % 3)?.direction()
        } else {
            make_dir(pos_loc, start_pick)?
        };
        let current_axis = make_dir(pos_loc, new_position)?;
        let mut angle = start_axis.angle_with_ref(current_axis, curr_axis.direction());
        if angle.abs() < precision::confusion() {
            return Ok(false);
        }
        if self.fields.read().skin_mode == AisManipulatorSkin::Flat {
            self.update_flat_sector(&start, &current_axis, angle)?;
        }
        let prev_state = self.fields.read().prev_state;
        if angle * prev_state < 0.0 && angle.abs() < std::f64::consts::FRAC_PI_2 {
            let sign = if prev_state > 0.0 { -1.0 } else { 1.0 };
            angle = sign * (std::f64::consts::PI * 2.0 - angle);
        }
        let mut new_trsf = Transform3::default();
        new_trsf.set_rotation(curr_axis, angle)?;
        *trsf *= new_trsf;
        self.fields.write().prev_state = angle;
        Ok(true)
    }

    /// The flat-skin rotation sector of the swept angle.
    fn update_flat_sector(
        &self,
        start: &Axis2,
        current_axis: &Direction3,
        angle: f64,
    ) -> Result<(), Failure> {
        let (axis, color, inner_radius, material, sector_group) = {
            let f = self.fields.read();
            let a = &f.axes[f.current_index as usize];
            (
                a.reference_axis,
                a.color,
                a.inner_radius(),
                self.attributes().shading_aspect().material(BOTH),
                f.sector_group.clone(),
            )
        };
        let rotation_start = if axis.direction().z() > 0.0 {
            start.y_direction()
        } else {
            start.direction()
        };
        let curr_axis = Self::axis_of(start, self.fields.read().current_index)?;
        let mut rotation_angle =
            rotation_start.angle_with_ref(*current_axis, curr_axis.direction());
        rotation_angle -= if angle > 0.0 { angle * 2.0 } else { angle };
        if axis.direction().z() > 0.0 {
            rotation_angle += std::f64::consts::FRAC_PI_2;
        }
        let mut rotation = Transform3::default();
        rotation.set_rotation(axis, rotation_angle)?;
        let aspect = Prs3dShadingAspect::new()?;
        aspect
            .aspect()
            .set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
        aspect.set_material(&material, BOTH);
        aspect.set_transparency(0.5, BOTH)?;
        aspect.set_color(color, BOTH)?;
        self.fields
            .write()
            .sector
            .init(0.0, inner_radius, &axis, angle.abs(), 20, 20)?;
        let group = sector_group.get().ok_or_else(|| {
            Failure::with_message("Standard_NullObject", "AIS_Manipulator: no sector group")
        })?;
        group.clear(true);
        group.set_primitives_aspect(&aspect.aspect().cast());
        group.add_primitive_array(&self.fields.read().sector.array().clone().cast(), true);
        group.set_transformation(&rotation);
        Ok(())
    }

    fn plane_transformation(&self, input: &Line3, trsf: &mut Transform3) -> Result<bool, Failure> {
        let Some(new_position) = self.plane_intersection(input, false)? else {
            return Ok(false);
        };
        if let Some(result) = self.start_or_moved(new_position) {
            return Ok(result);
        }
        let start_pick = self.fields.read().start_pick;
        let mut new_trsf = Transform3::default();
        new_trsf.set_translation(Vector3::between(start_pick, new_position));
        *trsf *= new_trsf;
        Ok(true)
    }

    /// `StartTransform`: the start state of a transformation from a pixel (when not started).
    /// Example: `manipulator.start_transform(x, y, &view)?`.
    pub fn start_transform(
        &self,
        x: i32,
        y: i32,
        view: &Handle<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        if self.fields.read().has_started_transformation {
            return Ok(());
        }
        let mut trsf = Transform3::default();
        self.object_transformation(x, y, view, &mut trsf)?;
        Ok(())
    }

    /// `StopTransform`: ends the transformation, restoring the start state unless applied.
    /// Example: `manipulator.stop_transform(false)?`.
    pub fn stop_transform(&self, to_apply: bool) -> Result<(), Failure> {
        if !self.is_attached() || !self.fields.read().has_started_transformation {
            return Ok(());
        }
        self.fields.write().has_started_transformation = false;
        if to_apply {
            return Ok(());
        }
        let objects = self.objects();
        let items = objects
            .get()
            .map(|s| s.sequence(|i| i.iter().cloned().collect::<Vec<_>>()))
            .unwrap_or_default();
        let (starts, start_position) = {
            let f = self.fields.read();
            (f.start_trsfs.clone(), f.start_position)
        };
        for (object, start) in items.iter().zip(starts.iter()) {
            if let Some(object) = object.get() {
                object.set_local_transformation(start)?;
            }
        }
        self.set_position(&start_position)
    }

    /// `Transform(trsf)`: applies a transformation from the start state to the attached
    /// objects (moving the anchors of persistent ones), and to the manipulator when it follows
    /// the active mode; `manipulator.transform(&trsf)?`.
    pub fn transform(&self, trsf: &Transform3) -> Result<(), Failure> {
        if !self.is_attached() || !self.fields.read().has_started_transformation {
            return Ok(());
        }
        let objects = self.objects();
        let items = objects
            .get()
            .map(|s| s.sequence(|i| i.iter().cloned().collect::<Vec<_>>()))
            .unwrap_or_default();
        let (starts, position, start_position, mode, behavior) = {
            let f = self.fields.read();
            (
                f.start_trsfs.clone(),
                f.position,
                f.start_position,
                f.current_mode,
                f.behavior_on_transform,
            )
        };
        for (object, old) in items.iter().zip(starts.iter()) {
            let Some(object) = object.get() else { continue };
            let pers = object.transform_persistence();
            if let Some(pers) = pers.get()
                && (pers.is_zoom_or_rotate() || pers.is_axial())
            {
                let anchor = pers.anchor_point()?.xyz() - position.location().xyz()
                    + start_position.location().transformed(*trsf).xyz();
                pers.set_anchor_point(&Point3::from(anchor))?;
                continue;
            }
            let parent = object.combined_parent_transformation();
            match parent.get() {
                Some(parent) if parent.form() != TransformForm::Identity => {
                    let parent_trsf = *parent.trsf();
                    let local = parent_trsf.inverted()? * *trsf * parent_trsf * *old;
                    object.set_local_transformation(&local)?;
                }
                _ => object.set_local_transformation(&(*trsf * *old))?,
            }
        }
        let follows = (mode == AisManipulatorMode::Translation && behavior.follow_translation)
            || (mode == AisManipulatorMode::Rotation && behavior.follow_rotation)
            || (mode == AisManipulatorMode::TranslationPlane && behavior.follow_dragging);
        if !follows {
            return Ok(());
        }
        let location = start_position.location().transformed(*trsf);
        let direction = start_position.direction().transformed(*trsf);
        let x_direction = start_position.x_direction().transformed(*trsf);
        self.set_position(&Axis2::with_x_direction(location, direction, x_direction)?)
    }

    /// `Transform(x, y, view)`: the transformation of a pixel, applied when defined.
    /// Example: `let trsf = manipulator.transform_at(x, y, &view)?;`.
    pub fn transform_at(
        &self,
        x: i32,
        y: i32,
        view: &Handle<dyn V3dViewSource>,
    ) -> Result<Transform3, Failure> {
        let mut trsf = Transform3::default();
        if self.object_transformation(x, y, view, &mut trsf)? {
            self.transform(&trsf)?;
        }
        Ok(trsf)
    }

    /// `SetPosition`: a changed position moves the axes and updates the transformation.
    /// Example: `manipulator.set_position(&position)?`.
    pub fn set_position(&self, position: &Axis2) -> Result<(), Failure> {
        {
            let mut f = self.fields.write();
            if f.position
                .location()
                .is_equal(&position.location(), precision::confusion())
                && f.position
                    .direction()
                    .is_equal(&position.direction(), precision::angular())
                && f.position
                    .x_direction()
                    .is_equal(&position.x_direction(), precision::angular())
            {
                return Ok(());
            }
            f.position = *position;
            for index in 0..3 {
                f.axes[index].position = Self::axis_of(position, index as i32)?;
            }
        }
        self.update_manipulator_transformation()
    }

    /// `updateTransformation` (protected): the local transformation of the position (its
    /// rotation only when zoom persistent, with an axial zoom persistence at the location;
    /// else an axial scale persistence); `manipulator.update_manipulator_transformation()?`.
    pub fn update_manipulator_transformation(&self) -> Result<(), Failure> {
        let (position, zoom) = {
            let f = self.fields.read();
            (f.position, f.is_zoom_persistent_mode)
        };
        let mut trsf = Transform3::default();
        let from = if zoom {
            Axis2::with_x_direction(gp::origin(), position.direction(), position.x_direction())?
        } else {
            position
        };
        trsf.set_transformation_between(Axis3::from_axis2(from), Axis3::from_axis2(gp::xoy()));
        let datum = Handle::new(Datum3::new(trsf));
        self.presentable_base().fields.write().local_transformation = datum.clone();
        self.update_transformation()?;
        for axis in &self.fields.read().axes {
            axis.transform(&datum);
        }
        let pers = self.transform_persistence();
        if zoom {
            let keep = pers.get().is_some_and(|p| {
                p.mode() == Graphic3dTransModeFlags::AXIAL_ZOOM_PERS
                    && p.anchor_point()
                        .is_ok_and(|a| a.is_equal(&position.location(), 0.0))
            });
            if !keep {
                let pers = Graphic3dTransformPers::with_point(
                    Graphic3dTransModeFlags::AXIAL_ZOOM_PERS,
                    &position.location(),
                )?;
                self.set_manipulator_transform_persistence(&Handle::new(pers).cast());
            }
        } else if pers
            .get()
            .is_none_or(|p| p.mode() != Graphic3dTransModeFlags::AXIAL_SCALE_PERS)
        {
            let pers = Graphic3dTransformPers::with_point(
                Graphic3dTransModeFlags::AXIAL_SCALE_PERS,
                &position.location(),
            )?;
            self.set_manipulator_transform_persistence(&Handle::new(pers).cast());
        }
        Ok(())
    }

    /// `setTransformPersistence` (protected): the persistence of the object and of the axis
    /// highlight presentations; `manipulator.set_manipulator_transform_persistence(&pers)`.
    pub fn set_manipulator_transform_persistence(
        &self,
        pers: &Handle<dyn Graphic3dTransformPersSource>,
    ) {
        prs_mgr_set_transform_persistence(self, pers);
        for axis in &self.fields.read().axes {
            axis.set_transform_persistence(pers);
        }
    }

    /// `DeactivateCurrentMode`: restores the aspect of the active part (unless activated on
    /// detection) and clears the active axis, mode and transformation.
    /// Example: `manipulator.deactivate_current_mode()?`.
    pub fn deactivate_current_mode(&self) -> Result<(), Failure> {
        let (on_detection, index, mode, skin) = {
            let f = self.fields.read();
            (
                f.is_activation_on_detection,
                f.current_index,
                f.current_mode,
                f.skin_mode,
            )
        };
        if !on_detection {
            let group = self.get_group(index, mode);
            let Some(group) = group.get() else {
                return Ok(());
            };
            let shading = self.attributes().shading_aspect();
            let aspect = Prs3dShadingAspect::new()?;
            if skin == AisManipulatorSkin::Flat {
                aspect
                    .aspect()
                    .set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
            }
            aspect
                .aspect()
                .set_interior_style(AspectInteriorStyle::Solid);
            aspect.set_material(&shading.material(BOTH), BOTH);
            if mode == AisManipulatorMode::TranslationPlane {
                aspect.set_transparency(1.0, BOTH)?;
            } else {
                aspect.set_transparency(shading.transparency(BOTH), BOTH)?;
                aspect.set_color(self.fields.read().axes[index as usize].color, BOTH)?;
            }
            group.set_group_primitives_aspect(&aspect.aspect().cast());
        }
        let mut f = self.fields.write();
        f.current_index = -1;
        f.current_mode = AisManipulatorMode::None;
        f.has_started_transformation = false;
        Ok(())
    }

    /// `SetZoomPersistence`: the zoom persistence (recomputed when changed; an axial scale
    /// persistence without it) and the transformation; `manipulator.set_zoom_persistence(false)?`.
    pub fn set_zoom_persistence(&self, enable: bool) -> Result<(), Failure> {
        if self.fields.read().is_zoom_persistent_mode != enable {
            self.set_to_update_all();
        }
        self.fields.write().is_zoom_persistent_mode = enable;
        if !enable {
            let pers = Graphic3dTransformPers::new(Graphic3dTransModeFlags::AXIAL_SCALE_PERS)?;
            self.set_manipulator_transform_persistence(&Handle::new(pers).cast());
        }
        self.update_manipulator_transformation()
    }
}
