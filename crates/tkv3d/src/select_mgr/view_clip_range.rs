//! `SelectMgr_ViewClipRange`: the depth ranges along the picking ray that the clipping planes
//! keep (the unclipped range) or cut (the chains' sub-ranges).
use rustycad_tkernel::precision::angular;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::real::{real_first, real_last};
use rustycad_tkmath::bnd::Range1;
use rustycad_tkmath::gp::{Axis1, Point3};
use rustycad_tkservice::graphic3d::Graphic3dSequenceOfHClipPlane;
use rustycad_tkservice::json_dump::{class_begin, field_dumped};
use std::io;

/// The range `[RealFirst, RealLast]`.
fn whole_range() -> Range1 {
    // The bounds are ordered, so the constructor check cannot fail.
    Range1::new(real_first(), real_last()).unwrap_or_default()
}

/// The clipped and unclipped depths of a picking ray.
///
/// Example: `let mut range = SelectMgrViewClipRange::default(); range.is_clipped(1.0);`.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectMgrViewClipRange {
    clip_ranges: Vec<Range1>,
    unclip_range: Range1,
}

impl Default for SelectMgrViewClipRange {
    /// `SelectMgr_ViewClipRange()`: void (nothing clipped).
    fn default() -> Self {
        Self {
            clip_ranges: Vec::new(),
            unclip_range: whole_range(),
        }
    }
}

impl SelectMgrViewClipRange {
    /// `IsClipped`: out of the unclipped range or inside a clipped one; `range.is_clipped(2.0)`.
    pub fn is_clipped(&self, depth: f64) -> bool {
        self.unclip_range.is_out(depth) || self.clip_ranges.iter().any(|r| !r.is_out(depth))
    }
    /// `GetNearestDepth`: the nearest unclipped depth of a range, None when it is clipped.
    /// Example: `range.get_nearest_depth(&depths)`.
    pub fn get_nearest_depth(&self, range: &Range1) -> Option<f64> {
        let mut depth = 0.0;
        self.get_nearest_depth_into(range, &mut depth)
            .then_some(depth)
    }
    /// Preserves the native output argument even on failure or a void query.
    /// Example: `let found = clip.get_nearest_depth_into(&query, &mut depth);`.
    pub fn get_nearest_depth_into(&self, range: &Range1, depth: &mut f64) -> bool {
        if !self.unclip_range.is_void() && self.unclip_range.is_out_range(*range) {
            return false;
        }
        range.get_min(depth);
        if !self.unclip_range.is_void() && self.unclip_range.is_out(*depth) {
            self.unclip_range.get_min(depth);
        }
        let Some(common) = self.common_clip_range(*depth) else {
            return true;
        };
        common.get_max(depth);
        !range.is_out(*depth)
    }
    // OCCT 3d097a0 merges once in insertion order, rather than computing a fixed point.
    fn common_clip_range(&self, depth: f64) -> Option<Range1> {
        let mut common = *self.clip_ranges.iter().find(|r| !r.is_out(depth))?;
        for clip in &self.clip_ranges {
            if !common.is_out_range(*clip) {
                common.add_range(*clip);
            }
        }
        Some(common)
    }
    /// `SetVoid`: nothing clipped; `range.set_void()`.
    pub fn set_void(&mut self) {
        self.clip_ranges.clear();
        self.unclip_range = whole_range();
    }
    /// `AddClippingPlanes`: the active planes cut the ray into the unclipped range and, for
    /// each chain, a clipped sub-range. Example: `range.add_clipping_planes(&planes, &ray)?`.
    pub fn add_clipping_planes(
        &mut self,
        planes: &Graphic3dSequenceOfHClipPlane,
        pick_ray: &Axis1,
    ) -> Result<(), Failure> {
        for handle in planes.items() {
            let Some(plane) = handle.get().filter(|p| p.is_on()) else {
                continue;
            };
            let mut sub_range = whole_range();
            let mut chain = handle.clone();
            while let Some(current) = chain.get() {
                if let Some((dist, dot)) = plane_distance(current.to_plane(), pick_ray)? {
                    self.trim(plane.is_chain(), &mut sub_range, dist, dot);
                }
                let next = current.chain_next_plane();
                chain = next;
            }
            if !sub_range.is_void() && plane.is_chain() {
                self.clip_ranges.push(sub_range);
            }
        }
        Ok(())
    }
    /// Trims the unclipped range (a plane) or the chain sub-range at a ray distance.
    fn trim(&mut self, is_chain: bool, sub_range: &mut Range1, dist_to_pln: f64, dot: f64) {
        match (is_chain, dot < 0.0) {
            (false, true) => self.unclip_range.trim_to(dist_to_pln),
            (false, false) => self.unclip_range.trim_from(dist_to_pln),
            (true, true) => sub_range.trim_from(dist_to_pln),
            (true, false) => sub_range.trim_to(dist_to_pln),
        }
    }
    /// `ChangeUnclipRange`; `range.change_unclip_range().trim_to(5.0)`.
    pub fn change_unclip_range(&mut self) -> &mut Range1 {
        &mut self.unclip_range
    }
    /// `AddClipSubRange`; `range.add_clip_sub_range(sub)`.
    pub fn add_clip_sub_range(&mut self, range: Range1) {
        self.clip_ranges.push(range);
    }
    /// `DumpJson`: the clipped ranges and the unclipped one; `range.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "SelectMgr_ViewClipRange")?;
        for clip in &self.clip_ranges {
            field_dumped(stream, depth, "&aClipRange", &|s, _| clip.dump_json(s))?;
        }
        let unclip = self.unclip_range;
        field_dumped(stream, depth, "&myUnclipRange", &|s, _| unclip.dump_json(s))
    }
}

/// The signed ray distance to a plane and the ray-normal dot product (1 for a parallel ray,
/// at RealLast); None for a parallel ray on the clipped side.
fn plane_distance(
    plane: rustycad_tkmath::gp::Plane3,
    pick_ray: &Axis1,
) -> Result<Option<(f64, f64)>, Failure> {
    let ray_dir = pick_ray.direction().xyz();
    let near = pick_ray.location();
    let [_, _, _, d] = plane.coefficients()?;
    let plane_dir = plane.axis().direction().xyz();
    let dot = ray_dir.dot(plane_dir);
    let distance = -near.xyz().dot(plane_dir) - d;
    if dot.abs() < angular() {
        return Ok((distance >= 0.0).then_some((real_last(), 1.0)));
    }
    let param = distance / dot;
    let point = Point3::from(near.xyz() + ray_dir * param);
    let dist = point.distance(&near);
    Ok(Some((if param < 0.0 { -dist } else { dist }, dot)))
}
