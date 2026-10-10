//! `Select3D_SensitiveSegment`: a sensitive segment (sensitivity 3).
use super::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase,
    Select3DSensitiveEntityMembers as _, select3d_sensitive_entity_dump_json,
};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::{SelectMgrEntityOwnerSource, SelectMgrSelectionType};
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::bvh::{BvhBox, std_max, std_min};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::json_dump::{base_class, class_begin, field_dumped};
use std::io;

/// A sensitive segment.
///
/// Example: `let segment = Select3DSensitiveSegment::new(&owner, a, b); segment.set_end_point(c);`.
pub struct Select3DSensitiveSegment {
    transient: StandardTransient,
    base: Select3DSensitiveEntityBase,
    ends: SharedState<(Point3, Point3)>,
}
rustycad_tkernel::impl_transient!(Select3DSensitiveSegment, "Select3D_SensitiveSegment", dyn Select3DSensitiveEntity, transient,
    interfaces: [dyn Select3DSensitiveEntity]);

impl Select3DSensitiveSegment {
    /// A segment of an owner, with the sensitivity factor 3; `Select3DSensitiveSegment::new(&owner, a, b)`.
    pub fn new(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        first: Point3,
        last: Point3,
    ) -> Self {
        let entity = Self {
            transient: StandardTransient::default(),
            base: Select3DSensitiveEntityBase::new(owner),
            ends: SharedState::new((first, last)),
        };
        entity.base.fields.write().sensitivity_factor = 3;
        entity
    }
    /// `SetStartPoint` (and the deprecated `StartPoint(p)`); `segment.set_start_point(p)`.
    pub fn set_start_point(&self, point: Point3) {
        self.ends.write().0 = point;
    }
    /// `SetEndPoint` (and the deprecated `EndPoint(p)`); `segment.set_end_point(p)`.
    pub fn set_end_point(&self, point: Point3) {
        self.ends.write().1 = point;
    }
    /// `StartPoint`; `segment.start_point()`.
    pub fn start_point(&self) -> Point3 {
        self.ends.read().0
    }
    /// `EndPoint`; `segment.end_point()`.
    pub fn end_point(&self) -> Point3 {
        self.ends.read().1
    }
}

impl Select3DSensitiveEntity for Select3DSensitiveSegment {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        &self.base
    }
    /// `Matches`: both ends inside (the segment for a polyline) for the inclusion test, else
    /// the segment overlaps and the distance to the middle is computed.
    fn matches(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        let (start, end) = *self.ends.read();
        if !manager.is_overlap_allowed() {
            if manager.get_active_selection_type() == SelectMgrSelectionType::Polyline {
                return manager.overlaps_segment(&start, &end, pick_result);
            }
            return Ok(manager.overlaps_point(&start, pick_result)?
                && manager.overlaps_point(&end, pick_result)?);
        }
        if !manager.overlaps_segment(&start, &end, pick_result)? {
            return Ok(false);
        }
        pick_result
            .set_dist_to_geom_center(manager.dist_to_geometry_center(&self.center_of_geometry())?);
        Ok(true)
    }
    fn nb_sub_elements(&self) -> i32 {
        2
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        let (a, b) = *self.ends.read();
        let min = Vec3::new(
            std_min(a.x(), b.x()),
            std_min(a.y(), b.y()),
            std_min(a.z(), b.z()),
        );
        let max = Vec3::new(
            std_max(a.x(), b.x()),
            std_max(a.y(), b.y()),
            std_max(a.z(), b.z()),
        );
        BvhBox::from_corners(min, max)
    }
    fn center_of_geometry(&self) -> Point3 {
        let (a, b) = *self.ends.read();
        Point3::from((a.xyz() + b.xyz()) * 0.5)
    }
    /// `GetConnected`: a new segment of the same owner and ends (sensitivity 3).
    fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        let (a, b) = *self.ends.read();
        Ok(Handle::new(Self::new(&self.owner_id(), a, b)).cast())
    }
    fn to_build_bvh(&self) -> bool {
        false
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Select3D_SensitiveSegment")?;
        base_class(stream, depth, &|s, d| {
            select3d_sensitive_entity_dump_json(self, s, d)
        })?;
        let (a, b) = *self.ends.read();
        field_dumped(stream, depth, "&myStart", &|s, _| a.dump_json(s))?;
        field_dumped(stream, depth, "&myEnd", &|s, _| b.dump_json(s))?;
        let bounds = self.bounding_box();
        field_dumped(stream, depth, "&aBoundingBox", &|s, _| bounds.dump_json(s))
    }
}
