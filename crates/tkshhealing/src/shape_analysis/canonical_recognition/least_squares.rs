// Rust translation of GetSurfaceByLS and the static GetSamplePoints, GetLSGap, FillSolverData
// and SetCanonicParameters of ModelingAlgorithms/TKShHealing/ShapeAnalysis/
// ShapeAnalysis_CanonicalRecognition.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The least-squares fit of a sphere, cylinder or cone through sample points of a wire:
//! a particle swarm around the sample surface, then Powell's method.
use super::surfaces::SurfFit;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{self, ShapeIterator, Wire};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::real_to_int;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::gcpnts::{AbscissaPoint, QuasiUniformAbscissa};
use rustycad_tkgeombase::geom_convert::{
    FuncConeLSDist, FuncCylinderLSDist, FuncSphereLSDist, HArrayOfXyz,
};
use rustycad_tkmath::elslib::{cone_d0, cone_parameters};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Axis3, Direction3, Plane3, Point3, Vector3, Xyz};
use rustycad_tkmath::math::{Matrix, MultipleVarFunction, Powell, Pso, Vector};
use std::f64::consts::FRAC_PI_2;

/// The edge lengths of the sample points: `GCPnts_AbscissaPoint::Length(C, tol)`.
fn curve_length(curve: &BRepAdaptorCurve, tol: f64) -> Result<f64, Failure> {
    let adaptor: &dyn Adaptor3dCurve = curve;
    AbscissaPoint::length_with_tolerance(adaptor, tol)
}

/// The static `GetSamplePoints`: quasi-uniform points on each non-degenerated edge, in
/// number proportional to its length (`max_nb_int` over the wire, at least 2 per edge);
/// `None` for a wire shorter than `tol` or without point.
fn sample_points(wire: &Wire, tol: f64, max_nb_int: i32) -> Result<Option<HArrayOfXyz>, Failure> {
    let length_tol = (tol / 10.0).max(1.0e-3);
    let mut curves = Vec::new();
    let mut total_length = 0.0;
    let mut it = ShapeIterator::new(wire, true, true);
    while it.more() {
        let edge = topods::edge(it.value()?)?;
        it.next();
        if BRepTool::degenerated(&edge)? {
            continue;
        }
        let curve = BRepAdaptorCurve::with_edge(&edge)?;
        total_length += curve_length(&curve, length_tol)?;
        curves.push(curve);
    }
    if total_length < tol {
        return Ok(None);
    }
    let mut points: Vec<Xyz> = Vec::new();
    for curve in &curves {
        let length = curve_length(curve, length_tol)?;
        let nb_points = real_to_int(length / total_length * f64::from(max_nb_int) + 1.0).max(2);
        let adaptor: &dyn Adaptor3dCurve = curve;
        let generator = QuasiUniformAbscissa::with_nb_points(adaptor, nb_points)?;
        if !generator.is_done() {
            continue;
        }
        for j in 1..=generator.nb_points()? {
            points.push(curve.value(generator.parameter(j)?)?.xyz());
        }
    }
    if points.is_empty() {
        return Ok(None);
    }
    let array = HArray1::with_bounds(1, points.len() as i32)
        .map_err(|e| e.into_failure("ShapeAnalysis_CanonicalRecognition::GetSamplePoints"))?;
    for (i, p) in points.into_iter().enumerate() {
        array.change_array1()[i as i32 + 1] = p;
    }
    Ok(Some(Handle::new(array)))
}

/// The static `GetLSGap`: the largest distance of the points to the sphere, cylinder or
/// cone of the frame and parameters (0 for another type).
fn ls_gap(points: &Array1<'_, Xyz>, target: SurfaceType, pos: &Axis3, params: &[f64]) -> f64 {
    let mut gap: f64 = 0.0;
    let loc = pos.location().xyz();
    let dir = Vector3::from(pos.direction());
    match target {
        SurfaceType::Sphere => {
            for p in points.iter() {
                gap = gap.max(((*p - loc).modulus() - params[0]).abs());
            }
        }
        SurfaceType::Cylinder => {
            for p in points.iter() {
                let d = Vector3::from(*p - loc).crossed(dir);
                gap = gap.max((d.magnitude() - params[0]).abs());
            }
        }
        SurfaceType::Cone => {
            let (ang, r) = (params[0], params[1]);
            for p in points.iter() {
                let pi = Point3::from(*p);
                let (u, v) = cone_parameters(*pos, r, ang, pi);
                gap = gap.max(pi.square_distance(&cone_d0(u, v, *pos, r, ang)));
            }
            gap = gap.sqrt();
        }
        _ => {}
    }
    gap
}

/// The optimization variables and their bounds (`FillSolverData`).
struct SolverData {
    start: Vector<'static>,
    lower: Vector<'static>,
    upper: Vector<'static>,
}

/// The static `FillSolverData`: the start point (location and parameters) and bounds
/// within `rel_dev` of the sample radius (and of the cone angle).
fn fill_solver_data(
    target: SurfaceType,
    pos: &Axis3,
    params: &[f64],
    nb_var: i32,
    rel_dev: f64,
) -> Result<SolverData, Failure> {
    let mut data = SolverData {
        start: Vector::with_bounds(1, nb_var)?,
        lower: Vector::with_bounds(1, nb_var)?,
        upper: Vector::with_bounds(1, nb_var)?,
    };
    let loc = pos.location();
    (data.start[1], data.start[2], data.start[3]) = (loc.x(), loc.y(), loc.z());
    data.start[4] = params[0];
    let dr = if target == SurfaceType::Cone {
        data.start[5] = params[1];
        let dr = rel_dev * params[1];
        if dr < precision::confusion() { 0.1 } else { dr }
    } else {
        rel_dev * params[0]
    };
    for i in 1..=3 {
        data.lower[i] = data.start[i] - dr;
        data.upper[i] = data.start[i] + dr;
    }
    if target == SurfaceType::Cone {
        fill_cone_angle(&mut data, params, dr, rel_dev);
        data.lower[5] = data.start[5] - dr;
        data.upper[5] = data.start[5] + dr;
    } else {
        data.lower[4] = data.start[4] - dr;
        data.upper[4] = data.start[4] + dr;
    }
    Ok(data)
}

/// The semi-angle bounds of a cone. Natively the upper bound of a positive angle adds
/// the radius deviation `dr`, not the angle one.
fn fill_cone_angle(data: &mut SolverData, params: &[f64], dr: f64, rel_dev: f64) {
    let d_ang = rel_dev * params[0].abs();
    if params[0] >= 0.0 {
        data.lower[4] = data.start[4] - d_ang;
        data.upper[4] = FRAC_PI_2.min(data.start[4] + dr);
    } else {
        data.lower[4] = (-FRAC_PI_2).max(data.start[4] - d_ang);
        data.upper[4] = data.start[4] + d_ang;
    }
}

/// The static `SetCanonicParameters`: the solution as a location and parameters.
fn set_canonic_parameters(
    target: SurfaceType,
    sol: &Vector<'_>,
    pos: &mut Axis3,
    params: &mut [f64],
) {
    pos.set_location(Point3::new(sol[1], sol[2], sol[3]));
    if matches!(target, SurfaceType::Sphere | SurfaceType::Cylinder) {
        params[0] = sol[4];
    } else if target == SurfaceType::Cone {
        params[0] = sol[4];
        params[1] = sol[5];
    }
}

/// The Powell directions: the unit matrix, with the location searched in the plane
/// normal to the axis of a cylinder or a cone ("to avoid searching along axis").
fn powell_directions(
    target: SurfaceType,
    pos: &Axis3,
    nb_var: i32,
) -> Result<Matrix<'static>, Failure> {
    let mut directions = Matrix::with_bounds(1, nb_var, 1, nb_var)
        .map_err(|e| e.into_failure("ShapeAnalysis_CanonicalRecognition::GetSurfaceByLS"))?;
    directions.init(0.0);
    for i in 1..=nb_var {
        directions[(i, i)] = 1.0;
    }
    if !matches!(target, SurfaceType::Cylinder | SurfaceType::Cone) {
        return Ok(directions);
    }
    let plane = Plane3::new(pos.location(), pos.direction())?;
    let u_dir = plane.position().x_direction();
    let v_dir = plane.position().y_direction();
    for i in 1..=3 {
        directions[(i, 1)] = u_dir.coord(i)?;
        directions[(i, 2)] = v_dir.coord(i)?;
        let uv_dir = Direction3::from_xyz(u_dir.xyz() + v_dir.xyz())?;
        directions[(i, 3)] = uv_dir.coord(i)?;
    }
    Ok(directions)
}

/// The least-squares functions, whose failures inside the solvers are given back after
/// them (natively they propagate as exceptions).
trait LsFunction: MultipleVarFunction {
    fn take_failure(&mut self) -> Result<(), Failure>;
}

macro_rules! ls_function {
    ($($function:ty),*) => {$(
        impl LsFunction for $function {
            fn take_failure(&mut self) -> Result<(), Failure> {
                <$function>::take_failure(self)
            }
        }
    )*};
}
ls_function!(FuncSphereLSDist, FuncCylinderLSDist, FuncConeLSDist);

/// `GetSurfaceByLS(theWire, theTol, theTarget, thePos, theParams, theGap, theStatus)`:
/// the sphere, cylinder or cone through sample points of the wire within `tol`, seeded by
/// the given frame and parameters.
pub(super) fn surface_by_ls(
    wire: &Wire,
    tol: f64,
    target: SurfaceType,
    fit: &mut SurfFit<'_>,
) -> Result<bool, Failure> {
    let Some(points) = sample_points(wire, tol, 100)? else {
        return Ok(false);
    };
    let cloud: Array1<'static, Xyz> = match points.get() {
        Some(array) => array.array1().clone(),
        None => return Ok(false),
    };
    *fit.gap = ls_gap(&cloud, target, fit.pos, fit.params);
    if *fit.gap <= tol {
        *fit.status = 0;
        return Ok(true);
    }
    let nb_var = match target {
        SurfaceType::Sphere | SurfaceType::Cylinder => 4,
        SurfaceType::Cone => 5,
        _ => return Ok(false),
    };
    // Relative precision of the sample surface parameters: the "exact" radius R is
    // searched in [R - 0.2 R, R + 0.2 R].
    let mut data = fill_solver_data(target, fit.pos, fit.params, nb_var, 0.2)?;
    let axis_dir = fit.pos.direction();
    let problem = LsProblem {
        target,
        tol,
        cloud: &cloud,
    };
    match target {
        SurfaceType::Sphere => problem.solve(&mut FuncSphereLSDist::new(&points), &mut data, fit),
        SurfaceType::Cylinder => problem.solve(
            &mut FuncCylinderLSDist::new(&points, axis_dir),
            &mut data,
            fit,
        ),
        _ => problem.solve(&mut FuncConeLSDist::new(&points, axis_dir), &mut data, fit),
    }
}

/// The target, tolerance and points of one least-squares fit.
struct LsProblem<'a> {
    target: SurfaceType,
    tol: f64,
    cloud: &'a Array1<'static, Xyz>,
}

impl LsProblem<'_> {
    /// The solution written as the frame and parameters, with its gap.
    fn accept(&self, sol: &Vector<'_>, fit: &mut SurfFit<'_>) -> f64 {
        set_canonic_parameters(self.target, sol, fit.pos, fit.params);
        *fit.gap = ls_gap(self.cloud, self.target, fit.pos, fit.params);
        *fit.gap
    }

    /// `math_PSO` within the bounds (10 steps per variable), then, when its gap is too
    /// large, `math_Powell` from its solution.
    fn solve<F: LsFunction>(
        &self,
        func: &mut F,
        data: &mut SolverData,
        fit: &mut SurfFit<'_>,
    ) -> Result<bool, Failure> {
        let nb_var = data.start.length();
        let mut steps = Vector::with_bounds(1, nb_var)?;
        for i in 1..=nb_var {
            steps[i] = (data.upper[i] - data.lower[i]) / 10.0;
        }
        let mut ls_dist = 0.0;
        {
            let mut global = Pso::new(func, &data.lower, &data.upper, &steps)?;
            global.perform(&steps, &mut ls_dist, &mut data.start)?;
        }
        func.take_failure()?;
        if self.accept(&data.start, fit) <= self.tol {
            *fit.status = 0;
            return Ok(true);
        }
        let directions = powell_directions(self.target, fit.pos, nb_var)?;
        let mut solver = Powell::new(func, precision::confusion())?;
        solver.perform(func, &data.start, &directions)?;
        func.take_failure()?;
        if !solver.is_done() {
            *fit.status = 1;
            return Ok(false);
        }
        solver.copy_location(&mut data.start)?;
        *fit.status = 0;
        Ok(self.accept(&data.start, fit) <= self.tol)
    }
}
