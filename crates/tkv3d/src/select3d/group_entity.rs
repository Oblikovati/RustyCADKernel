//! Group entity operations retain native cache writes, owner propagation and copy defaults.
use super::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase,
    Select3DSensitiveEntityMembers as _, Select3DSensitiveGroup, select3d_set_build_bvh,
};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Point3;
use std::io;
impl Select3DSensitiveEntity for Select3DSensitiveGroup {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        // The composite owner remains separate from each shared child owner.
        &self.entity
    }
    fn matches(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        super::triangulation_failure::catch_triangulation_failure(|| {
            self.match_group_volume(manager, pick_result)
        })
    }
    fn nb_sub_elements(&self) -> i32 {
        // Membership and primitive indices can differ after a defined child callback failure.
        self.state.read().nodes.len() as i32
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        if self.state.read().bnd_box.is_valid() {
            return self.state.read().bnd_box;
        }
        // do not apply the transformation because sensitives AABBs are already transformed
        for sensitive in &self.entities() {
            self.combine_member_bounds(sensitive);
        }
        self.state.read().bnd_box
    }
    fn center_of_geometry(&self) -> Point3 {
        // Cached native centers are not recomputed when children later change.
        self.state.read().center
    }
    fn set(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) {
        self.entity.fields.write().owner_id = owner.clone();
        for entity in self.entities().iter().filter_map(Handle::get) {
            entity.set(owner);
        }
    }
    fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        let group = Self::new(&self.owner_id(), self.must_match_all());
        let mut connected = Vec::new();
        for entity in self.entities().iter().filter_map(Handle::get) {
            connected.push(entity.get_connected()?);
        }
        group.add_all(&connected);
        Ok(Handle::new(group).cast())
    }
    fn bvh(&self) -> Result<(), Failure> {
        // Typed native failures cross nonfallible child hooks without swallowing Rust bugs.
        super::triangulation_failure::catch_triangulation_failure(|| select3d_set_build_bvh(self))
    }
    fn to_build_bvh(&self) -> bool {
        // Native Add/Remove/Clear deliberately leave this dirty flag unchanged.
        self.set.is_dirty()
    }
    /// `Clear`: forgets every entity (the owner is kept).
    fn clear(&self) {
        let mut state = self.state.write();
        state.nodes.clear();
        state.bnd_box.clear();
        state.center = Point3::new(0.0, 0.0, 0.0);
        state.bvh_prim_indexes.clear();
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        // The extracted writer keeps original child order and native field names.
        self.dump_group_json(stream, depth)
    }
}
