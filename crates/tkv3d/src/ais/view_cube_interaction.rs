//! Martensite's usable ViewCube retains native animation and dynamic-highlight state.
use super::{
    AisAnimationCamera, AisAnimationMembers, AisInteractiveObject, AisInteractiveObjectBase,
    AisViewCubePresentationBase, AisViewCubePresentationSource,
};
use crate::{prs_mgr::*, prs3d::*, select_mgr::*};
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use rustycad_tkservice::{aspect::AspectTypeOfFacingModel, graphic3d::*};
use std::sync::Arc;
/// Optional application callbacks; implement both methods to observe actual animation transitions.
/// Example: `cube.set_animation_events(Some(Arc::new(MyAnimationEvents)))`.
pub trait AisViewCubeAnimationEvents: Send + Sync {
    /// Called after a continuing frame, after any requested redraw; `events.after_animation()`.
    fn after_animation(&self);
    /// Called once at completion, before the animation detaches its view; `events.animation_finished()`.
    fn animation_finished(&self);
}
/// Native protected interaction policy and camera handles; `cube.interaction.fields.read()`.
pub struct AisViewCubeInteractionFields {
    /// Native stopped camera animation, initially duration 0.5 seconds.
    pub animation: Handle<AisAnimationCamera>,
    /// Retained starting camera, copied in place at every click.
    pub start: Handle<Graphic3dCamera>,
    /// Retained ending camera, copied in place at every click.
    pub end: Handle<Graphic3dCamera>,
    /// Automatically start camera animation on owner click.
    pub auto_start: bool,
    /// Run the uninterruptible native click loop when enabled.
    pub fixed_loop: bool,
    /// Fit selection first, falling back to the displayed scene.
    pub fit_selected: bool,
    /// Force the default up direction rather than choosing the nearest quarter turn.
    pub reset_up: bool,
    /// Application callbacks; absence preserves the native base class's empty virtual hooks.
    pub events: Option<Arc<dyn AisViewCubeAnimationEvents>>,
}
/// Embedded native class state for application subclasses; `base.presentation.settings.write()`.
pub struct AisViewCubeInteractionBase {
    /// Existing real geometry and appearance core.
    pub presentation: AisViewCubePresentationBase,
    /// Retained interaction state; cloned handles preserve identity.
    pub fields: SharedState<AisViewCubeInteractionFields>,
}
impl AisViewCubeInteractionBase {
    /// Initialize every native constructor field; `AisViewCubeInteractionBase::new()?`.
    pub fn new() -> Result<Self, Failure> {
        let animation = Handle::new(AisAnimationCamera::new("AIS_ViewCube", &Handle::default()));
        animation.set_own_duration(0.5);
        let base = Self {
            presentation: AisViewCubePresentationBase::new()?,
            fields: SharedState::new(AisViewCubeInteractionFields {
                animation,
                start: Handle::new(Graphic3dCamera::default()),
                end: Handle::new(Graphic3dCamera::default()),
                auto_start: true,
                fixed_loop: true,
                fit_selected: true,
                reset_up: false,
                events: None,
            }),
        };
        initialize_cube_dynamic_highlight(&base)?;
        Ok(base)
    }
}
/// ViewCube virtual interaction contract; geometry overrides and animation hooks remain dispatchable.
/// Example: delegate `interaction_base()` to the subclass's embedded state.
pub trait AisViewCubeSource: AisViewCubePresentationSource {
    /// Embedded complete constructor state; `cube.interaction_base()`.
    fn interaction_base(&self) -> &AisViewCubeInteractionBase;
    /// Dispatch the native click action; `cube.handle_click(&owner)?`.
    fn handle_click(&self, owner: &Handle<AisViewCubeOwner>) -> Result<(), Failure> {
        super::view_cube_animation::ais_view_cube_handle_click(self, owner)
    }
    /// Dispatch animation preparation; `cube.start_animation(&owner)?`.
    fn start_animation(&self, owner: &Handle<AisViewCubeOwner>) -> Result<(), Failure> {
        super::view_cube_animation::ais_view_cube_start_animation(self, owner)
    }
    /// Advance a real timer frame; `cube.update_animation(false)?`.
    fn update_animation(&self, redraw: bool) -> Result<bool, Failure> {
        super::view_cube_animation::ais_view_cube_update_animation(self, redraw)
    }
    /// Fit the requested camera using selection/scene bounds; `cube.view_fit_all(&view,&camera)?`.
    fn view_fit_all(
        &self,
        view: &crate::v3d::V3dView,
        camera: &Graphic3dCamera,
    ) -> Result<(), Failure> {
        super::view_cube_camera_fit::ais_view_cube_view_fit_all(self, view, camera)
    }
    /// Native base hook dispatches registered application events; `cube.on_after_animation()`.
    fn on_after_animation(&self) {
        let events = self.interaction_base().fields.read().events.clone();
        if let Some(events) = events {
            events.after_animation();
        }
    }
    /// Native base finish hook dispatches registered application events; `cube.on_animation_finished()`.
    fn on_animation_finished(&self) {
        let events = self.interaction_base().fields.read().events.clone();
        if let Some(events) = events {
            events.animation_finished();
        }
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn AisViewCubeSource,
    "AIS_ViewCube",
    dyn AisInteractiveObject
);
/// Concrete picking/highlight/camera widget; `AisViewCube::new()?`.
/// Whole-class parity remains partial while generated class declarations and derived animation transport are unfinished.
pub struct AisViewCube {
    transient: StandardTransient,
    /// Native constructor and mutable interaction state.
    pub interaction: AisViewCubeInteractionBase,
}
rustycad_tkernel::impl_transient!(AisViewCube,"AIS_ViewCube",dyn AisInteractiveObject,transient,
    interfaces:[dyn AisViewCubeSource,dyn AisInteractiveObject,dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl AisViewCube {
    /// Create a stopped 0.5-second native widget; attach its context before picking/clicking.
    /// Example: `let cube=AisViewCube::new()?; context.display(&cube.cast(),false)?`.
    pub fn new() -> Result<Handle<Self>, Failure> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            interaction: AisViewCubeInteractionBase::new()?,
        });
        prs_mgr_presentable_bind(&object.cast());
        Ok(object)
    }
    /// Install application observers without replacing camera animation; `cube.set_animation_events(Some(events))`.
    pub fn set_animation_events(&self, events: Option<Arc<dyn AisViewCubeAnimationEvents>>) {
        self.interaction.fields.write().events = events;
    }
}
impl AisInteractiveObject for AisViewCube {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.interaction.presentation.interactive
    }
}
impl AisViewCubePresentationSource for AisViewCube {
    fn view_cube_presentation_base(&self) -> &AisViewCubePresentationBase {
        &self.interaction.presentation
    }
}
impl AisViewCubeSource for AisViewCube {
    fn interaction_base(&self) -> &AisViewCubeInteractionBase {
        &self.interaction
    }
}
fn initialize_cube_dynamic_highlight(base: &AisViewCubeInteractionBase) -> Result<(), Failure> {
    let drawer = Handle::new(Prs3dDrawer::new());
    drawer.link_to(
        &base
            .presentation
            .interactive
            .presentable
            .fields
            .read()
            .drawer,
    );
    configure_cube_dynamic_drawer(&drawer)?;
    store_cube_dynamic_drawer(base, drawer);
    Ok(())
}
use super::AisViewCubeOwner;

fn cube_dynamic_shading() -> Result<Handle<Prs3dShadingAspect>, Failure> {
    let shading = Handle::new(Prs3dShadingAspect::new()?);
    shading
        .aspect()
        .set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    shading.set_color(
        Color::named(NamedColor::CYAN1),
        AspectTypeOfFacingModel::BothSide,
    )?;
    Ok(shading)
}

fn store_cube_dynamic_drawer(base: &AisViewCubeInteractionBase, drawer: Handle<Prs3dDrawer>) {
    base.presentation
        .interactive
        .presentable
        .fields
        .write()
        .dyn_hilight_drawer = drawer;
}

pub(super) fn configure_cube_dynamic_drawer(drawer: &Prs3dDrawer) -> Result<(), Failure> {
    let shading = cube_dynamic_shading()?;
    drawer.set_shading_aspect(shading);
    drawer.set_z_layer(GRAPHIC3D_ZLAYER_ID_TOPMOST);
    drawer.set_color(Color::named(NamedColor::CYAN1));
    Ok(())
}
