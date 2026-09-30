// Rust translation of the Perform members of ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_IWalking.gxx
// ("IntWalk_IWalking_1.gxx") at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Perform`: loads the start points into the walking data, then walks the open lines
//! from the path points and the closed lines from the interior points.
//!
//! The states of the walking data: a path point is 12 (not tangent, not passing),
//! 11 (tangent, not passing), 2 (not tangent, passing) or 1 (tangent, passing); an
//! interior point is 13 on a closed line and 12 on a line that opened. A processed
//! point becomes negative.
use super::{DUMMY, IWalkingTypes, IntWalkIWalking, TheIWFunction};
use crate::int_imp::ThePSurfaceTool;
use crate::int_surf::{
    IntSurfInteriorPoint, IntSurfInteriorPointTool, IntSurfPathPoint, IntSurfPathPointTool,
};
use crate::int_walk::IntWalkWalkingData;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::bnd::Range1;
use rustycad_tkmath::math::Vector;

/// The multiplicity parameters of the path points, in order (`Umult`, `Vmult`).
pub(super) struct Multiples {
    pub(super) u: Vec<f64>,
    pub(super) v: Vec<f64>,
}

impl Multiples {
    /// `Umult(j)`, `Vmult(j)`: 1-based, Standard_OutOfRange outside.
    pub(super) fn at(&self, j: i32) -> Result<(f64, f64), Failure> {
        let u: f64 = *super::sequence_value(&self.u, j, "Umult")?;
        Ok((u, self.v[j as usize - 1]))
    }
}

/// `IsTangentExtCheck`: whether the function vanishes (within its tolerance) at the four
/// neighbours of `(u, v)` one step away, clamped to the domain; such a point allows any
/// direction and its closed line is not walked.
fn is_tangent_ext_check<F: TheIWFunction>(
    func: &mut F,
    (u, v): (f64, f64),
    (step_u, step_v): (f64, f64),
    [uinf, usup, vinf, vsup]: [f64; 4],
) -> Result<bool, Failure> {
    let tol: f64 = func.tolerance();
    let par_u: [f64; 4] = [std_min(u + step_u, usup), std_max(u - step_u, uinf), u, u];
    let par_v: [f64; 4] = [v, v, std_min(v + step_v, vsup), std_max(v - step_v, vinf)];
    let mut x: Vector<'static> = Vector::with_bounds(1, 2)?;
    let mut val: Vector<'static> = Vector::with_bounds(1, 1)?;
    for i in 0..4 {
        (x[1], x[2]) = (par_u[i], par_v[i]);
        if !func.checked_value(&x, &mut val)? {
            continue;
        }
        if func.root().abs() > tol {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The walking data of a path point: its state and start parameters.
fn path_walking_data(point: &IntSurfPathPoint) -> Result<IntWalkWalkingData, Failure> {
    let mut etat: i32 = 1;
    if !IntSurfPathPointTool::is_passing_pnt(point) {
        etat = 11;
    }
    if !IntSurfPathPointTool::is_tangent(point) {
        etat += 1;
    }
    let (ustart, vstart) = IntSurfPathPointTool::value2d(point)?;
    Ok(IntWalkWalkingData {
        ustart,
        vstart,
        etat,
    })
}

impl<T: IWalkingTypes> IntWalkIWalking<T> {
    /// `Perform(Pnts1, Pnts2, Func, Caro, Reversed)`: walks the open lines from the
    /// path points `pnts1` and the closed lines from the interior points `pnts2`, on
    /// the surface `caro` (the second of the pair, or the first when `reversed`).
    ///
    /// `walk.perform(&paths, &interiors, &mut func, &surface, false)?`.
    pub fn perform(
        &mut self,
        pnts1: &[IntSurfPathPoint],
        pnts2: &[IntSurfInteriorPoint],
        func: &mut T::Function,
        caro: &<T::SurfaceTool as ThePSurfaceTool>::Surface,
        reversed: bool,
    ) -> Result<(), Failure> {
        let mut rajout: bool = false;
        self.clear();
        self.reversed = reversed;
        self.load_bounds(caro)?;
        let steps: (f64, f64) = (
            self.pas * (self.um_last - self.um),
            self.pas * (self.vm_last - self.vm),
        );
        let mut mult = Multiples {
            u: Vec::new(),
            v: Vec::new(),
        };
        self.load_path_points(pnts1, &mut mult, true)?;
        self.load_interior_points(pnts2, func, steps)?;
        self.load_tolerance(caro)?;
        func.set_surface(caro);
        self.estimate_ranges()?;
        if !pnts1.is_empty() {
            self.compute_open_line(&mult, pnts1, func, &mut rajout)?;
        }
        if !pnts2.is_empty() {
            self.compute_close_line(&mult, pnts1, pnts2, func, &mut rajout)?;
        }
        if self.to_fill_holes {
            self.fill_holes(&mult, pnts1, func, &mut rajout)?;
        }
        self.finish(pnts1);
        Ok(())
    }
    /// `Perform(Pnts1, Func, Caro, Reversed)`: the open lines only. The native member
    /// neither clears nor resets the walking data, so on a fresh object it reads past
    /// them (undefined natively, Standard_OutOfRange here); after another Perform it
    /// reads the earlier entries, as natively. `walk.perform_open(&paths, &mut func, &surface, false)?`.
    pub fn perform_open(
        &mut self,
        pnts1: &[IntSurfPathPoint],
        func: &mut T::Function,
        caro: &<T::SurfaceTool as ThePSurfaceTool>::Surface,
        reversed: bool,
    ) -> Result<(), Failure> {
        let mut rajout: bool = false;
        self.reversed = reversed;
        let mut mult = Multiples {
            u: Vec::new(),
            v: Vec::new(),
        };
        self.load_path_points(pnts1, &mut mult, false)?;
        self.load_tolerance(caro)?;
        self.load_bounds(caro)?;
        func.set_surface(caro);
        if !pnts1.is_empty() {
            self.compute_open_line(&mult, pnts1, func, &mut rajout)?;
        }
        self.finish(pnts1);
        Ok(())
    }
    /// The parameter bounds of the surface, ordered.
    fn load_bounds(
        &mut self,
        caro: &<T::SurfaceTool as ThePSurfaceTool>::Surface,
    ) -> Result<(), Failure> {
        self.um = T::SurfaceTool::first_u_parameter(caro)?;
        self.vm = T::SurfaceTool::first_v_parameter(caro)?;
        self.um_last = T::SurfaceTool::last_u_parameter(caro)?;
        self.vm_last = T::SurfaceTool::last_v_parameter(caro)?;
        if self.um_last < self.um {
            std::mem::swap(&mut self.um, &mut self.um_last);
        }
        if self.vm_last < self.vm {
            std::mem::swap(&mut self.vm, &mut self.vm_last);
        }
        Ok(())
    }
    /// The parameter resolutions of `Precision::Confusion()`.
    fn load_tolerance(
        &mut self,
        caro: &<T::SurfaceTool as ThePSurfaceTool>::Surface,
    ) -> Result<(), Failure> {
        self.tolerance[1] = T::SurfaceTool::u_resolution(caro, precision::confusion())?;
        self.tolerance[2] = T::SurfaceTool::v_resolution(caro, precision::confusion())?;
        Ok(())
    }
    /// The walking data of the path points and their multiplicity parameters; the
    /// full Perform treats a passing, not tangent point (state 2) as 11 and widens the
    /// estimated ranges.
    fn load_path_points(
        &mut self,
        pnts1: &[IntSurfPathPoint],
        mult: &mut Multiples,
        full: bool,
    ) -> Result<(), Failure> {
        for point in pnts1 {
            let mut data: IntWalkWalkingData = path_walking_data(point)?;
            if full && data.etat == 2 {
                data.etat = 11;
            }
            if full {
                self.my_s_range_u.add(data.ustart);
                self.my_s_range_v.add(data.vstart);
            }
            self.wd1.push(data);
            let nb_mult: i32 = IntSurfPathPointTool::multiplicity(point)?;
            self.nb_multiplicities.push(nb_mult);
            for j in 1..=nb_mult {
                let (u, v) = IntSurfPathPointTool::parameters(point, j)?;
                mult.u.push(u);
                mult.v.push(v);
            }
        }
        Ok(())
    }
    /// The walking data of the interior points: 13 unless the function vanishes all
    /// around the point (checked before the surface is set on the function, as natively).
    fn load_interior_points(
        &mut self,
        pnts2: &[IntSurfInteriorPoint],
        func: &mut T::Function,
        steps: (f64, f64),
    ) -> Result<(), Failure> {
        let bounds: [f64; 4] = [self.um, self.um_last, self.vm, self.vm_last];
        for point in pnts2 {
            let (ustart, vstart) = IntSurfInteriorPointTool::value2d(point);
            self.my_s_range_u.add(ustart);
            self.my_s_range_v.add(vstart);
            let mut etat: i32 = 1;
            if !is_tangent_ext_check(func, (ustart, vstart), steps, bounds)? {
                etat = 13;
            }
            self.wd2.push(IntWalkWalkingData {
                ustart,
                vstart,
                etat,
            });
        }
        Ok(())
    }
    /// The section ranges: the start points' range enlarged by its width and kept in
    /// the domain, or the whole domain when the points are too close.
    fn estimate_ranges(&mut self) -> Result<(), Failure> {
        let (tol_u, tol_v) = (self.tolerance[1], self.tolerance[2]);
        self.my_s_range_u = estimated_range(self.my_s_range_u, tol_u, (self.um, self.um_last))?;
        self.my_s_range_v = estimated_range(self.my_s_range_v, tol_v, (self.vm, self.vm_last))?;
        Ok(())
    }
    /// The hole filling of the full Perform: at most 10 rounds while more than one
    /// line end is alone, each walking closed lines from the points found in the holes.
    fn fill_holes(
        &mut self,
        mult: &Multiples,
        pnts1: &[IntSurfPathPoint],
        func: &mut T::Function,
        rajout: &mut bool,
    ) -> Result<(), Failure> {
        let (max_nb_iter, mut nb_iter) = (10, 0);
        while self.seq_alone.len() > 1 && nb_iter < max_nb_iter {
            nb_iter += 1;
            let mut pnts_in_holes: Vec<IntSurfInteriorPoint> = Vec::new();
            let mut copy_seq_alone: Vec<i32> = self.seq_alone.clone();
            self.fill_pnts_in_holes(func, &mut copy_seq_alone, &mut pnts_in_holes)?;
            self.wd2.clear();
            self.wd2.push(DUMMY);
            for point in &pnts_in_holes {
                let (ustart, vstart) = IntSurfInteriorPointTool::value2d(point);
                self.wd2.push(IntWalkWalkingData {
                    ustart,
                    vstart,
                    etat: 13,
                });
            }
            self.compute_close_line(mult, pnts1, &pnts_in_holes, func, rajout)?;
        }
        Ok(())
    }
    /// The path points on which no line starts or ends become single points.
    fn finish(&mut self, pnts1: &[IntSurfPathPoint]) {
        for (i, point) in pnts1.iter().enumerate() {
            // The data of point i is at index i + 1 (index 0 is the dummy entry); the
            // native loop reads it unchecked, so a short vector stops the scan here.
            if self.wd1.get(i + 1).is_some_and(|data| data.etat > 0) {
                self.seq_single.push(point.clone());
            }
        }
        self.done = true;
    }
}

/// The estimated section range of one parameter (`mySRangeU` in Perform).
fn estimated_range(
    range: Range1,
    tolerance: f64,
    (first, last): (f64, f64),
) -> Result<Range1, Failure> {
    let mut range: Range1 = range;
    if range.delta() > std_max(tolerance, precision::p_confusion()) {
        range.enlarge(range.delta());
        range.common(Range1::new(first, last)?);
        return Ok(range);
    }
    Range1::new(first, last)
}
