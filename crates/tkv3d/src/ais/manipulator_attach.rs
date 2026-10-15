//! `AIS_Manipulator` attachment: the attached objects (kept as the owner), placement on
//! them, mode activation and detachment.
use super::*;
use crate::prs_mgr::*;
use rustycad_tkernel::ncollection::HSequence;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, exceptions::program_error};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{self, Point3};
use rustycad_tkservice::graphic3d::Graphic3dTransformPersMembers;

/// The attached objects: `NCollection_HSequence<Handle(AIS_InteractiveObject)>`.
pub type AisManipulatorObjects = HSequence<Handle<dyn AisInteractiveObject>>;

impl AisManipulator {
    fn this_object(&self) -> Handle<dyn AisInteractiveObject> {
        self.presentable_base().this().cast()
    }

    /// `EnableMode`: activates a mode in the context of an attached manipulator.
    /// Example: `manipulator.enable_mode(AisManipulatorMode::Rotation)?`.
    pub fn enable_mode(&self, mode: AisManipulatorMode) -> Result<(), Failure> {
        if !self.is_attached() {
            return Ok(());
        }
        let context = self.get_context();
        let Some(context) = context.get() else {
            return Ok(());
        };
        context.activate(&self.this_object(), mode as i32, false)
    }

    /// `attachToPoint` (protected): placed at a point with the world axes.
    /// Example: `manipulator.attach_to_point(&point)?`.
    pub fn attach_to_point(&self, point: &Point3) -> Result<(), Failure> {
        let mut position = gp::xoy();
        position.set_location(*point);
        self.set_position(&position)
    }

    /// `attachToBox` (protected): placed at the center of a non-void box.
    /// Example: `manipulator.attach_to_box(&bounds)?`.
    pub fn attach_to_box(&self, bounds: &BoundingBox3) -> Result<(), Failure> {
        if bounds.is_void() {
            return Ok(());
        }
        let [xmin, ymin, zmin, xmax, ymax, zmax] = bounds.get()?;
        let center = Point3::new(
            (xmin + xmax) * 0.5,
            (ymin + ymax) * 0.5,
            (zmin + zmax) * 0.5,
        );
        self.attach_to_point(&center)
    }

    /// `adjustSize` (protected): half the largest box extent as the size; a void box is a
    /// `Standard_ConstructionError`; `manipulator.adjust_size(&bounds)?`.
    pub fn adjust_size(&self, bounds: &BoundingBox3) -> Result<(), Failure> {
        let [xmin, ymin, zmin, xmax, ymax, zmax] = bounds.get()?;
        let size = (xmax - xmin).max((ymax - ymin).max(zmax - zmin));
        self.set_size((size * 0.5) as f32);
        Ok(())
    }

    /// `Attach(object)`: attaches one object (a manipulator is ignored).
    /// Example: `manipulator.attach(&object, &AisManipulatorOptionsForAttach::default())?`.
    pub fn attach(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        options: &AisManipulatorOptionsForAttach,
    ) -> Result<(), Failure> {
        if object.is_null() {
            return Err(Failure::with_message(
                "Standard_NullObject",
                "AIS_Manipulator::Attach: null object",
            ));
        }
        if Self::is_manipulator(object) {
            return Ok(());
        }
        let objects = Handle::new(AisManipulatorObjects::default());
        objects.get().expect("new sequence").append(object.clone());
        self.attach_objects(&objects, options)
    }

    /// `Attach(objects)`: attaches objects (kept as the owner), placed and sized on the
    /// first one, displayed (or updated) and loaded in its context, with every mode enabled.
    /// Example: `manipulator.attach_objects(&objects, &options)?`.
    pub fn attach_objects(
        &self,
        objects: &Handle<AisManipulatorObjects>,
        options: &AisManipulatorOptionsForAttach,
    ) -> Result<(), Failure> {
        let sequence = objects.get().ok_or_else(null_objects)?;
        let Some(first) = sequence.sequence(|items| items.first().cloned()) else {
            return Ok(());
        };
        self.set_owner(&objects.clone().cast::<StandardTransient>());
        let mut bounds = BoundingBox3::default();
        let current = first.get().ok_or_else(null_objects)?;
        current.bounding_box(&mut bounds);
        if options.adjust_position {
            self.attach_position(current, &bounds)?;
        }
        if options.adjust_size {
            self.adjust_size(&bounds)?;
        }
        self.attach_context()?;
        if options.enable_modes {
            for mode in [
                AisManipulatorMode::Rotation,
                AisManipulatorMode::Translation,
                AisManipulatorMode::Scaling,
                AisManipulatorMode::TranslationPlane,
            ] {
                self.enable_mode(mode)?;
            }
        }
        Ok(())
    }

    fn attach_position(
        &self,
        object: &dyn AisInteractiveObject,
        bounds: &BoundingBox3,
    ) -> Result<(), Failure> {
        let pers = object.transform_persistence();
        if let Some(pers) = pers.get()
            && (pers.is_zoom_or_rotate() || pers.is_axial())
        {
            return self.attach_to_point(&pers.anchor_point()?);
        }
        self.attach_to_box(bounds)
    }

    fn attach_context(&self) -> Result<(), Failure> {
        let object = self.object()?;
        let context = object.get().ok_or_else(null_objects)?.get_context();
        let Some(context) = context.get() else {
            return Ok(());
        };
        let this = self.this_object();
        if !context.is_displayed(&this) {
            context.display(&this, false)?;
        } else {
            context.update(&this, false)?;
            context.recompute_selection_only(&this)?;
        }
        context.load(&this, -1)
    }

    /// `Detach`: deactivates the current mode, removes the manipulator from the context of
    /// the attached object and forgets the objects; `manipulator.detach()?`.
    pub fn detach(&self) -> Result<(), Failure> {
        self.deactivate_current_mode()?;
        if !self.is_attached() {
            return Ok(());
        }
        let object = self.object()?;
        let context = object.get().map(|o| o.get_context()).unwrap_or_default();
        if let Some(context) = context.get() {
            context.remove(&self.this_object(), false)?;
        }
        self.clear_owner();
        Ok(())
    }

    /// `Objects`: the attached objects (null when the owner is not a sequence of them).
    /// Example: `manipulator.objects()`.
    pub fn objects(&self) -> Handle<AisManipulatorObjects> {
        self.get_owner().cast::<AisManipulatorObjects>()
    }

    pub(super) fn objects_count(&self) -> Result<i32, Failure> {
        let objects = self.objects();
        let sequence = objects.get().ok_or_else(null_objects)?;
        Ok(sequence.sequence(|items| items.len() as i32))
    }

    /// `Object()`: the first attached object; `manipulator.object()?`.
    pub fn object(&self) -> Result<Handle<dyn AisInteractiveObject>, Failure> {
        self.object_at(1)
    }

    /// `Object(index)`: an attached object; an index outside the objects is a
    /// `Standard_ProgramError`, no objects a `Standard_NullObject` (a null dereference
    /// natively); `manipulator.object_at(2)?`.
    pub fn object_at(&self, index: i32) -> Result<Handle<dyn AisInteractiveObject>, Failure> {
        let objects = self.objects();
        let sequence = objects.get().ok_or_else(null_objects)?;
        sequence.sequence(|items| {
            if index < 1 || index > items.len() as i32 {
                return Err(program_error(
                    "AIS_Manipulator::Object(): wrong index value",
                ));
            }
            Ok(items
                .iter()
                .nth(index as usize - 1)
                .cloned()
                .unwrap_or_default())
        })
    }
}

fn null_objects() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "AIS_Manipulator: no attached objects",
    )
}
