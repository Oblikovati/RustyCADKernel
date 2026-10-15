// Rust translation of the steps shared by the SimulSurf and PerformSurf members of
// ChFi3d_FilBuilder and ChFi3d_ChBuilder (ChFi3d_FilBuilder.cxx, ChFi3d_ChBuilder.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The steps the fillet and chamfer builders repeat around a walking: the ends taken at
//! the ends of the spine, the sections stored as the simulation, the common points of
//! the ends of the line, and the obstacles kept at the ends.
use super::{ChFi3dBuilder, SurfCall, SurfOut};
use crate::brep_blend::BRepBlendLineHandle;
use crate::chfi_ds::{CircSection, SpineHandle, SurfDataHandle, lock};
use crate::chfi3d::fil_common_point;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};

/// The sections as a shared 1-based array (`NCollection_HArray1<ChFiDS_CircSection>`),
/// the simulation object of a surface data (null without sections).
pub(crate) fn sections_transient(
    sections: Vec<CircSection>,
) -> Result<Handle<StandardTransient>, Failure> {
    if sections.is_empty() {
        return Ok(Handle::default());
    }
    let range = |e| Failure::with_message("Standard_RangeError", &format!("{e:?}"));
    let mut a = Array1::with_bounds(1, sections.len() as i32).map_err(range)?;
    for (i, s) in (1..).zip(sections) {
        a.set_value(i, s).map_err(range)?;
    }
    Ok(Handle::new(HArray1::from_owned(a)).cast())
}

impl ChFi3dBuilder {
    /// `if (intf) First = sp->FirstParameter(1); if (intl) Last = ...`: the ends of a
    /// walking meeting obstacles taken at the ends of the spine.
    pub(crate) fn extend_ends_to_spine(
        &self,
        spine: &SpineHandle,
        out: &mut SurfOut,
    ) -> Result<(), Failure> {
        let g = lock(spine);
        if out.intf != 0 {
            out.first = g.first_parameter_of(1)?;
        }
        if out.intl != 0 {
            out.last = g.last_parameter_of(g.nb_edges())?;
        }
        Ok(())
    }

    /// The common points of the ends of the line (`ChFi3d_FilCommonPoint`): its first
    /// surface on the side `sides[0]` of the data, its second on `sides[1]`.
    pub(crate) fn store_line_ends(
        &self,
        data: &SurfDataHandle,
        lin: &BRepBlendLineHandle,
        sides: [i32; 2],
    ) -> Result<(), Failure> {
        let l = lock(lin);
        let mut d = lock(data);
        let tol = self.tolapp3d;
        let (t1, t2) = (l.transition_on_s1()?, l.transition_on_s2()?);
        fil_common_point(
            l.start_point_on_first(),
            t1,
            true,
            d.change_vertex(true, sides[0]),
            tol,
        )?;
        fil_common_point(
            l.end_point_on_first(),
            t1,
            false,
            d.change_vertex(false, sides[0]),
            tol,
        )?;
        fil_common_point(
            l.start_point_on_second(),
            t2,
            true,
            d.change_vertex(true, sides[1]),
            tol,
        )?;
        fil_common_point(
            l.end_point_on_second(),
            t2,
            false,
            d.change_vertex(false, sides[1]),
            tol,
        )
    }

    /// Whether an end of the walking stays an obstacle (`intf = !SearchFace(...)`):
    /// false when a face continues the contour at the common point on an arc of the
    /// first, else of the second face; `current` when neither is on an arc.
    pub(crate) fn obstacle_at_end(
        &self,
        data: &SurfDataHandle,
        call: &SurfCall<'_>,
        first: bool,
        current: i32,
    ) -> Result<i32, Failure> {
        let (cp1, cp2) = {
            let d = lock(data);
            (d.vertex(first, 1).clone(), d.vertex(first, 2).clone())
        };
        let mut result = current;
        let mut ok = false;
        let mut bid = Face::default();
        if cp1.is_on_arc() {
            let found = self.search_face(Some(call.spine), &cp1, &call.s1.face(), &mut bid)?;
            result = i32::from(!found);
            ok = result != 0;
        }
        if cp2.is_on_arc() && !ok {
            let found = self.search_face(Some(call.spine), &cp2, &call.s2.face(), &mut bid)?;
            result = i32::from(!found);
        }
        Ok(result)
    }
}
