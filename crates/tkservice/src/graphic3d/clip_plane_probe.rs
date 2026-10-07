//! The point and box tests of `Graphic3d_ClipPlane` (one half-space, and the union of a
//! chain) and its `DumpJson`.
use super::{Graphic3dAspectsSource, Graphic3dBndBox3d, Graphic3dClipPlane, Graphic3dClipState};
use crate::json_dump::{
    class_begin, field_dumped, field_integer, field_pointer, field_string, field_unsigned,
};
use rustycad_tkernel::ncollection::Vec4;
use rustycad_tkernel::standard::dump::{DumpStream, dump_vector};
use std::io;

impl Graphic3dClipPlane {
    /// Calls `visit` on this plane and on each following plane of the chain until it
    /// answers `Some`.
    fn find_in_chain<R>(&self, mut visit: impl FnMut(&Self) -> Option<R>) -> Option<R> {
        if let Some(found) = visit(self) {
            return Some(found);
        }
        let mut cursor = self.chain_next_plane();
        while let Some(plane) = cursor.get() {
            if let Some(found) = visit(plane) {
                return Some(found);
            }
            let following = plane.chain_next_plane();
            cursor = following;
        }
        None
    }
    /// The state of a point against the chain union: `In` when any plane keeps it, else `On`
    /// when any plane touches it, else `Out`. Example: `plane.probe_point(&Vec4::new(0.0, 0.0, 1.0, 1.0))`.
    pub fn probe_point(&self, point: &Vec4<f64>) -> Graphic3dClipState {
        let mut state = Graphic3dClipState::Out;
        let inside = self.find_in_chain(|plane| match plane.probe_point_halfspace(point) {
            Graphic3dClipState::In => Some(Graphic3dClipState::In),
            Graphic3dClipState::On => {
                state = Graphic3dClipState::On;
                None
            }
            Graphic3dClipState::Out => None,
        });
        inside.unwrap_or(state)
    }
    /// The state of a box against the chain union; `plane.probe_box(&bounds)`.
    pub fn probe_box(&self, bounds: &Graphic3dBndBox3d) -> Graphic3dClipState {
        let mut state = Graphic3dClipState::Out;
        let inside = self.find_in_chain(|plane| {
            if plane.is_box_full_in_halfspace(bounds) {
                return Some(Graphic3dClipState::In);
            }
            if !plane.is_box_full_out_halfspace(bounds) {
                state = Graphic3dClipState::On;
            }
            None
        });
        inside.unwrap_or(state)
    }
    /// Whether the box is cut by the chain. Each partially cut plane re-tests the maximum
    /// point against THIS plane, not the iterated one, as natively. Example: `plane.probe_box_touch(&bounds)`.
    pub fn probe_box_touch(&self, bounds: &Graphic3dBndBox3d) -> bool {
        let touches = self.find_in_chain(|plane| {
            if plane.is_box_full_in_halfspace(bounds) {
                return Some(false);
            }
            let cut = !plane.is_box_full_out_halfspace(bounds)
                && self.probe_box_max_point_halfspace(bounds) != Graphic3dClipState::Out;
            cut.then_some(true)
        });
        touches.unwrap_or(false)
    }
    /// The state of a point against this plane only; `plane.probe_point_halfspace(&point)`.
    pub fn probe_point_halfspace(&self, point: &Vec4<f64>) -> Graphic3dClipState {
        let value = self.state.read().equation.dot(point);
        if value < 0.0 {
            Graphic3dClipState::Out
        } else if value == 0.0 {
            Graphic3dClipState::On
        } else {
            Graphic3dClipState::In
        }
    }
    /// The state of a box against this plane only; `plane.probe_box_halfspace(&bounds)`.
    pub fn probe_box_halfspace(&self, bounds: &Graphic3dBndBox3d) -> Graphic3dClipState {
        if self.is_box_full_out_halfspace(bounds) {
            return Graphic3dClipState::Out;
        }
        if self.is_box_full_in_halfspace(bounds) {
            Graphic3dClipState::In
        } else {
            Graphic3dClipState::On
        }
    }
    /// Whether the point is clipped by this plane; `plane.is_point_out_halfspace(&point)`.
    pub fn is_point_out_halfspace(&self, point: &Vec4<f64>) -> bool {
        self.probe_point_halfspace(point) == Graphic3dClipState::Out
    }
    /// The box corner furthest along the normal (`toward_max`) or against it.
    fn extreme_corner(&self, bounds: &Graphic3dBndBox3d, toward_max: bool) -> Vec4<f64> {
        let equation = self.state.read().equation;
        let (min, max) = (bounds.corner_min(), bounds.corner_max());
        let pick = |positive: bool, low: f64, high: f64| {
            if positive == toward_max { high } else { low }
        };
        Vec4::new(
            pick(equation.x > 0.0, min.x, max.x),
            pick(equation.y > 0.0, min.y, max.y),
            pick(equation.z > 0.0, min.z, max.z),
            1.0,
        )
    }
    /// Whether the whole box is clipped by this plane; `plane.is_box_full_out_halfspace(&bounds)`.
    pub fn is_box_full_out_halfspace(&self, bounds: &Graphic3dBndBox3d) -> bool {
        self.is_point_out_halfspace(&self.extreme_corner(bounds, true))
    }
    /// The state of the box corner furthest along the normal.
    /// Example: `plane.probe_box_max_point_halfspace(&bounds)`.
    pub fn probe_box_max_point_halfspace(&self, bounds: &Graphic3dBndBox3d) -> Graphic3dClipState {
        self.probe_point_halfspace(&self.extreme_corner(bounds, true))
    }
    /// Whether the whole box is kept by this plane; `plane.is_box_full_in_halfspace(&bounds)`.
    pub fn is_box_full_in_halfspace(&self, bounds: &Graphic3dBndBox3d) -> bool {
        !self.is_point_out_halfspace(&self.extreme_corner(bounds, false))
    }
    /// `DumpJson`; `plane.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let s = self.snapshot();
        class_begin(stream, "Graphic3d_ClipPlane")?;
        field_pointer(stream, "this", std::ptr::from_ref(self) as usize)?;
        field_dumped(stream, depth, "myAspect.get()", &|o, d| {
            s.aspect.dump_json(o, d)
        })?;
        field_pointer(
            stream,
            "myNextInChain.get()",
            s.next.object_address() as usize,
        )?;
        field_pointer(stream, "myPrevInChain", s.prev.object_address() as usize)?;
        field_string(stream, "myId", s.id.as_bytes())?;
        field_dumped(stream, depth, "&myPlane", &|o, d| s.plane.dump_json(o, d))?;
        let [e, r] = [s.equation, s.equation_rev];
        dump_vector(stream, "Equation", &[e.x, e.y, e.z, e.w])?;
        dump_vector(stream, "EquationRev", &[r.x, r.y, r.z, r.w])?;
        field_integer(stream, "myChainLenFwd", s.chain_len_fwd)?;
        field_integer(stream, "myFlags", s.flags)?;
        field_unsigned(stream, "myEquationMod", u64::from(s.equation_mod))?;
        field_unsigned(stream, "myAspectMod", u64::from(s.aspect_mod))?;
        field_integer(stream, "myIsOn", i32::from(s.is_on))?;
        field_integer(stream, "myIsCapping", i32::from(s.is_capping))
    }
}
