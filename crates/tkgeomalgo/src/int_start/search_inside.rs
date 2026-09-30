// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntStart/IntStart_SearchInside.gxx
// and IntStart_SearchInside.lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntStart_SearchInside`: start points of the walking inside the domain of a
//! parametric surface, where the implicit function vanishes (docs/geomalgo-design.md
//! section 12).
use super::failures::{not_done, sequence_value};
use super::tools::{SearchInsideTypes, TheInsideFunction, TheTopolTool};
use crate::int_imp::ThePSurfaceTool;
use crate::int_surf::IntSurfInteriorPoint;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::gp::{Point2, Point3, Vector2};
use rustycad_tkmath::math::{FunctionSetRoot, Vector};
use std::marker::PhantomData;

/// `IntStart_SearchInside`: the interior start points found by the last Perform.
///
/// ```ignore
/// let mut search = SearchInside::<IntPatchTypes>::new();
/// search.perform(&mut function, &surface, &domain, 1.0e-6)?;
/// ```
pub struct SearchInside<T: SearchInsideTypes> {
    done: bool,
    list: Vec<IntSurfInteriorPoint>,
    types: PhantomData<T>,
}

impl<T: SearchInsideTypes> Default for SearchInside<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// The parameter box of the surface and the sampling steps of the first Perform.
struct InsideFrame {
    umin: f64,
    vmin: f64,
    umax: f64,
    vmax: f64,
    du: f64,
    dv: f64,
}

/// The solver inputs of one start: the start point and its bounds.
struct InsideStart {
    uvap: [f64; 2],
    binf: [f64; 2],
    bsup: [f64; 2],
}

type SurfaceOf<T> = <<T as SearchInsideTypes>::SurfaceTool as ThePSurfaceTool>::Surface;

impl<T: SearchInsideTypes> SearchInside<T> {
    /// The empty constructor: not done. `SearchInside::<T>::new().is_done() == false`.
    pub fn new() -> Self {
        Self {
            done: false,
            list: Vec::new(),
            types: PhantomData,
        }
    }

    /// The constructor that performs the search (`IntStart_SearchInside(Func, PS, T,
    /// Epsilon)`). `SearchInside::<T>::with_domain(&mut f, &s, &domain, 1e-6)?`.
    pub fn with_domain(
        func: &mut T::Function,
        ps: &SurfaceOf<T>,
        domain: &T::TopolTool,
        epsilon: f64,
    ) -> Result<Self, Failure> {
        let mut search: Self = Self::new();
        search.perform(func, ps, domain, epsilon)?;
        Ok(search)
    }

    /// `IsDone()`. `search.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `NbPoints()`: StdFail_NotDone before a Perform. `search.nb_points()?`.
    pub fn nb_points(&self) -> Result<i32, Failure> {
        self.check_done("IntStart_SearchInside::NbPoints")?;
        Ok(self.list.len() as i32)
    }

    /// `Value(Index)`: StdFail_NotDone before a Perform, Standard_OutOfRange outside
    /// `1..=NbPoints()`. `search.value(1)?`.
    pub fn value(&self, index: i32) -> Result<&IntSurfInteriorPoint, Failure> {
        self.check_done("IntStart_SearchInside::Value")?;
        sequence_value(&self.list, index, "IntStart_SearchInside::Value")
    }

    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if self.done {
            return Ok(());
        }
        Err(not_done(member))
    }

    /// `Perform(Func, PS, T, Epsilon)`: samples the domain, solves from every sample and
    /// from twelve points near the corners, and keeps the new solutions inside the
    /// domain. `search.perform(&mut f, &s, &domain, 1e-6)?`.
    pub fn perform(
        &mut self,
        func: &mut T::Function,
        ps: &SurfaceOf<T>,
        domain: &T::TopolTool,
        epsilon: f64,
    ) -> Result<(), Failure> {
        self.done = false;
        self.list.clear();
        let (mut frame, nbsample) = Self::frame(ps, domain)?;
        let toler: [f64; 2] = Self::resolutions(ps)?;
        let max_toler: f64 = Self::classify_tolerance(&frame, toler);
        func.set_surface(ps);
        let tol: f64 = func.tolerance();
        let mut rsnld = FunctionSetRoot::new(&*func, &vector2(toler)?, 100)?;
        frame.umin += frame.du * 0.01;
        frame.vmin += frame.dv * 0.01;
        frame.umax -= frame.du * 0.01;
        frame.vmax -= frame.dv * 0.01;
        for i in 1..=nbsample + 12 {
            let Some(start) = Self::start(func, ps, domain, &frame, i, nbsample)? else {
                continue;
            };
            let solution = Self::solve(func, &mut rsnld, &start, tol)?;
            if let Some((psol, uvap)) = solution {
                self.keep((func, domain), psol, uvap, (epsilon, toler, max_toler))?;
            }
        }
        self.done = true;
        Ok(())
    }

    /// The parameter box, the sampling steps (half-cell sizes of the domain sampling) and
    /// the number of samples of the domain.
    fn frame(ps: &SurfaceOf<T>, domain: &T::TopolTool) -> Result<(InsideFrame, i32), Failure> {
        let umin: f64 = T::SurfaceTool::first_u_parameter(ps)?;
        let vmin: f64 = T::SurfaceTool::first_v_parameter(ps)?;
        let umax: f64 = T::SurfaceTool::last_u_parameter(ps)?;
        let vmax: f64 = T::SurfaceTool::last_v_parameter(ps)?;
        let nbsample_u: i32 = domain.nb_samples_u()?;
        let nbsample_v: i32 = domain.nb_samples_v()?;
        let nbsample: i32 = domain.nb_samples()?;
        let mut du: f64 = umax - umin;
        let mut dv: f64 = vmax - vmin;
        du /= f64::from(nbsample_u) * 0.5;
        dv /= f64::from(nbsample_v) * 0.5;
        let frame = InsideFrame {
            umin,
            vmin,
            umax,
            vmax,
            du,
            dv,
        };
        Ok((frame, nbsample))
    }

    fn resolutions(ps: &SurfaceOf<T>) -> Result<[f64; 2], Failure> {
        let toler1: f64 = T::SurfaceTool::u_resolution(ps, precision::confusion())?;
        let toler2: f64 = T::SurfaceTool::v_resolution(ps, precision::confusion())?;
        Ok([toler1, toler2])
    }

    /// The classification tolerance: the points must not come too near the
    /// restrictions (lbr, 15 May 1997).
    fn classify_tolerance(frame: &InsideFrame, toler: [f64; 2]) -> f64 {
        let mut max_toler: f64 = toler[0];
        if toler[1] > max_toler {
            max_toler = toler[1];
        }
        max_toler *= 1000.0;
        if max_toler > frame.du * 0.001 {
            max_toler = frame.du * 0.001;
        }
        if max_toler > frame.dv * 0.001 {
            max_toler = frame.dv * 0.001;
        }
        max_toler
    }

    /// The start of iteration `i`: a domain sample (skipped when the function is far
    /// from zero there) or, past the samples, one of the twelve points near the corners
    /// (lbr, 30 October 1997: a surface tangent to two edges near a corner).
    fn start(
        func: &mut T::Function,
        ps: &SurfaceOf<T>,
        domain: &T::TopolTool,
        frame: &InsideFrame,
        i: i32,
        nbsample: i32,
    ) -> Result<Option<InsideStart>, Failure> {
        if i > nbsample {
            return Ok(Some(bounded_start(
                frame,
                corner_point(frame, i - nbsample),
            )));
        }
        let (s2d, _) = domain.sample_point(i)?;
        let start: InsideStart = bounded_start(frame, s2d);
        let pextrm1: Point3 = T::SurfaceTool::value(ps, start.binf[0], start.binf[1])?;
        let pextrm2: Point3 = T::SurfaceTool::value(ps, start.bsup[0], start.bsup[1])?;
        let mut valf: Vector<'static> = Vector::with_bounds(1, 1)?;
        func.checked_value(&vector2(start.uvap)?, &mut valf)?;
        let rvalf: f64 = valf[1];
        let dist_pp: f64 = pextrm1.square_distance(&pextrm2);
        // `nepastester`: the native test is `rvalf * rvalf > 3 * DistPP`, false on NaN.
        if rvalf * rvalf > 3.0 * dist_pp {
            return Ok(None);
        }
        Ok(Some(start))
    }

    /// Solves from `start`; the solution point and parameters when the root is null and
    /// not tangent.
    fn solve(
        func: &mut T::Function,
        rsnld: &mut FunctionSetRoot,
        start: &InsideStart,
        tol: f64,
    ) -> Result<Option<(Point3, [f64; 2])>, Failure> {
        let solved = rsnld.perform_bounded(
            func,
            &vector2(start.uvap)?,
            &vector2(start.binf)?,
            &vector2(start.bsup)?,
            false,
        );
        func.take_failure()?;
        solved?;
        // The native tests are `IsDone()`, then `abs(Root()) <= Tol`, then
        // `!IsTangent()`; a NaN root fails the second one.
        let null_root: bool = rsnld.is_done() && func.root().abs() <= tol;
        if !null_root || func.is_tangent()? {
            return Ok(None);
        }
        let psol: Point3 = func.point();
        let root: &Vector<'static> = rsnld.root()?;
        Ok(Some((psol, [root[1], root[2]])))
    }

    /// Appends the solution when it is a new point inside the domain.
    fn keep(
        &mut self,
        (func, domain): (&mut T::Function, &T::TopolTool),
        psol: Point3,
        uvap: [f64; 2],
        (epsilon, toler, max_toler): (f64, [f64; 2], f64),
    ) -> Result<(), Failure> {
        let known: bool = self.list.iter().any(|ipj| {
            let pj: Point3 = ipj.value();
            (pj.x() - psol.x()).abs() <= epsilon
                && (pj.y() - psol.y()).abs() <= epsilon
                && (pj.z() - psol.z()).abs() <= epsilon
                && (uvap[0] - ipj.u_parameter()).abs() <= toler[0]
                && (uvap[1] - ipj.v_parameter()).abs() <= toler[1]
        });
        if known {
            return Ok(());
        }
        // `false`: the point is not reframed on a periodic domain.
        let situ: State = domain.classify(Point2::new(uvap[0], uvap[1]), max_toler, false)?;
        if situ == State::In {
            self.list.push(interior_point(func, psol, uvap)?);
        }
        Ok(())
    }

    /// `Perform(Func, PS, UStart, VStart)`: one solve from `(UStart, VStart)` when it is
    /// inside the parameter box. `search.perform_from(&mut f, &s, 0.5, 0.5)?`.
    pub fn perform_from(
        &mut self,
        func: &mut T::Function,
        ps: &SurfaceOf<T>,
        u_start: f64,
        v_start: f64,
    ) -> Result<(), Failure> {
        self.done = false;
        self.list.clear();
        let binf: [f64; 2] = [
            T::SurfaceTool::first_u_parameter(ps)?,
            T::SurfaceTool::first_v_parameter(ps)?,
        ];
        let bsup: [f64; 2] = [
            T::SurfaceTool::last_u_parameter(ps)?,
            T::SurfaceTool::last_v_parameter(ps)?,
        ];
        let toler: [f64; 2] = Self::resolutions(ps)?;
        if u_start - binf[0] > -toler[0]
            && u_start - bsup[0] < toler[0]
            && v_start - binf[1] > -toler[1]
            && v_start - bsup[1] < toler[1]
        {
            func.set_surface(ps);
            let start = InsideStart {
                uvap: [u_start, v_start],
                binf,
                bsup,
            };
            let mut rsnld = FunctionSetRoot::new(&*func, &vector2(toler)?, 100)?;
            if let Some((psol, uvap)) = Self::solve(func, &mut rsnld, &start, func.tolerance())? {
                self.list.push(interior_point(func, psol, uvap)?);
            }
        }
        self.done = true;
        Ok(())
    }
}

/// `IntSurf_InteriorPoint(psol, U, V, Func.Direction3d(), Func.Direction2d())`.
fn interior_point<F: TheInsideFunction>(
    func: &mut F,
    psol: Point3,
    uvap: [f64; 2],
) -> Result<IntSurfInteriorPoint, Failure> {
    let direction3d = func.direction3d()?;
    let direction2d = func.direction2d()?;
    Ok(IntSurfInteriorPoint::with_value(
        psol,
        uvap[0],
        uvap[1],
        direction3d,
        Vector2::from(direction2d),
    ))
}

/// The start point and its solver box, clipped to the frame.
fn bounded_start(frame: &InsideFrame, s2d: Point2) -> InsideStart {
    let uvap: [f64; 2] = [s2d.x(), s2d.y()];
    InsideStart {
        uvap,
        binf: [
            std_max(frame.umin, uvap[0] - frame.du),
            std_max(frame.vmin, uvap[1] - frame.dv),
        ],
        bsup: [
            std_min(frame.umax, uvap[0] + frame.du),
            std_min(frame.vmax, uvap[1] + frame.dv),
        ],
    }
}

/// The `k`-th (1..=12) point near the corners: twice the four corners at 2% of the
/// steps, then once at 0.5%.
fn corner_point(frame: &InsideFrame, k: i32) -> Point2 {
    let ratio: f64 = if k <= 8 { 0.02 } else { 0.005 };
    let corner: i32 = (k - 1) % 4;
    let u: f64 = if corner % 2 == 0 {
        frame.umin + frame.du * ratio
    } else {
        frame.umax - frame.du * ratio
    };
    let v: f64 = if corner < 2 {
        frame.vmin + frame.dv * ratio
    } else {
        frame.vmax - frame.dv * ratio
    };
    Point2::new(u, v)
}

fn vector2(values: [f64; 2]) -> Result<Vector<'static>, Failure> {
    let mut vector: Vector<'static> = Vector::with_bounds(1, 2)?;
    (vector[1], vector[2]) = (values[0], values[1]);
    Ok(vector)
}
