// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of the static helpers FillPointsHash, MakeNewWLine, MovePoint and
// DeleteOuterPoints of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_WLineTool.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The points hash of a walking line (delete, store, vertex), the line it rebuilds and
//! the removal of the points out of the domains (step II of `ComputePurgedWLine`).
use crate::int_patch::failures::sequence_value_mut;
use crate::int_patch::{IntPatchPoint, IntPatchPointLine, IntPatchWLine};
use crate::int_start::TheTopolTool;
use crate::int_start::failures::sequence_value;
use crate::int_surf::IntSurfLineOn2S;
use crate::intf::native_int_cast;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::gp::{self, Point2, Point3};

/// The hash codes of `FillPointsHash`.
pub(super) const DELETE: i32 = 1;
pub(super) const STORE: i32 = 0;
pub(super) const VERTEX: i32 = -1;

/// `NCollection_Array1<int>(1, NbPnts)`: the points hash, read and written 1-based with
/// the Standard_OutOfRange of the native array.
pub(super) struct PointsHash {
    items: Vec<i32>,
}

impl PointsHash {
    /// `thePointsHash(i)`.
    pub(super) fn get(&self, index: i32) -> Result<i32, Failure> {
        sequence_value(&self.items, index, "NCollection_Array1::Value").copied()
    }

    /// `thePointsHash.SetValue(i, value)`.
    pub(super) fn set(&mut self, index: i32, value: i32) -> Result<(), Failure> {
        *sequence_value_mut(&mut self.items, index, "NCollection_Array1::SetValue")? = value;
        Ok(())
    }

    /// `Upper()`.
    pub(super) fn upper(&self) -> i32 {
        self.items.len() as i32
    }
}

/// `FillPointsHash`: every point stored, the points of the vertices marked as vertices.
pub(super) fn fill_points_hash(wline: &IntPatchWLine) -> Result<PointsHash, Failure> {
    let nb: i32 = wline.nb_pnts()?;
    let mut hash = PointsHash {
        items: vec![STORE; usize::try_from(nb).unwrap_or_default()],
    };
    for v in 1..=wline.nb_vertex() {
        let vertex: IntPatchPoint = wline.vertex(v)?;
        hash.set(native_int_cast(vertex.parameter_on_line()), VERTEX)?;
    }
    Ok(hash)
}

/// The locals of the loop of `MakeNewWLine`.
struct NewLineBuilder<'a> {
    wline: &'a IntPatchWLine,
    purged: Handle<IntSurfLineOn2S>,
    local: Handle<IntPatchWLine>,
    is_outer: bool,
    old_line_idx: i32,
    vertex_idx: i32,
    index_prev: i32,
    idx_old: i32,
    p_prev: Point3,
    p_old: Point3,
}

impl NewLineBuilder<'_> {
    /// A point to store: kept when the distances to the last two stored points are
    /// even enough, otherwise a point is inserted in the longer segment. Returns the
    /// next value of the loop index minus one.
    fn store_point(&mut self, hash: &mut PointsHash, i: i32) -> Result<i32, Failure> {
        let p: Point3 = self.wline.point(i)?.value();
        let sq_dist_prev: f64 = self.p_prev.square_distance(&self.p_old);
        let sq_dist: f64 = self.p_prev.square_distance(&p);
        let ratio: f64 = if sq_dist_prev < gp::resolution() {
            0.0
        } else {
            9.0 * sq_dist / sq_dist_prev
        };
        if self.is_outer
            || ratio < gp::resolution()
            || (1.0 < ratio && ratio < 81.0)
            || i - self.index_prev <= 1
            || i - self.idx_old <= 1
        {
            // The difference in distances is satisfactory (1/9 < aSqDist/aSqDistPrev < 9).
            self.purged.add(&self.wline.point(i)?);
            self.old_line_idx += 1;
            (self.p_old, self.p_prev) = (self.p_prev, p);
            (self.idx_old, self.index_prev) = (self.index_prev, i);
            return Ok(i);
        }
        if sq_dist >= sq_dist_prev * 9.0 {
            // The current segment is much longer (aSqDist/aSqDistPrev >= 9).
            let mid: i32 = (i + self.index_prev) / 2;
            hash.set(mid, STORE)?;
            return Ok(mid - 1);
        }
        self.split_previous(hash, i)
    }

    /// The previous segment is much longer (aSqDist/aSqDistPrev <= 1/9): the previous
    /// point is replaced by a point between the last two stored ones.
    fn split_previous(&mut self, hash: &mut PointsHash, i: i32) -> Result<i32, Failure> {
        let mut next: i32 = i;
        if self.index_prev - self.idx_old > 1 {
            // Delete aPPrev from WL and insert a point between aPOld and aPPrev.
            self.purged.remove_point(self.purged.nb_points())?;
            self.old_line_idx -= 1;
            next = (self.idx_old + self.index_prev) / 2;
            hash.set(next, STORE)?;
            self.p_prev = self.p_old;
            self.index_prev = self.idx_old;
        } else {
            self.p_old = self.p_prev;
            self.idx_old = self.index_prev;
        }
        // Next iterations will start from this inserted point.
        Ok(next - 1)
    }

    /// A vertex: added with its new parameter on the line, with its point.
    fn add_vertex(&mut self, i: i32) -> Result<(), Failure> {
        let mut vertex: IntPatchPoint = self.wline.vertex(self.vertex_idx)?;
        self.vertex_idx += 1;
        vertex.set_parameter(f64::from(self.old_line_idx));
        self.old_line_idx += 1;
        self.local.add_vertex(&vertex, false);
        let point = self.wline.point(i)?;
        self.purged.add(&point);
        (self.p_prev, self.p_old) = (point.value(), point.value());
        (self.index_prev, self.idx_old) = (i, i);
        Ok(())
    }
}

/// `MakeNewWLine`: the walking line of the points and vertices the hash keeps; unless
/// `is_outer`, the kept points are evened out.
pub(super) fn make_new_wline(
    wline: &IntPatchWLine,
    hash: &mut PointsHash,
    is_outer: bool,
) -> Result<Handle<IntPatchWLine>, Failure> {
    let purged: Handle<IntSurfLineOn2S> = Handle::new(IntSurfLineOn2S::new(None));
    let local: Handle<IntPatchWLine> = Handle::new(IntPatchWLine::new_undecided(&purged, false));
    local.set_creating_way_info(wline.get_creating_way());
    let origin = Point3::new(0.0, 0.0, 0.0);
    let mut builder = NewLineBuilder {
        wline,
        purged,
        local,
        is_outer,
        old_line_idx: 1,
        vertex_idx: 1,
        index_prev: -1,
        idx_old: -1,
        p_prev: origin,
        p_old: origin,
    };
    let mut i: i32 = 1;
    while i <= hash.upper() {
        match hash.get(i)? {
            STORE => i = builder.store_point(hash, i)?,
            VERTEX => builder.add_vertex(i)?,
            // Other points will be rejected by purger.
            _ => {}
        }
        i += 1;
    }
    Ok(builder.local)
}

/// `MovePoint`: brings the parameters into the bounds of the surface (the walking takes
/// care of the closeness to the parametric space, so no interpolation is made).
fn move_point(s: &Adaptor3dSurfaceHandle, u: &mut f64, v: &mut f64) -> Result<(), Failure> {
    if *u < s.first_u_parameter()? {
        *u = s.first_u_parameter()?;
    }
    if *u > s.last_u_parameter()? {
        *u = s.last_u_parameter()?;
    }
    if *v < s.first_v_parameter()? {
        *v = s.first_v_parameter()?;
    }
    if *v > s.last_v_parameter()? {
        *v = s.last_v_parameter()?;
    }
    Ok(())
}

/// The surfaces and domains of the purge.
pub(super) struct PurgeInputs<'a> {
    pub(super) s1: &'a Adaptor3dSurfaceHandle,
    pub(super) s2: &'a Adaptor3dSurfaceHandle,
    pub(super) dom1: &'a dyn TheTopolTool,
    pub(super) dom2: &'a dyn TheTopolTool,
}

impl PurgeInputs<'_> {
    /// Whether the point `i` of the line is out of one of the domains.
    fn is_out(&self, wline: &IntPatchWLine, i: i32) -> Result<bool, Failure> {
        let (x1, y1, x2, y2) = wline.point(i)?.parameters();
        let state1: State =
            self.dom1
                .classify(Point2::new(x1, y1), precision::confusion(), true)?;
        let state2: State =
            self.dom2
                .classify(Point2::new(x2, y2), precision::confusion(), true)?;
        Ok(state1 == State::Out || state2 == State::Out)
    }

    /// The vertex `index` of `local` moves to the point `geom_idx` of `wline`, with its
    /// parameters brought into the bounds; the point `point_index` of `local` follows.
    fn move_end_vertex(
        &self,
        (wline, local): (&IntPatchWLine, &IntPatchWLine),
        index: i32,
        geom_idx: i32,
        point_index: i32,
    ) -> Result<(), Failure> {
        let mut vertex: IntPatchPoint = local.vertex(index)?;
        let geom = wline.point(geom_idx)?;
        vertex.set_value(geom.value());
        let (mut u1, mut v1, mut u2, mut v2) = geom.parameters();
        move_point(self.s1, &mut u1, &mut v1)?;
        move_point(self.s2, &mut u2, &mut v2)?;
        vertex.set_parameters(u1, v1, u2, v2);
        local.replace(index, &vertex)?;
        // Change point in walking line.
        local.set_point(point_index, &vertex)
    }
}

/// The result of the scan of one end of the line.
struct EndScan {
    changed: bool,
    geom_idx: i32,
    all_deleted: bool,
}

/// Marks the points out of the domains from the first point on, up to the first point
/// inside, which becomes a vertex.
fn scan_forward(
    inputs: &PurgeInputs<'_>,
    wline: &IntPatchWLine,
    hash: &mut PointsHash,
) -> Result<EndScan, Failure> {
    let mut scan = EndScan {
        changed: false,
        geom_idx: 1,
        all_deleted: true,
    };
    for i in 1..=wline.nb_pnts()? {
        if inputs.is_out(wline, i)? {
            hash.set(i, DELETE)?;
            scan.changed = true;
            continue;
        }
        scan.all_deleted = false;
        scan.geom_idx = (i - 1).max(1);
        if hash.get(i)? == VERTEX {
            // Use data what lies in (i) point / vertex.
            scan.geom_idx = i;
        }
        hash.set(i, VERTEX)?;
        break;
    }
    Ok(scan)
}

/// The same scan from the last point backward.
fn scan_backward(
    inputs: &PurgeInputs<'_>,
    wline: &IntPatchWLine,
    hash: &mut PointsHash,
) -> Result<EndScan, Failure> {
    let nb: i32 = wline.nb_pnts()?;
    let mut scan = EndScan {
        changed: false,
        geom_idx: nb,
        all_deleted: true,
    };
    for i in (1..=nb).rev() {
        if inputs.is_out(wline, i)? {
            hash.set(i, DELETE)?;
            // Move vertex to first good point.
            scan.changed = true;
            continue;
        }
        scan.all_deleted = false;
        scan.geom_idx = (i + 1).min(nb);
        if hash.get(i)? == VERTEX {
            // Use data what lies in (i) point / vertex.
            scan.geom_idx = i;
        }
        hash.set(i, VERTEX)?;
        break;
    }
    Ok(scan)
}

/// `DeleteOuterPoints`: the points out of the domains at both ends of the line are
/// removed and the end vertices moved to the first points inside (step II).
pub(super) fn delete_outer_points(
    wline: &Handle<IntPatchWLine>,
    inputs: &PurgeInputs<'_>,
) -> Result<Handle<IntPatchWLine>, Failure> {
    let mut hash: PointsHash = fill_points_hash(wline)?;
    if inputs.s1.is_u_periodic()?
        || inputs.s1.is_v_periodic()?
        || inputs.s2.is_u_periodic()?
        || inputs.s2.is_v_periodic()?
    {
        return Ok(wline.clone());
    }
    let first: EndScan = scan_forward(inputs, wline, &mut hash)?;
    if first.all_deleted {
        // ALL points are out of bounds: case boolean bcut_complex F5 and similar.
        return Ok(wline.clone());
    }
    let last: EndScan = scan_backward(inputs, wline, &mut hash)?;
    if !first.changed && !last.changed {
        // Nothing is done, return input.
        return Ok(wline.clone());
    }
    // Build new line and modify geometry of necessary vertices.
    let local: Handle<IntPatchWLine> = make_new_wline(wline, &mut hash, true)?;
    if first.changed {
        inputs.move_end_vertex((wline, &local), 1, first.geom_idx, 1)?;
    }
    if last.changed {
        let (nb_vertex, nb_pnts) = (local.nb_vertex(), local.nb_pnts()?);
        inputs.move_end_vertex((wline, &local), nb_vertex, last.geom_idx, nb_pnts)?;
    }
    Ok(local)
}
