// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntSurf/IntSurf_LineOn2S.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntSurf_LineOn2S`: a polyline of points on two surfaces with lazy bounding boxes.
use super::IntSurfPntOn2S;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{BaseAllocator, Sequence};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::bnd::{BoundingBox2, BoundingBox3};
use rustycad_tkmath::gp::{Point2, Point3};

/// The locked fields: the points and the three boxes, which are Whole while stale.
struct LineOn2SState {
    seq: Sequence<IntSurfPntOn2S>,
    buv1: BoundingBox2,
    buv2: BoundingBox2,
    bxyz: BoundingBox3,
}

/// A line of `IntSurf_PntOn2S` points (`IntSurf_LineOn2S`), shared through handles.
///
/// The three bounding boxes (3d, first and second parametric space) are built lazily by
/// the `IsOut*` queries; a box is Whole while it is stale, as natively, and each edit
/// either extends a built box or marks it stale.
/// `let line = IntSurfLineOn2S::new(None); line.add(&IntSurfPntOn2S::new());`
pub struct IntSurfLineOn2S {
    parent: StandardTransient,
    state: SharedState<LineOn2SState>,
}
impl_transient!(
    IntSurfLineOn2S,
    "IntSurf_LineOn2S",
    StandardTransient,
    parent
);

impl Default for IntSurfLineOn2S {
    fn default() -> Self {
        Self::new(None)
    }
}

/// The Standard_OutOfRange of an index outside `1..=NbPoints`.
fn out_of_range(member: &str, index: i32, length: i32) -> Failure {
    Failure::with_message(
        "Standard_OutOfRange",
        &format!("IntSurf_LineOn2S::{member}: index {index} outside 1..={length}"),
    )
}

/// The one-based sequence index of `index`, or Standard_OutOfRange.
fn checked_index(member: &str, index: i32, length: i32) -> Result<usize, Failure> {
    if index < 1 || index > length {
        return Err(out_of_range(member, index, length));
    }
    Ok(index as usize)
}

impl LineOn2SState {
    fn whole_boxes(&mut self) {
        self.buv1.set_whole();
        self.buv2.set_whole();
        self.bxyz.set_whole();
    }

    /// Extends each built box with `p`, as `Add` and `InsertBefore` do.
    fn extend_boxes(&mut self, p: &IntSurfPntOn2S) {
        if !self.bxyz.is_whole() {
            self.bxyz.add_point(p.value());
        }
        if !self.buv1.is_whole() {
            self.buv1.add_point(p.value_on_surface(true));
        }
        if !self.buv2.is_whole() {
            self.buv2.add_point(p.value_on_surface(false));
        }
    }

    fn point(&mut self, member: &str, index: i32) -> Result<&mut IntSurfPntOn2S, Failure> {
        let at: usize = checked_index(member, index, self.seq.length())?;
        self.seq.change_value(at)
    }
}

impl IntSurfLineOn2S {
    /// An empty line; `None` is the native null allocator (`IntSurf_LineOn2S(theAllocator)`).
    /// `IntSurfLineOn2S::new(None).nb_points() == 0`.
    pub fn new(allocator: Option<BaseAllocator>) -> Self {
        let seq: Sequence<IntSurfPntOn2S> = match allocator {
            Some(resource) => Sequence::with_allocator(resource),
            None => Sequence::default(),
        };
        let mut state = LineOn2SState {
            seq,
            buv1: BoundingBox2::default(),
            buv2: BoundingBox2::default(),
            bxyz: BoundingBox3::default(),
        };
        state.whole_boxes();
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
        }
    }

    /// Appends a point (`Add`); `line.add(&point)`.
    pub fn add(&self, p: &IntSurfPntOn2S) {
        let mut state = self.state.write();
        state.seq.append(*p);
        state.extend_boxes(p);
    }

    /// The number of points (`NbPoints`); `line.nb_points()`.
    pub fn nb_points(&self) -> i32 {
        self.state.read().seq.length()
    }

    /// The point of rank `index` (`Value(Index)`), a copy because the line is locked;
    /// Standard_OutOfRange outside `1..=NbPoints`. `line.value(1)?.value()`.
    pub fn value(&self, index: i32) -> Result<IntSurfPntOn2S, Failure> {
        let state = self.state.read();
        let at: usize = checked_index("Value", index, state.seq.length())?;
        state.seq.value(at).copied()
    }

    /// Reverses the order of the points (`Reverse`); the boxes do not change.
    /// `line.reverse()`.
    pub fn reverse(&self) {
        self.state.write().seq.reverse();
    }

    /// Keeps the points `1..Index-1` and returns a new line with the others (`Split`);
    /// the new line gets the default allocator, as natively, and this line keeps its boxes.
    /// `let tail = line.split(2)?;`
    pub fn split(&self, index: i32) -> Result<Handle<IntSurfLineOn2S>, Failure> {
        let mut tail: Sequence<IntSurfPntOn2S> = Sequence::default();
        self.state
            .write()
            .seq
            .split(index.max(0) as usize, &mut tail)?;
        let split = IntSurfLineOn2S::new(None);
        for p in tail.iter() {
            split.add(p);
        }
        Ok(Handle::new(split))
    }

    /// Replaces the point of rank `index` (`Value(Index, P)`) and marks every box stale.
    /// `line.set_value(2, &point)?`.
    pub fn set_value(&self, index: i32, p: &IntSurfPntOn2S) -> Result<(), Failure> {
        let mut state = self.state.write();
        *state.point("Value", index)? = *p;
        state.whole_boxes();
        Ok(())
    }

    /// Replaces the 3d point of rank `index` (`SetPoint`) and marks every box stale.
    /// `line.set_point(2, Point3::new(1.0, 0.0, 0.0))?`.
    pub fn set_point(&self, index: i32, pnt: Point3) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.point("SetPoint", index)?.set_point(pnt);
        state.whole_boxes();
        Ok(())
    }

    /// Replaces the parameters of rank `index` on one surface (`SetUV`) and extends
    /// that surface's box when it is built. `line.set_uv(1, true, 0.5, 0.5)?`.
    pub fn set_uv(&self, index: i32, on_first: bool, u: f64, v: f64) -> Result<(), Failure> {
        let mut state = self.state.write();
        state
            .point("SetUV", index)?
            .set_parameters_on(on_first, u, v);
        if on_first && !state.buv1.is_whole() {
            state.buv1.add_point(Point2::new(u, v));
        } else if !on_first && !state.buv2.is_whole() {
            state.buv2.add_point(Point2::new(u, v));
        }
        Ok(())
    }

    /// Removes every point and marks the boxes stale (`Clear`); `line.clear()`.
    pub fn clear(&self) {
        let mut state = self.state.write();
        state.seq.clear(None);
        state.whole_boxes();
    }

    /// Inserts `p` before rank `index`, or appends it when `index` is past the end
    /// (`InsertBefore`); built boxes are extended. `line.insert_before(1, &point)?`.
    pub fn insert_before(&self, index: i32, p: &IntSurfPntOn2S) -> Result<(), Failure> {
        let mut state = self.state.write();
        if index > state.seq.length() {
            state.seq.append(*p);
        } else {
            let at: usize = checked_index("InsertBefore", index, state.seq.length())?;
            state.seq.insert_before(at, *p)?;
        }
        state.extend_boxes(p);
        Ok(())
    }

    /// Removes the point of rank `index` and marks the boxes stale (`RemovePoint`).
    /// `line.remove_point(1)?`.
    pub fn remove_point(&self, index: i32) -> Result<(), Failure> {
        let mut state = self.state.write();
        let at: usize = checked_index("RemovePoint", index, state.seq.length())?;
        state.seq.remove(at)?;
        state.whole_boxes();
        Ok(())
    }
}

/// Enlarges a built parametric box by 1% of its larger extent, as `IsOutSurf1Box` does.
fn enlarge_uv_box(uv_box: &mut BoundingBox2) {
    // The box holds at least one point here, so it is not void.
    let [pu1, pv1, mut pu2, mut pv2] = uv_box.get().unwrap_or_else(|_| unreachable!());
    pu2 -= pu1;
    pv2 -= pv1;
    if pu2 > pv2 {
        uv_box.enlarge(pu2 * 0.01);
    } else {
        uv_box.enlarge(pv2 * 0.01);
    }
}

/// Enlarges the built 3d box by 1% of its largest extent, as `IsOutBox` does.
fn enlarge_xyz_box(xyz_box: &mut BoundingBox3) {
    let limits = xyz_box.limits();
    let x1: f64 = limits.x_max - limits.x_min;
    let y1: f64 = limits.y_max - limits.y_min;
    let z1: f64 = limits.z_max - limits.z_min;
    let largest: f64 = if x1 > y1 {
        if x1 > z1 { x1 } else { z1 }
    } else if y1 > z1 {
        y1
    } else {
        z1
    };
    xyz_box.enlarge(largest * 0.01);
}

impl LineOn2SState {
    /// Rebuilds a stale parametric box from the points of one surface.
    fn build_uv_box(&mut self, on_first: bool) {
        let mut uv_box = BoundingBox2::default();
        uv_box.set_void();
        for p in self.seq.iter() {
            uv_box.add_point(p.value_on_surface(on_first));
        }
        enlarge_uv_box(&mut uv_box);
        if on_first {
            self.buv1 = uv_box;
        } else {
            self.buv2 = uv_box;
        }
    }
}

impl IntSurfLineOn2S {
    /// Whether `p1uv` is outside the enlarged box of the parameters on the first
    /// surface (`IsOutSurf1Box`); an empty line never excludes. `line.is_out_surf1_box(uv)`.
    pub fn is_out_surf1_box(&self, p1uv: Point2) -> bool {
        self.is_out_uv_box(true, p1uv)
    }

    /// Whether `p2uv` is outside the enlarged box of the parameters on the second
    /// surface (`IsOutSurf2Box`). `line.is_out_surf2_box(uv)`.
    pub fn is_out_surf2_box(&self, p2uv: Point2) -> bool {
        self.is_out_uv_box(false, p2uv)
    }

    fn is_out_uv_box(&self, on_first: bool, uv: Point2) -> bool {
        let mut state = self.state.write();
        if state.seq.is_empty() {
            return false;
        }
        let stale: bool = if on_first {
            state.buv1.is_whole()
        } else {
            state.buv2.is_whole()
        };
        if stale {
            state.build_uv_box(on_first);
        }
        let uv_box: &BoundingBox2 = if on_first { &state.buv1 } else { &state.buv2 };
        uv_box.is_out_point(uv)
    }

    /// Whether `pxyz` is outside the enlarged box of the 3d points (`IsOutBox`); an
    /// empty line never excludes. `line.is_out_box(Point3::new(9.0, 9.0, 9.0))`.
    pub fn is_out_box(&self, pxyz: Point3) -> bool {
        let mut state = self.state.write();
        if state.seq.is_empty() {
            return false;
        }
        if state.bxyz.is_whole() {
            let mut xyz_box = BoundingBox3::default();
            xyz_box.set_void();
            for p in state.seq.iter() {
                xyz_box.add_point(p.value());
            }
            enlarge_xyz_box(&mut xyz_box);
            state.bxyz = xyz_box;
        }
        state.bxyz.is_out_point(pxyz)
    }
}
