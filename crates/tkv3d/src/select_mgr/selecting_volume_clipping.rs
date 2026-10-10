//! Clip ownership and partial update timing of the selection manager.
use super::{SelectMgrSelectingVolumeManager, SelectMgrSelectionType, SelectMgrViewClipRange};
use crate::select_basics::SelectBasicsSelectingVolumeManager;
use rustycad_tkernel::standard::{Failure, Handle, dump::DumpStream};
use rustycad_tkmath::gp::Axis1;
use rustycad_tkservice::{
    graphic3d::Graphic3dSequenceOfHClipPlane,
    json_dump::{class_begin, field_dumped, field_integer, field_pointer},
};
use std::io;
impl SelectMgrSelectingVolumeManager {
    /// Shared view clip planes; `manager.view_clipping()`.
    pub fn view_clipping(&self) -> &Handle<Graphic3dSequenceOfHClipPlane> {
        &self.view_planes
    }
    /// Shared object clip planes; `manager.object_clipping()`.
    pub fn object_clipping(&self) -> &Handle<Graphic3dSequenceOfHClipPlane> {
        &self.object_planes
    }
    /// Current depth clipping; `manager.view_clip_ranges()`.
    pub fn view_clip_ranges(&self) -> &SelectMgrViewClipRange {
        &self.clip_range
    }
    /// Replace only the range value; `manager.set_view_clip_ranges(&range)`.
    pub fn set_view_clip_ranges(&mut self, range: &SelectMgrViewClipRange) {
        self.clip_range = range.clone();
    }
    /// Copy both plane handles and range; `manager.copy_view_clipping(&other)`.
    pub fn copy_view_clipping(&mut self, other: &Self) {
        self.view_planes = other.view_planes.clone();
        self.object_planes = other.object_planes.clone();
        self.clip_range = other.clip_range.clone();
    }
    /// Replace planes and recompute Point ranges using an optional world-space ray.
    /// Example: `manager.set_view_clipping(&view, &object, Some(&world))?`.
    pub fn set_view_clipping(
        &mut self,
        view: &Handle<Graphic3dSequenceOfHClipPlane>,
        object: &Handle<Graphic3dSequenceOfHClipPlane>,
        world: Option<&Self>,
    ) -> Result<(), Failure> {
        self.view_planes = view.clone();
        self.object_planes = object.clone();
        if self.get_active_selection_type() != SelectMgrSelectionType::Point {
            return Ok(());
        }
        self.clip_range.set_void();
        self.add_volume_clipping(view, world)?;
        self.add_volume_clipping(object, world)
    }
    fn add_volume_clipping(
        &mut self,
        planes: &Handle<Graphic3dSequenceOfHClipPlane>,
        world: Option<&Self>,
    ) -> Result<(), Failure> {
        let Some(planes) = planes.get().filter(|planes| !planes.is_empty()) else {
            return Ok(());
        };
        let volume = world
            .unwrap_or(self)
            .required_volume("SetViewClipping world ray")?;
        let ray = Axis1::new(volume.get_near_pnt()?, volume.get_view_ray_direction()?);
        self.clip_range.add_clipping_planes(planes, &ray)
    }
    pub(super) fn dump_selecting_volume(
        &self,
        stream: &mut DumpStream,
        depth: i32,
    ) -> io::Result<()> {
        class_begin(stream, "SelectMgr_SelectingVolumeManager")?;
        field_pointer(
            stream,
            "myActiveSelectingVolume.get()",
            self.active.object_address() as usize,
        )?;
        field_pointer(
            stream,
            "myViewClipPlanes.get()",
            self.view_planes.object_address() as usize,
        )?;
        field_pointer(
            stream,
            "myObjectClipPlanes.get()",
            self.object_planes.object_address() as usize,
        )?;
        field_dumped(stream, depth, "&myViewClipRange", &|stream, depth| {
            self.clip_range.dump_json(stream, depth)
        })?;
        field_integer(stream, "myToAllowOverlap", self.allow_overlap as i32)
    }
}
