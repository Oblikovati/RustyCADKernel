//! Entity lifecycle retains shared geometry/edges and fresh connected BVH state.
use super::sensitive_triangulation::Select3DSensitiveTriangulation;
use super::triangulation_failure::catch_triangulation_failure;
use super::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase,
    Select3DSensitiveEntityMembers as _, Select3DTypeOfSensitivity, select3d_set_build_bvh,
    select3d_set_dump_json, select3d_set_matches,
};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{GeneralTransform3, Point3};
use rustycad_tkmath::poly::Triangulation;
use rustycad_tkservice::json_dump::{base_class, class_begin, field_dumped, field_integer};
use std::io;
impl Select3DSensitiveEntity for Select3DSensitiveTriangulation {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        // Owner and sensitivity edits remain visible to connected construction.
        &self.entity
    }
    fn matches(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        catch_triangulation_failure(|| {
            if self.triangul.get().is_some_and(Triangulation::has_geometry) {
                return select3d_set_matches(self, manager, pick_result, false);
            }
            self.matches_box(manager, pick_result)
        })
    }
    fn nb_sub_elements(&self) -> i32 {
        // Node count is queried live; primitive count stays captured at construction.
        self.triangul.get().map_or(0, Triangulation::nb_nodes)
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        if !self.state.read().bnd_box.is_valid() {
            self.compute_bounding_box();
        }
        self.apply_transformation()
    }
    fn center_of_geometry(&self) -> Point3 {
        // Native COG remains local even when the initial location is nonidentity.
        self.cdg3d
    }
    fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        let edges = self.state.read().free_edges.clone();
        let is_interior = self.sens_type == Select3DTypeOfSensitivity::Interior;
        Ok(Handle::new(Self::with_free_edge_handle(
            &self.owner_id(),
            &self.triangul,
            &self.init_location,
            &edges,
            self.cdg3d,
            is_interior,
        )?)
        .cast())
    }
    fn bvh(&self) -> Result<(), Failure> {
        // Rebuilding reads current shared topology with the retained primitive ordering.
        catch_triangulation_failure(|| select3d_set_build_bvh(self))
    }
    fn to_build_bvh(&self) -> bool {
        // Queries must not eagerly build the tree.
        self.set.is_dirty()
    }
    fn clear(&self) {
        // Shared geometry survives the native inherited set Clear policy.
        // Discarding it here would change later selection behavior.
    }
    fn has_init_location(&self) -> bool {
        // Identity follows the native location chain rather than transformed coordinates.
        !self.init_location.is_identity()
    }
    fn inv_init_location(&self) -> GeneralTransform3 {
        // Viewer selection composes this inverse with the owning object transform.
        self.inv_init_location
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Select3D_SensitiveTriangulation")?;
        base_class(stream, depth, &|s, d| select3d_set_dump_json(self, s, d))?;
        if let Some(trg) = self.triangul.get() {
            field_dumped(stream, depth, "myTriangul.get()", &|s, _| trg.dump_json(s))?;
        }
        let location = self.init_location.clone();
        field_dumped(stream, depth, "&myInitLocation", &|s, d| {
            location.dump_json(s, d)
        })?;
        field_integer(stream, "mySensType", self.sens_type as i32)?;
        let (count, bounds) = {
            let state = self.state.read();
            (state.primitives_nb, state.bnd_box)
        };
        field_integer(stream, "myPrimitivesNb", count)?;
        field_dumped(stream, depth, "&myBndBox", &|s, _| bounds.dump_json(s))
    }
}
