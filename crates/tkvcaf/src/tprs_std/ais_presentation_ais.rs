// Rust port of ApplicationFramework/TKVCAF/TPrsStd/TPrsStd_AISPresentation.cxx (the
// protected AISUpdate, AISDisplay, AISErase and ActivateSelectionMode) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! How `TPrsStd_AISPresentation` builds its interactive object with the driver of the data,
//! applies the owned settings to it and shows, erases or removes it in the contexts.
use super::ais_presentation_settings::AisChange;
use super::{TPrsStdAISPresentation, TPrsStdAISViewer, TPrsStdDriverTable};
use rustycad_tkcaf::tdata_xtd::TDataXtdPresentation;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tklcaf::tdf::{TdfAttribute, TdfAttributeState};
use rustycad_tkservice::graphic3d::Graphic3dMaterialAspect;
use rustycad_tkv3d::ais::{
    AisInteractiveContext, AisInteractiveObject, AisInteractiveObjectMembers,
    AisSelectionModesConcurrency,
};
use rustycad_tkv3d::prs_mgr::PrsMgrPresentableObjectMembers;

impl TPrsStdAISPresentation {
    /// `AISUpdate`: builds or updates the object with the driver of the data, applies the
    /// owned settings and redisplays a displayed object.
    pub(super) fn ais_update(&self) -> Result<(), Failure> {
        self.backup()?;
        let data = self.get_data()?;
        rustycad_tklcaf::live(&data, "TPrsStd_AISPresentation::AISUpdate")?.backup()?;
        if self.label().is_null() {
            return Ok(());
        }
        let context = self.get_ais_context()?;
        self.rebuild_with_driver(&context)?;
        let ais = self.get_ais();
        if !ais.is_null() {
            self.apply_owned_settings(&ais)?;
            self.activate_selection_mode()?;
        }
        if self.is_displayed()?
            && let Some(context) = context.get()
        {
            context.redisplay(&ais, false, false)?;
        }
        Ok(())
    }
    /// The driver part of `AISUpdate`: a new object owned by this attribute, or the updated
    /// one (the previous object leaves the context when the driver replaced it).
    fn rebuild_with_driver(&self, context: &Handle<AisInteractiveContext>) -> Result<(), Failure> {
        let mut driver = Handle::default();
        if !TPrsStdDriverTable::get().find_driver(&self.get_driver_guid()?, &mut driver) {
            return Ok(());
        }
        let driver = rustycad_tklcaf::live(&driver, "TPrsStd_AISPresentation::AISUpdate")?;
        let previous = self.get_ais();
        let mut object = previous.clone();
        if !driver.update(&self.label(), &mut object)? {
            return Ok(());
        }
        if !previous.is_null() && object.object_address() == previous.object_address() {
            return Ok(());
        }
        if let Some(context) = context.get().filter(|_| !previous.is_null()) {
            context.remove(&previous, false)?;
        }
        self.set_ais(object.clone());
        let owner = self.attribute_base().this()?.cast::<StandardTransient>();
        rustycad_tklcaf::live(&object, "TPrsStd_AISPresentation::AISUpdate")?.set_owner(&owner);
        Ok(())
    }
    /// The settings part of `AISUpdate`: each owned value the object does not have yet.
    fn apply_owned_settings(&self, ais: &Handle<dyn AisInteractiveObject>) -> Result<(), Failure> {
        let object = rustycad_tklcaf::live(ais, "TPrsStd_AISPresentation::AISUpdate")?;
        if self.has_own_color()? {
            let color = self.color()?;
            if !object.has_color() || object.color().name() != color {
                let value = Color::named(color);
                self.apply_to_ais(
                    ais,
                    AisChange {
                        through_context: &|c, o| c.set_color(o, value, false),
                        direct: &|o| o.set_color(value),
                    },
                )?;
            }
        }
        if self.has_own_material()? {
            let material = self.material()?;
            if !object.has_material() || object.material() != material {
                let aspect = Graphic3dMaterialAspect::new(material);
                self.apply_to_ais(
                    ais,
                    AisChange {
                        through_context: &|c, o| c.set_material(o, &aspect, false),
                        direct: &|o| o.set_material(&aspect),
                    },
                )?;
            }
        }
        self.apply_owned_transparency_and_width(ais, object)?;
        if self.has_own_mode()? {
            let mode = self.mode()?;
            if object.display_mode() != mode {
                object.set_display_mode(mode);
            }
        }
        Ok(())
    }
    /// The transparency and width steps of the owned settings.
    fn apply_owned_transparency_and_width(
        &self,
        ais: &Handle<dyn AisInteractiveObject>,
        object: &dyn AisInteractiveObject,
    ) -> Result<(), Failure> {
        if self.has_own_transparency()? {
            let value = self.transparency()?;
            if object.transparency() != value {
                self.apply_to_ais(
                    ais,
                    AisChange {
                        through_context: &|c, o| c.set_transparency(o, value, false),
                        direct: &|o| o.set_transparency(value),
                    },
                )?;
            }
        }
        if self.has_own_width()? {
            let width = self.width()?;
            if !object.has_width() || object.width() != width {
                self.apply_to_ais(
                    ais,
                    AisChange {
                        through_context: &|c, o| c.set_width(o, width, false),
                        direct: &|o| o.set_width(width),
                    },
                )?;
            }
        }
        Ok(())
    }
    /// `AISDisplay`: displays the object in the viewer's context (after removing it from
    /// another context it belongs to) and marks the data displayed.
    pub(super) fn ais_display(&self) -> Result<(), Failure> {
        if self.label().is_null() {
            return Ok(());
        }
        let context = self.get_ais_context()?;
        let Some(context_ref) = context.get() else {
            return Ok(());
        };
        let ais = self.get_ais();
        let Some(object) = ais.get() else {
            return Ok(());
        };
        let own = object.get_context();
        if let Some(own_ref) = own
            .get()
            .filter(|_| own.object_address() != context.object_address())
        {
            own_ref.remove(&ais, false)?;
        }
        if self.is_displayed()? && context_ref.is_displayed(&ais) {
            return Ok(());
        }
        context_ref.display(&ais, false)?;
        if context_ref.is_displayed(&ais) {
            self.set_displayed(true)?;
        }
        Ok(())
    }
    /// `AISErase(theIsRemove)`: erases (or removes and drops) the object in the viewer's
    /// context and in the object's own context; the data is marked not displayed.
    pub(super) fn ais_erase(&self, remove: bool) -> Result<(), Failure> {
        let ais = self.get_ais();
        let Some(object) = ais.get() else {
            return Ok(());
        };
        self.backup()?;
        if self.label().is_null() {
            // Natively the own context is still null on this branch: nothing happens.
            return Ok(());
        }
        if self.is_attribute(&TDataXtdPresentation::get_id())? {
            self.set_displayed(false)?;
        }
        let mut viewer: Handle<TPrsStdAISViewer> = Handle::default();
        if !TPrsStdAISViewer::find(&self.label(), &mut viewer)? {
            return Ok(());
        }
        let own = object.get_context();
        let context = rustycad_tklcaf::live(&viewer, "TPrsStd_AISPresentation::AISErase")?
            .get_interactive_context();
        let distinct = own.object_address() != context.object_address();
        for target in [Some(&context), distinct.then_some(&own)]
            .into_iter()
            .flatten()
        {
            let Some(target) = target.get() else {
                continue;
            };
            if remove {
                target.remove(&ais, false)?;
            } else {
                target.erase(&ais, false)?;
            }
        }
        if remove {
            self.set_ais(Handle::default());
        }
        Ok(())
    }
    /// `ActivateSelectionMode`: activates the owned selection modes of the object in the
    /// viewer's context (one mode: -1 deactivates all, another mode is added when not
    /// active; several modes: the first replaces the active ones, the others are added).
    pub(super) fn activate_selection_mode(&self) -> Result<(), Failure> {
        let ais = self.get_ais();
        if ais.is_null() || !self.has_own_selection_mode()? {
            return Ok(());
        }
        let context = self.get_ais_context()?;
        let Some(context) = context.get() else {
            return Ok(());
        };
        let mut activated = List::default();
        context.activated_modes(&ais, &mut activated);
        let count = self.get_nb_selection_modes()?;
        if count == 1 {
            let mode = self.selection_mode(1)?;
            if mode == -1 {
                return context.deactivate(&ais);
            }
            if !activated.iter().any(|active| *active == mode) {
                context.set_selection_mode_active(
                    &ais,
                    mode,
                    true,
                    AisSelectionModesConcurrency::Multiple,
                    false,
                )?;
            }
            return Ok(());
        }
        for index in 1..=count {
            let concurrency = if index == 1 {
                AisSelectionModesConcurrency::Single
            } else {
                AisSelectionModesConcurrency::Multiple
            };
            let mode = self.selection_mode(index)?;
            context.set_selection_mode_active(&ais, mode, true, concurrency, false)?;
        }
        Ok(())
    }
}
