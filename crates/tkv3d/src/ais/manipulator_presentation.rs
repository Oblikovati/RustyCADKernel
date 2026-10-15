//! `AIS_Manipulator` presentation, selection and highlighting: the shaded parts, the
//! sensitive parts of each mode, the highlight of the detected and selected parts, and the
//! flat-skin parts facing the camera.
use super::manipulator_sensitive::AisManipulatorSensCircle;
use super::*;
use crate::prs_mgr::*;
use crate::prs3d::{Prs3dDrawer, Prs3dShadingAspect, Prs3dTypeOfHighlight};
use crate::select_mgr::*;
use crate::select3d::*;
use crate::v3d::V3dViewSource;
use rustycad_tkernel::ncollection::Vec2;
use rustycad_tkernel::standard::{Failure, Handle, HandleReference};
use rustycad_tkmath::gp::{self, Axis1, Axis2, Circle3, Point3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkservice::aspect::{AspectInteriorStyle, AspectTypeOfFacingModel};
use rustycad_tkservice::graphic3d::*;

const BOTH: AspectTypeOfFacingModel = AspectTypeOfFacingModel::BothSide;

impl AisManipulator {
    /// `Compute` (mode 1 only): the center sphere, the flat-skin outer circle and sector
    /// groups, then the parts of each axis in its color, and the transformation.
    fn compute_manipulator(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        prs: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        if mode != AisDisplayMode::Shaded as i32 {
            return Ok(());
        }
        prs.set_infinite_state(true);
        prs.set_mutable(true);
        let skin = self.fields.read().skin_mode;
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
        aspect.set_transparency(shading.transparency(BOTH), BOTH)?;
        let drawer_aspect = shading.aspect().cast::<dyn Graphic3dAspectsSource>();
        self.compute_center(prs, skin, &drawer_aspect)?;
        let any_rotation = self.fields.read().axes.iter().any(|a| a.has_rotation);
        if skin == AisManipulatorSkin::Flat && any_rotation {
            self.compute_flat_circle(prs, &drawer_aspect, &aspect)?;
        }
        for index in 0..3 {
            let group = prs.new_group()?;
            let fill = Graphic3dAspectFillArea3d::default();
            *fill.aspects_base().fields.write() =
                aspect.aspect().aspects_base().fields.read().clone();
            let axis_aspect = Prs3dShadingAspect::from_aspect(Handle::new(fill));
            let color = self.fields.read().axes[index].color;
            axis_aspect.set_color(color, BOTH)?;
            group.set_group_primitives_aspect(&axis_aspect.aspect().cast());
            let mut axis = self.fields.read().axes[index].clone();
            axis.compute(manager, prs, &axis_aspect, skin)?;
            axis.set_transform_persistence(&self.transform_persistence());
            self.fields.write().axes[index] = axis;
        }
        self.update_manipulator_transformation()
    }

    fn compute_center(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        skin: AisManipulatorSkin,
        aspect: &Handle<dyn Graphic3dAspectsSource>,
    ) -> Result<(), Failure> {
        let radius = self.fields.read().axes[0].axis_radius * 2.0;
        let mut center = self.fields.read().center.clone();
        center.init(radius, &gp::origin(), skin, 20, 20)?;
        let group = prs.new_group()?;
        group.set_primitives_aspect(aspect);
        group.add_primitive_array(&center.array().clone().cast(), true);
        self.fields.write().center = center;
        Ok(())
    }

    fn compute_flat_circle(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        drawer_aspect: &Handle<dyn Graphic3dAspectsSource>,
        aspect: &Prs3dShadingAspect,
    ) -> Result<(), Failure> {
        let (inner, size, facettes) = {
            let f = self.fields.read();
            (
                f.axes[0].inner_radius(),
                f.axes[0].size(),
                f.axes[0].facettes_number,
            )
        };
        let mut circle = self.fields.read().circle.clone();
        circle.init(
            inner,
            size,
            &Axis1::new(gp::origin(), gp::dz()),
            f64::from(2.0f32) * std::f64::consts::PI,
            facettes * 4,
            20,
        )?;
        let circle_group = prs.new_group()?;
        circle_group.set_primitives_aspect(drawer_aspect);
        circle_group.add_primitive_array(&circle.array().clone().cast(), true);
        let sector_group = prs.new_group()?;
        sector_group.set_group_primitives_aspect(&aspect.aspect().cast());
        let mut f = self.fields.write();
        f.circle = circle;
        f.circle_group = circle_group;
        f.sector_group = sector_group;
        Ok(())
    }

    /// `ComputeSelection`: the sensitive parts of a manipulation mode, each owned by an
    /// `AIS_ManipulatorOwner` of priority 9 (with lines, rings and points of a sensitivity
    /// growing with the size in the shaded skin).
    fn compute_manipulator_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        let mode = AisManipulatorMode::from_i32(mode);
        if mode == AisManipulatorMode::None {
            return Ok(());
        }
        let selection = selection.get().ok_or_else(|| {
            Failure::with_message("Standard_NullObject", "AIS_Manipulator: null selection")
        })?;
        let f = self.fields.read().clone();
        let coef = f.axes[0].size() / 100.0;
        let high = ((coef * 15.0) as i32).clamp(3, 15);
        let low = ((coef * 10.0) as i32).clamp(2, 10);
        let shaded = f.skin_mode == AisManipulatorSkin::Shaded;
        let this: Handle<dyn SelectMgrSelectableObject> = self.presentable_base().this().cast();
        for index in 0..3 {
            let axis = &f.axes[index];
            let (enabled, group, array) = match mode {
                AisManipulatorMode::Translation => (
                    axis.has_translation,
                    &axis.translator_group,
                    axis.triangle_array.clone(),
                ),
                AisManipulatorMode::Rotation => (
                    axis.has_rotation,
                    &axis.rotator_group,
                    axis.circle.array().clone(),
                ),
                AisManipulatorMode::Scaling => (
                    axis.has_scaling,
                    &axis.scaler_group,
                    axis.cube.array.clone(),
                ),
                _ => (
                    axis.has_dragging,
                    &axis.dragger_group,
                    axis.sector.array().clone(),
                ),
            };
            if !enabled {
                continue;
            }
            let owner: Handle<dyn SelectMgrEntityOwnerSource> =
                AisManipulatorOwner::new(&this, index as i32, mode, 9).cast();
            if shaded {
                self.add_shaded_sensitives(selection, &f, index, mode, &owner, high, low)?;
            }
            let location = group
                .get()
                .map_or_else(Location::default, |g| Location::new(g.transformation()));
            let triangles = Select3DSensitivePrimitiveArray::new(&owner)?;
            let array = array.get().ok_or_else(|| {
                Failure::with_message("Standard_NullObject", "AIS_Manipulator: part not computed")
            })?;
            triangles.init_triangulation(&Select3DPrimArrayInit::whole(
                &array.attributes(),
                &array.indices(),
                &location,
                true,
                1,
            ))?;
            selection.add(&Handle::new(triangles).cast())?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn add_shaded_sensitives(
        &self,
        selection: &SelectMgrSelection,
        f: &AisManipulatorFields,
        index: usize,
        mode: AisManipulatorMode,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        high: i32,
        low: i32,
    ) -> Result<(), Failure> {
        let axis = &f.axes[index];
        match mode {
            AisManipulatorMode::Translation => {
                let line = Select3DSensitiveSegment::new(owner, gp::origin(), axis.arrow_tip_pos);
                line.set_sensitivity_factor(high)?;
                selection.add(&Handle::new(line).cast())
            }
            AisManipulatorMode::Rotation => {
                let circle = Circle3::new(
                    Axis2::new(gp::origin(), axis.reference_axis.direction())?,
                    f64::from(axis.circle_radius),
                )?;
                let sensitive = AisManipulatorSensCircle::new(owner, &circle);
                sensitive.set_sensitivity_factor(low)?;
                selection.add(&Handle::new(sensitive).cast())
            }
            AisManipulatorMode::Scaling => {
                let point = Select3DSensitivePoint::new(owner, axis.cube_pos);
                point.set_sensitivity_factor(high)?;
                selection.add(&Handle::new(point).cast())
            }
            _ => self.add_dragger_segments(selection, f, index, owner, high, low),
        }
    }

    fn add_dragger_segments(
        &self,
        selection: &SelectMgrSelection,
        f: &AisManipulatorFields,
        index: usize,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        high: i32,
        low: i32,
    ) -> Result<(), Failure> {
        let offset = if f.is_zoom_persistent_mode {
            f64::from(high) * (0.5 + std::f64::consts::SQRT_2)
        } else {
            0.0
        };
        let (next, last) = (&f.axes[(index + 1) % 3], &f.axes[(index + 2) % 3]);
        let p1 = next
            .arrow_tip_pos
            .translated((last.reference_axis.direction().xyz() * offset).into());
        let p2 = last
            .arrow_tip_pos
            .translated((next.reference_axis.direction().xyz() * offset).into());
        let middle = (p1.xyz() + p2.xyz()) / 2.0;
        let origin = middle.normalized()? * offset;
        let line1 = Select3DSensitiveSegment::new(owner, p1, p2);
        line1.set_sensitivity_factor(low)?;
        selection.add(&Handle::new(line1).cast())?;
        let line2 =
            Select3DSensitiveSegment::new(owner, Point3::from(origin), Point3::from(middle));
        line2.set_sensitivity_factor(low)?;
        selection.add(&Handle::new(line2).cast())
    }

    /// `RecomputeSelection`: recomputes the selection of a mode, natively that of the first
    /// attached object (cleared and filled with the manipulator parts); a manipulator without
    /// objects is a `Standard_NullObject` failure (a null dereference natively).
    /// Example: `manipulator.recompute_selection(AisManipulatorMode::Rotation)?`.
    pub fn recompute_selection(&self, mode: AisManipulatorMode) -> Result<(), Failure> {
        if mode == AisManipulatorMode::None {
            return Ok(());
        }
        let object = self.object()?;
        let object = object.get().ok_or_else(|| {
            Failure::with_message("Standard_NullObject", "AIS_Manipulator: null object")
        })?;
        let selection = object.selection(mode as i32);
        if let Some(value) = selection.get() {
            value.clear();
            self.compute_manipulator_selection(&selection, mode as i32)?;
        }
        Ok(())
    }

    /// `HilightSelected`: in the selection activation, the selected part highlighted (a
    /// translucent axis color for a shaded plane dragger) and made active; another owner
    /// colors the whole manipulator with the context highlight style.
    fn hilight_manipulator_selected(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        owners: &[Handle<dyn SelectMgrEntityOwnerSource>],
    ) -> Result<(), Failure> {
        let Some(first) = owners.first() else {
            return Ok(());
        };
        if self.fields.read().is_activation_on_detection {
            return Ok(());
        }
        let context = self.get_context();
        let context = context.get().ok_or_else(|| {
            Failure::with_message("Standard_NullObject", "AIS_Manipulator: no context")
        })?;
        let style = context
            .highlight_style(Prs3dTypeOfHighlight::Dynamic)?
            .clone();
        let owner = first.cast::<AisManipulatorOwner>();
        let Some(owner) = owner.get() else {
            let this = self.presentable_base().this();
            return manager.color(
                &this,
                &style,
                0,
                &Handle::default(),
                GRAPHIC3D_ZLAYER_ID_TOPMOST,
            );
        };
        let (highlight, dragger, skin) = {
            let f = self.fields.read();
            (
                f.highlight_aspect.clone(),
                f.dragger_highlight.clone(),
                f.skin_mode,
            )
        };
        highlight.aspect().set_interior_color(style.color());
        let group = self.get_group(owner.index(), owner.mode());
        let Some(group) = group.get() else {
            return Ok(());
        };
        if owner.mode() == AisManipulatorMode::TranslationPlane
            && skin == AisManipulatorSkin::Shaded
        {
            let color = self.fields.read().axes[owner.index() as usize].color;
            dragger.set_color(color, BOTH)?;
            group.set_group_primitives_aspect(&dragger.aspect().cast());
        } else {
            group.set_group_primitives_aspect(&highlight.aspect().cast());
        }
        let mut f = self.fields.write();
        f.current_index = owner.index();
        f.current_mode = owner.mode();
        Ok(())
    }

    /// `HilightOwnerWithColor`: the highlight presentation of the detected part in the
    /// topmost immediate layer (made active in the detection activation).
    fn hilight_manipulator_owner(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        style: &Handle<Prs3dDrawer>,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    ) -> Result<(), Failure> {
        let prs = self.get_highlight_presentation(owner);
        let Some(value) = prs.get() else {
            return Ok(());
        };
        let owner = owner.cast::<AisManipulatorOwner>();
        let owner = owner.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "AIS_Manipulator: not a manipulator owner",
            )
        })?;
        value
            .cstructure()
            .cstructure_base()
            .fields
            .write()
            .view_affinity = self.view_affinity();
        let skin = self.fields.read().skin_mode;
        if owner.mode() == AisManipulatorMode::TranslationPlane
            && skin == AisManipulatorSkin::Shaded
        {
            let drawer = Prs3dDrawer::new();
            drawer.set_color(self.fields.read().axes[owner.index() as usize].color);
            drawer.set_transparency(0.5);
            value.highlight(&Handle::new(drawer).cast(), true);
        } else {
            value.highlight(&style.clone().cast(), true);
        }
        let highlight = self.fields.read().highlight_aspect.clone();
        for group in value.groups() {
            group.set_group_primitives_aspect(&highlight.aspect().cast());
        }
        value.set_z_layer(GRAPHIC3D_ZLAYER_ID_TOPMOST);
        manager.add_to_immediate_list(&prs);
        if !self.fields.read().is_activation_on_detection {
            return Ok(());
        }
        if self.has_active_mode() {
            self.deactivate_current_mode()?;
        }
        let mut f = self.fields.write();
        f.current_index = owner.index();
        f.current_mode = owner.mode();
        Ok(())
    }

    /// `ProcessDragging`: starts (in an active mode), confirms, updates, aborts or stops the
    /// transformation (clearing the flat sector or redisplaying on stop).
    fn process_manipulator_dragging(
        &self,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
        from: Vec2<i32>,
        to: Vec2<i32>,
        action: AisDragAction,
    ) -> Result<bool, Failure> {
        match action {
            AisDragAction::Start => {
                if !self.has_active_mode() {
                    return Ok(false);
                }
                self.start_transform(from.x, from.y, view)?;
            }
            AisDragAction::Confirmed => {}
            AisDragAction::Update => {
                self.transform_at(to.x, to.y, view)?;
            }
            AisDragAction::Abort => self.stop_transform(false)?,
            AisDragAction::Stop => {
                self.stop_transform(true)?;
                if self.fields.read().skin_mode == AisManipulatorSkin::Flat {
                    let group = self.fields.read().sector_group.clone();
                    if let Some(group) = group.get() {
                        group.clear(true);
                    }
                } else if let Some(ctx) = context.get() {
                    let this = self.presentable_base().this().cast();
                    if ctx.is_displayed(&this) {
                        ctx.redisplay(&this, true, false)?;
                    }
                }
            }
        }
        Ok(true)
    }
}

impl AisInteractiveObject for AisManipulator {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base
    }
    fn process_dragging(
        &self,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
        _: &Handle<dyn SelectMgrEntityOwnerSource>,
        from: Vec2<i32>,
        to: Vec2<i32>,
        action: AisDragAction,
    ) -> bool {
        rustycad_tkernel::standard::raise_native_failure(
            self.process_manipulator_dragging(context, view, from, to, action),
        )
    }
}

impl PrsMgrPresentableObject for AisManipulator {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.presentable
    }
    fn compute(
        &self,
        manager: &dyn HandleReference<PrsMgrPresentationManager>,
        prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        self.compute_manipulator(
            &manager.observe().clone_handle(),
            &prs.observe().clone_handle(),
            mode,
        )
    }
    /// `SetTransformPersistence`: refused in the zoom persistence mode (a debug assertion
    /// natively), else as `setTransformPersistence`.
    fn set_transform_persistence(&self, pers: &Handle<dyn Graphic3dTransformPersSource>) {
        if self.fields.read().is_zoom_persistent_mode {
            return;
        }
        self.set_manipulator_transform_persistence(pers);
    }
    /// `setLocalTransformation`: custom transformations are not supported (ignored).
    fn set_local_transformation_datum(
        &self,
        _: &Handle<rustycad_tkmath::toploc::Datum3>,
    ) -> Result<(), Failure> {
        Ok(())
    }
    /// `RecomputeTransformation`: in the flat skin, the parts turned to face the camera and
    /// their selections recomputed.
    fn recompute_transformation(&self, camera: &Handle<Graphic3dCamera>) {
        rustycad_tkernel::standard::raise_native_failure(
            self.recompute_flat_transformation(camera),
        );
    }
    crate::ais_presentable_overrides!();
}

impl SelectMgrSelectableObject for AisManipulator {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.selectable
    }
    fn compute_selection(
        &self,
        selection: &dyn HandleReference<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        self.compute_manipulator_selection(&selection.observe().clone_handle(), mode)
    }
    fn is_auto_hilight(&self) -> bool {
        false
    }
    /// `ClearSelected`: deactivates the current mode.
    fn clear_selected(&self) {
        rustycad_tkernel::standard::raise_native_failure(self.deactivate_current_mode());
    }
    fn hilight_selected(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        owners: &[Handle<dyn SelectMgrEntityOwnerSource>],
    ) -> Result<(), Failure> {
        self.hilight_manipulator_selected(manager, owners)
    }
    fn hilight_owner_with_color(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        style: &Handle<Prs3dDrawer>,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    ) -> Result<(), Failure> {
        self.hilight_manipulator_owner(manager, style, owner)
    }
}
