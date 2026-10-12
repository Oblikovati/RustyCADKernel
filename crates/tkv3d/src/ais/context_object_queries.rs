//! Query overloads append to caller lists; None has different filtering meanings upstream.
use super::{context_object_walk::AisObjectWalk, *};
use crate::prs_mgr::PrsMgrDisplayStatus;
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::v3d::V3dViewSource;
use rustycad_tkernel::{
    ncollection::List,
    standard::{Failure, Handle},
};
use rustycad_tkservice::graphic3d::Graphic3dCViewMembers;
type InteractiveList = List<Handle<dyn AisInteractiveObject>>;
impl AisInteractiveContext {
    /// Append displayed objects in bucket order; `context.displayed_objects(&mut objects)?`.
    pub fn displayed_objects(&self, objects: &mut InteractiveList) -> Result<(), Failure> {
        self.objects_by_display_status(PrsMgrDisplayStatus::Displayed, objects)
    }
    /// Append erased objects in bucket order; `context.erased_objects(&mut objects)?`.
    pub fn erased_objects(&self, objects: &mut InteractiveList) -> Result<(), Failure> {
        self.objects_by_display_status(PrsMgrDisplayStatus::Erased, objects)
    }
    /// Append displayed objects matching kind/signature; `context.displayed_objects_by_kind(kind, -1, &mut objects)?`.
    pub fn displayed_objects_by_kind(
        &self,
        kind: AisKindOfInteractive,
        signature: i32,
        objects: &mut InteractiveList,
    ) -> Result<(), Failure> {
        self.objects_by_display_status_and_kind(
            kind,
            signature,
            PrsMgrDisplayStatus::Displayed,
            objects,
        )
    }
    /// Append erased objects matching kind/signature; `context.erased_objects_by_kind(kind, -1, &mut objects)?`.
    pub fn erased_objects_by_kind(
        &self,
        kind: AisKindOfInteractive,
        signature: i32,
        objects: &mut InteractiveList,
    ) -> Result<(), Failure> {
        self.objects_by_display_status_and_kind(
            kind,
            signature,
            PrsMgrDisplayStatus::Erased,
            objects,
        )
    }
    /// Append objects with exactly this state, including None; `context.objects_by_display_status(state, &mut objects)?`.
    pub fn objects_by_display_status(
        &self,
        status: PrsMgrDisplayStatus,
        objects: &mut InteractiveList,
    ) -> Result<(), Failure> {
        self.append_matching_objects(objects, |object| object.display_status() == status)
    }
    /// Filter by kind/signature; status None disables the state filter; `context.objects_by_display_status_and_kind(kind, -1, state, &mut objects)?`.
    pub fn objects_by_display_status_and_kind(
        &self,
        kind: AisKindOfInteractive,
        signature: i32,
        status: PrsMgrDisplayStatus,
        objects: &mut InteractiveList,
    ) -> Result<(), Failure> {
        self.append_matching_objects(objects, |object| {
            (status == PrsMgrDisplayStatus::None || object.display_status() == status)
                && matches_kind(object, kind, signature)
        })
    }
    /// Append registered objects; None/-1 alone disables kind filtering; `context.objects_inside(&mut objects, AisKindOfInteractive::None, -1)?`.
    pub fn objects_inside(
        &self,
        objects: &mut InteractiveList,
        kind: AisKindOfInteractive,
        signature: i32,
    ) -> Result<(), Failure> {
        self.append_matching_objects(objects, |object| {
            (kind == AisKindOfInteractive::None && signature == -1)
                || matches_kind(object, kind, signature)
        })
    }
    /// Match native affinity querying, including objects outside a requested status; `context.objects_for_view(&mut objects, &view, true, state)?`.
    pub fn objects_for_view(
        &self,
        objects: &mut InteractiveList,
        view: &Handle<dyn V3dViewSource>,
        visible: bool,
        status: PrsMgrDisplayStatus,
    ) -> Result<(), Failure> {
        let id = view
            .get()
            .expect("ObjectsForView view is null; expected a live V3d view")
            .view()
            .identification();
        // OCCT 3d097a0 appends mismatching states before inspecting affinity.
        self.append_matching_objects(objects, |object| {
            (status != PrsMgrDisplayStatus::None && object.display_status() != status)
                || object.view_affinity().is_visible(id) == visible
        })
    }
    fn append_matching_objects(
        &self,
        objects: &mut InteractiveList,
        accept: impl Fn(&Handle<dyn AisInteractiveObject>) -> bool,
    ) -> Result<(), Failure> {
        let mut walk = AisObjectWalk::new(self)?;
        while let Some(object) = walk.current() {
            if accept(object) {
                objects.append(object.clone());
            }
            walk.next(self)?;
        }
        Ok(())
    }
}
fn matches_kind(
    object: &Handle<dyn AisInteractiveObject>,
    kind: AisKindOfInteractive,
    signature: i32,
) -> bool {
    object.kind_of_interactive() == kind && (signature == -1 || object.signature() == signature)
}
