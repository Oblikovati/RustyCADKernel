// Rust port of ApplicationFramework/TKVCAF/TPrsStd/TPrsStd_AISPresentation.cxx (display
// control and the visualization settings) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The members of `TPrsStd_AISPresentation` that change the settings of the label's data
//! and pass them to the interactive object, through the context when there is one.
use super::TPrsStdAISPresentation;
use rustycad_tkcaf::tdata_xtd::TDataXtdPresentation;
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Guid, Handle};
use rustycad_tklcaf::tdf::TdfAttributeState;
use rustycad_tkservice::graphic3d::{Graphic3dMaterialAspect, Graphic3dNameOfMaterial};
use rustycad_tkv3d::ais::{AisInteractiveContext, AisInteractiveObject};
use rustycad_tkv3d::prs_mgr::PrsMgrPresentableObjectMembers;

/// The context form of a change: `aContext->X(myAIS, ..., false)`.
pub(super) type ContextChange<'a> =
    &'a dyn Fn(&AisInteractiveContext, &Handle<dyn AisInteractiveObject>) -> Result<(), Failure>;

/// A change to an interactive object: through its context (no viewer update), or directly
/// on the object when the data framework has no viewer.
pub(super) struct AisChange<'a> {
    pub(super) through_context: ContextChange<'a>,
    pub(super) direct: &'a dyn Fn(&dyn AisInteractiveObject) -> Result<(), Failure>,
}

impl TPrsStdAISPresentation {
    /// Changes the data (`getData()->...`).
    fn change_data(
        &self,
        change: impl FnOnce(&TDataXtdPresentation) -> Result<(), Failure>,
    ) -> Result<(), Failure> {
        let data = self.get_data()?;
        change(rustycad_tklcaf::live(&data, "TPrsStd_AISPresentation")?)
    }
    /// The native `if (!aContext.IsNull()) aContext->X(myAIS, ..., false); else myAIS->X(...)`.
    pub(super) fn apply_to_ais(
        &self,
        ais: &Handle<dyn AisInteractiveObject>,
        change: AisChange<'_>,
    ) -> Result<(), Failure> {
        let context = self.get_ais_context()?;
        if let Some(context) = context.get() {
            return (change.through_context)(context, ais);
        }
        let object = rustycad_tklcaf::live(ais, "TPrsStd_AISPresentation")?;
        (change.direct)(object)
    }
    /// `myAIS.IsNull()` then `AISUpdate()`: builds the interactive object when there is none;
    /// returns the (possibly new) object.
    fn ais_or_update(&self) -> Result<Handle<dyn AisInteractiveObject>, Failure> {
        if self.get_ais().is_null() {
            self.ais_update()?;
        }
        Ok(self.get_ais())
    }
}

/// Display control.
impl TPrsStdAISPresentation {
    /// `Display(theIsUpdate)`: updates the object when asked or when there is none, then
    /// displays it.
    pub fn display(&self, update: bool) -> Result<(), Failure> {
        if update || self.get_ais().is_null() {
            self.ais_update()?;
        }
        self.ais_display()
    }
    /// `Erase(theIsRemove)`: erases (or removes) a displayed object.
    pub fn erase(&self, remove: bool) -> Result<(), Failure> {
        if self.is_displayed()? || remove {
            self.ais_erase(remove)?;
        }
        Ok(())
    }
    /// `Update`: `AISUpdate`.
    pub fn update(&self) -> Result<(), Failure> {
        self.ais_update()
    }
    /// `SetDisplayed(theIsDisplayed)`: backs up, then sets the data's flag.
    pub fn set_displayed(&self, displayed: bool) -> Result<(), Failure> {
        self.backup()?;
        self.change_data(|data| data.set_displayed(displayed))
    }
    /// `SetDriverGUID(theGUID)`: backs up, then sets the data's driver.
    pub fn set_driver_guid(&self, guid: &Guid) -> Result<(), Failure> {
        self.backup()?;
        self.change_data(|data| data.set_driver_guid(guid))
    }
}

/// Material and transparency.
impl TPrsStdAISPresentation {
    /// `UnsetMaterial`.
    pub fn unset_material(&self) -> Result<(), Failure> {
        let ais = self.get_ais();
        let ais_has = ais.get().is_some_and(|object| object.has_material());
        if !self.has_own_material()? && !ais.is_null() && !ais_has {
            return Ok(());
        }
        self.change_data(TDataXtdPresentation::unset_material)?;
        let ais = self.ais_or_update()?;
        if !ais.get().is_some_and(|object| object.has_material()) {
            return Ok(());
        }
        self.apply_to_ais(
            &ais,
            AisChange {
                through_context: &|context, object| context.unset_material(object, false),
                direct: &|object| object.unset_material(),
            },
        )
    }
    /// `SetMaterial(theName)`. Natively the first check reads `HasOwnMode`, not
    /// `HasOwnMaterial`; the port keeps it.
    pub fn set_material(&self, name: Graphic3dNameOfMaterial) -> Result<(), Failure> {
        self.backup()?;
        let ais_has = |ais: &Handle<dyn AisInteractiveObject>| {
            ais.get()
                .is_some_and(|object| object.has_material() && object.material() == name)
        };
        let index = self.read_data(TDataXtdPresentation::material_index)?;
        if self.has_own_mode()? && index == name as i32 && ais_has(&self.get_ais()) {
            return Ok(());
        }
        self.change_data(|data| data.set_material_index(name as i32))?;
        let ais = self.ais_or_update()?;
        if ais.is_null() || ais_has(&ais) {
            return Ok(());
        }
        let aspect = Graphic3dMaterialAspect::new(name);
        self.apply_to_ais(
            &ais,
            AisChange {
                through_context: &|context, object| context.set_material(object, &aspect, false),
                direct: &|object| object.set_material(&aspect),
            },
        )
    }
    /// `SetTransparency(theValue)`.
    pub fn set_transparency(&self, value: f64) -> Result<(), Failure> {
        self.backup()?;
        let ais_has = |ais: &Handle<dyn AisInteractiveObject>| {
            ais.get()
                .is_some_and(|object| object.transparency() == value)
        };
        if self.has_own_transparency()? && self.transparency()? == value && ais_has(&self.get_ais())
        {
            return Ok(());
        }
        self.change_data(|data| data.set_transparency(value))?;
        let ais = self.ais_or_update()?;
        if ais.is_null() || ais_has(&ais) {
            return Ok(());
        }
        self.apply_to_ais(
            &ais,
            AisChange {
                through_context: &|context, object| context.set_transparency(object, value, false),
                direct: &|object| object.set_transparency(value),
            },
        )
    }
    /// `UnsetTransparency`.
    pub fn unset_transparency(&self) -> Result<(), Failure> {
        if !self.has_own_transparency()? {
            return Ok(());
        }
        self.change_data(TDataXtdPresentation::unset_transparency)?;
        let ais = self.ais_or_update()?;
        if ais.is_null() {
            return Ok(());
        }
        self.apply_to_ais(
            &ais,
            AisChange {
                through_context: &|context, object| context.unset_transparency(object, false),
                direct: &|object| object.unset_transparency(),
            },
        )
    }
}

/// `Quantity_Color aColor; myAIS->Color(aColor);` compared by name.
fn ais_has_color(ais: &Handle<dyn AisInteractiveObject>, color: NamedColor) -> bool {
    ais.get()
        .is_some_and(|object| object.has_color() && object.color().name() == color)
}

/// Color and width.
impl TPrsStdAISPresentation {
    /// `UnsetColor`.
    pub fn unset_color(&self) -> Result<(), Failure> {
        let ais = self.get_ais();
        let ais_has = ais.get().is_some_and(|object| object.has_color());
        if !self.has_own_color()? && !ais.is_null() && !ais_has {
            return Ok(());
        }
        self.change_data(TDataXtdPresentation::unset_color)?;
        let ais = self.ais_or_update()?;
        if !ais.get().is_some_and(|object| object.has_color()) {
            return Ok(());
        }
        self.apply_to_ais(
            &ais,
            AisChange {
                through_context: &|context, object| context.unset_color(object, false),
                direct: &|object| object.unset_color(),
            },
        )
    }
    /// `SetColor(theColor)`.
    pub fn set_color(&self, color: NamedColor) -> Result<(), Failure> {
        self.backup()?;
        if self.has_own_color()? && self.color()? == color && ais_has_color(&self.get_ais(), color)
        {
            return Ok(());
        }
        self.change_data(|data| data.set_color(color))?;
        let ais = self.ais_or_update()?;
        if ais.is_null() || ais_has_color(&ais, color) {
            return Ok(());
        }
        let value = Color::named(color);
        self.apply_to_ais(
            &ais,
            AisChange {
                through_context: &|context, object| context.set_color(object, value, false),
                direct: &|object| object.set_color(value),
            },
        )
    }
    /// `SetWidth(theWidth)`. Natively it does not build a missing object.
    pub fn set_width(&self, width: f64) -> Result<(), Failure> {
        self.backup()?;
        let ais_has = |ais: &Handle<dyn AisInteractiveObject>| {
            ais.get()
                .is_some_and(|object| object.has_width() && object.width() == width)
        };
        if self.has_own_width()? && self.width()? == width && ais_has(&self.get_ais()) {
            return Ok(());
        }
        self.change_data(|data| data.set_width(width))?;
        let ais = self.get_ais();
        if ais.is_null() || ais_has(&ais) {
            return Ok(());
        }
        self.apply_to_ais(
            &ais,
            AisChange {
                through_context: &|context, object| context.set_width(object, width, false),
                direct: &|object| object.set_width(width),
            },
        )
    }
    /// `UnsetWidth`.
    pub fn unset_width(&self) -> Result<(), Failure> {
        let ais = self.get_ais();
        let ais_has = ais.get().is_some_and(|object| object.has_width());
        if !self.has_own_width()? && !ais.is_null() && !ais_has {
            return Ok(());
        }
        self.change_data(TDataXtdPresentation::unset_width)?;
        let ais = self.ais_or_update()?;
        if !ais.get().is_some_and(|object| object.has_width()) {
            return Ok(());
        }
        self.apply_to_ais(
            &ais,
            AisChange {
                through_context: &|context, object| context.unset_width(object, false),
                direct: &|object| object.unset_width(),
            },
        )
    }
}

/// Display and selection modes.
impl TPrsStdAISPresentation {
    /// `SetMode(theMode)`.
    pub fn set_mode(&self, mode: i32) -> Result<(), Failure> {
        self.backup()?;
        let ais_has = |ais: &Handle<dyn AisInteractiveObject>| {
            ais.get()
                .is_some_and(|object| object.display_mode() == mode)
        };
        if self.has_own_mode()? && self.mode()? == mode && ais_has(&self.get_ais()) {
            return Ok(());
        }
        self.change_data(|data| data.set_mode(mode))?;
        let ais = self.ais_or_update()?;
        if ais.is_null() || ais_has(&ais) {
            return Ok(());
        }
        self.apply_to_ais(
            &ais,
            AisChange {
                through_context: &|context, object| context.set_display_mode(object, mode, false),
                direct: &|object| {
                    object.set_display_mode(mode);
                    Ok(())
                },
            },
        )
    }
    /// `UnsetMode`.
    pub fn unset_mode(&self) -> Result<(), Failure> {
        let ais = self.get_ais();
        let ais_has = ais.get().is_some_and(|object| object.has_display_mode());
        if !self.has_own_mode()? && !ais.is_null() && !ais_has {
            return Ok(());
        }
        self.change_data(TDataXtdPresentation::unset_mode)?;
        let ais = self.ais_or_update()?;
        if !ais.get().is_some_and(|object| object.has_display_mode()) {
            return Ok(());
        }
        self.apply_to_ais(
            &ais,
            AisChange {
                through_context: &|context, object| context.unset_display_mode(object, false),
                direct: &|object| {
                    object.unset_display_mode();
                    Ok(())
                },
            },
        )
    }
    /// `SetSelectionMode(theSelectionMode, theTransaction)`.
    pub fn set_selection_mode(&self, mode: i32, transaction: bool) -> Result<(), Failure> {
        if transaction {
            self.backup()?;
        }
        self.change_data(|data| data.set_selection_mode(mode, transaction))?;
        self.update_or_activate()
    }
    /// `AddSelectionMode(theSelectionMode, theTransaction)`.
    pub fn add_selection_mode(&self, mode: i32, transaction: bool) -> Result<(), Failure> {
        if transaction {
            self.backup()?;
        }
        self.change_data(|data| data.add_selection_mode(mode, transaction))?;
        self.update_or_activate()
    }
    /// `UnsetSelectionMode`: clears the data's modes, then `AISUpdate`.
    pub fn unset_selection_mode(&self) -> Result<(), Failure> {
        self.change_data(TDataXtdPresentation::unset_selection_mode)?;
        self.ais_update()
    }
    /// The tail of the selection-mode setters: builds a missing object, else activates the
    /// modes on the existing one.
    fn update_or_activate(&self) -> Result<(), Failure> {
        if self.get_ais().is_null() {
            self.ais_update()
        } else {
            self.activate_selection_mode()
        }
    }
}
