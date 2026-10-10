//! Heavyweight prebuilding from OCCT 3d097a0; shape construction is not yet ported.
use crate::{
    select_mgr::SelectMgrSelection,
    select3d::{Select3DSensitiveEntity, Select3DSensitiveGroup},
};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};

/// Namespace for native BRep selection helpers; `StdSelectBrepSelectionTool::pre_build_bvh(&selection)?`.
pub struct StdSelectBrepSelectionTool;
impl StdSelectBrepSelectionTool {
    /// Build entities with at least 800,000 sub-elements, including exact group children.
    /// Example: `StdSelectBrepSelectionTool::pre_build_bvh(&selection)?`.
    pub fn pre_build_bvh(selection: &SelectMgrSelection) -> Result<(), Failure> {
        for sensitive in selection.entities() {
            let entity = sensitive.base_sensitive();
            prebuild_heavy_sensitive(&entity)?;
            if !entity.is_instance(Select3DSensitiveGroup::static_type()) {
                continue;
            }
            let group = entity.cast::<Select3DSensitiveGroup>();
            for child in group.entities() {
                prebuild_heavy_sensitive(&child)?;
            }
        }
        Ok(())
    }
}
fn prebuild_heavy_sensitive(entity: &Handle<dyn Select3DSensitiveEntity>) -> Result<(), Failure> {
    if entity.nb_sub_elements() >= 800_000 {
        entity.bvh()?;
    }
    Ok(())
}
