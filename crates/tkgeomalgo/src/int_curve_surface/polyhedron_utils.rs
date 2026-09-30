// Rust translation of IntCurveSurface_PolyhedronUtils.pxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntCurveSurface_PolyhedronUtils`: the grid triangulation of a surface patch.
//! The native functions read `void*` arrays of points, parameters and bound flags
//! indexed from 1 (index 0 is allocated but never written); here they read
//! [`PolyhedronGrid`], whose vectors keep that layout.
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_first;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom_grid_eval::GeomGridEvalSurface;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{self, Point3, Xyz};

/// `THE_MIN_EDGE_LENGTH_SQUARED`: the squared length below which an edge is null.
pub const THE_MIN_EDGE_LENGTH_SQUARED: f64 = 1e-15;

/// The sampled grid of a polyhedron: `pnts[i]`, `u[i]`, `v[i]` and `is_on_bounds[i]`
/// for the point index `i` from 1 (`C_MyPnts`, `C_MyU`, `C_MyV`, `C_MyIsOnBounds`).
#[derive(Clone, Debug, Default)]
pub struct PolyhedronGrid {
    /// The points; index 0 holds the origin, as `new gp_Pnt[t]` does.
    pub pnts: Vec<Point3>,
    /// The u parameters; index 0 is 0 (uninitialized natively).
    pub u: Vec<f64>,
    /// The v parameters; index 0 is 0 (uninitialized natively).
    pub v: Vec<f64>,
    /// Whether each point lies on the patch boundary; index 0 is false.
    pub is_on_bounds: Vec<bool>,
}

/// `AllocateArrays`: `(nbdu + 1) * (nbdv + 1) + 1` entries of each array;
/// `allocate_arrays(3, 3)`.
pub fn allocate_arrays(nb_delta_u: i32, nb_delta_v: i32) -> PolyhedronGrid {
    let t: usize = ((nb_delta_u + 1) * (nb_delta_v + 1) + 1).max(0) as usize;
    PolyhedronGrid {
        pnts: vec![Point3::default(); t],
        u: vec![0.0; t],
        v: vec![0.0; t],
        is_on_bounds: vec![false; t],
    }
}

/// `Destroy`: releases the arrays; `destroy(&mut grid)`.
pub fn destroy(grid: &mut PolyhedronGrid) {
    *grid = PolyhedronGrid::default();
}

/// The `n + 1` parameters `first + i * step` of a uniform subdivision, from index 1.
fn uniform_params(first: f64, step: f64, n: i32) -> Result<Array1<'static, f64>, Failure> {
    let mut params: Array1<'static, f64> =
        Array1::with_bounds(1, n + 1).map_err(|e| e.into_failure("NCollection_Array1"))?;
    for i in 0..=n {
        params[i + 1] = first + f64::from(i) * step;
    }
    Ok(params)
}

/// Copies the evaluated `grid` into the arrays with its parameters (read from their
/// lower bounds) and flags, adding each point to `bnd`.
fn store_grid(
    grid: &Array2<'_, Point3>,
    (upars, vpars): (&Array1<'_, f64>, &Array1<'_, f64>),
    (nb_delta_u, nb_delta_v): (i32, i32),
    target: &mut PolyhedronGrid,
    bnd: &mut BoundingBox3,
) {
    let (i0, j0) = (upars.lower(), vpars.lower());
    let mut index: usize = 1;
    for i1 in 0..=nb_delta_u {
        for i2 in 0..=nb_delta_v {
            let tp: Point3 = grid[(i1 + 1, i2 + 1)];
            target.pnts[index] = tp;
            target.u[index] = upars[i1 + i0];
            target.v[index] = vpars[i2 + j0];
            target.is_on_bounds[index] = i1 == 0 || i1 == nb_delta_u || i2 == 0 || i2 == nb_delta_v;
            bnd.add_point(tp);
            index += 1;
        }
    }
}

/// `InitUniform`: the grid of `(nbdu + 1) x (nbdv + 1)` points at `u0 + i * dU`,
/// `v0 + j * dV`; `init_uniform(&eval, [u0, v0, u1, v1], (3, 3), &mut grid, &mut bnd)?`.
pub fn init_uniform(
    eval: &GeomGridEvalSurface<'_>,
    [u0, v0, u1, v1]: [f64; 4],
    (nb_delta_u, nb_delta_v): (i32, i32),
    target: &mut PolyhedronGrid,
    bnd: &mut BoundingBox3,
) -> Result<(), Failure> {
    let du: f64 = (u1 - u0) / f64::from(nb_delta_u);
    let dv: f64 = (v1 - v0) / f64::from(nb_delta_v);
    let (upars, vpars) = (
        uniform_params(u0, du, nb_delta_u)?,
        uniform_params(v0, dv, nb_delta_v)?,
    );
    let grid = eval.evaluate_grid(&upars, &vpars)?;
    store_grid(
        &grid,
        (&upars, &vpars),
        (nb_delta_u, nb_delta_v),
        target,
        bnd,
    );
    Ok(())
}

/// `InitWithParams`: the grid at the given parameters;
/// `init_with_params(&eval, (&upars, &vpars), (nbdu, nbdv), &mut grid, &mut bnd)?`.
pub fn init_with_params(
    eval: &GeomGridEvalSurface<'_>,
    (upars, vpars): (&Array1<'_, f64>, &Array1<'_, f64>),
    (nb_delta_u, nb_delta_v): (i32, i32),
    target: &mut PolyhedronGrid,
    bnd: &mut BoundingBox3,
) -> Result<(), Failure> {
    let grid = eval.evaluate_grid(upars, vpars)?;
    store_grid(&grid, (upars, vpars), (nb_delta_u, nb_delta_v), target, bnd);
    Ok(())
}

/// `ComputeBorderDeflection`: along the isoline at `parameter` (a u-isoline when
/// `is_u_iso`) cut into `nb_samples` segments of `[p_min, p_max]`, the largest
/// distance of a mid-parameter point to its chord middle (RealFirst when none is
/// larger, 0 for no segment); `compute_border_deflection(&eval, 0.0, 0.0, 1.0, true, 3)?`.
pub fn compute_border_deflection(
    eval: &GeomGridEvalSurface<'_>,
    parameter: f64,
    [p_min, p_max]: [f64; 2],
    is_u_iso: bool,
    nb_samples: i32,
) -> Result<f64, Failure> {
    if nb_samples <= 0 {
        return Ok(0.0);
    }
    let delta: f64 = (p_max - p_min) / f64::from(nb_samples);
    let varying: Array1<'static, f64> = border_params(p_min, delta, nb_samples)?;
    let mut fixed: Array1<'static, f64> =
        Array1::with_bounds(1, 1).map_err(|e| e.into_failure("NCollection_Array1"))?;
    fixed[1] = parameter;
    let grid = if is_u_iso {
        eval.evaluate_grid(&fixed, &varying)?
    } else {
        eval.evaluate_grid(&varying, &fixed)?
    };
    let at = |k: i32| {
        if is_u_iso {
            grid[(1, k)].xyz()
        } else {
            grid[(k, 1)].xyz()
        }
    };
    let mut deflection: f64 = real_first();
    for i in 0..nb_samples {
        let (p1, p2, par_mid) = (at(2 * i + 1), at(2 * i + 3), at(2 * i + 2));
        let mid: Xyz = (p2 + p1) / 2.0;
        let dist: f64 = (mid - par_mid).modulus();
        if dist > deflection {
            deflection = dist;
        }
    }
    Ok(deflection)
}

/// The `2 n + 1` interleaved boundary and middle parameters of a border.
fn border_params(p_min: f64, delta: f64, n: i32) -> Result<Array1<'static, f64>, Failure> {
    let mut params: Array1<'static, f64> =
        Array1::with_bounds(1, 2 * n + 1).map_err(|e| e.into_failure("NCollection_Array1"))?;
    for i in 0..=n {
        params[2 * i + 1] = p_min + f64::from(i) * delta;
        if i < n {
            params[2 * i + 2] = p_min + (f64::from(i) + 0.5) * delta;
        }
    }
    Ok(params)
}

/// `ComputeMaxBorderDeflection`: the largest border deflection of the four sides
/// (u = u0, u = u1, v = v0, v = v1), from RealFirst;
/// `compute_max_border_deflection(&eval, [u0, v0, u1, v1], (3, 3))?`.
pub fn compute_max_border_deflection(
    eval: &GeomGridEvalSurface<'_>,
    [u0, v0, u1, v1]: [f64; 4],
    (nb_delta_u, nb_delta_v): (i32, i32),
) -> Result<f64, Failure> {
    let sides: [(f64, [f64; 2], bool, i32); 4] = [
        (u0, [v0, v1], true, nb_delta_v),
        (u1, [v0, v1], true, nb_delta_v),
        (v0, [u0, u1], false, nb_delta_u),
        (v1, [u0, u1], false, nb_delta_u),
    ];
    let mut max_deflection: f64 = real_first();
    for (parameter, range, is_u_iso, nb_samples) in sides {
        let deflection: f64 =
            compute_border_deflection(eval, parameter, range, is_u_iso, nb_samples)?;
        if deflection > max_deflection {
            max_deflection = deflection;
        }
    }
    Ok(max_deflection)
}

/// `NbTriangles`: `2 * nbdu * nbdv`; `nb_triangles(3, 3) == 18`.
pub fn nb_triangles(nb_delta_u: i32, nb_delta_v: i32) -> i32 {
    nb_delta_u * nb_delta_v * 2
}

/// `NbPoints`: `(nbdu + 1) * (nbdv + 1)`; `nb_points(3, 3) == 16`.
pub fn nb_points(nb_delta_u: i32, nb_delta_v: i32) -> i32 {
    (nb_delta_u + 1) * (nb_delta_v + 1)
}

/// `Triangle`: the three point indices of triangle `index`, by arithmetic only (no
/// range check natively); `triangle(1, 3) == [1, 5, 2]`.
pub fn triangle(index: i32, nb_delta_v: i32) -> [i32; 3] {
    let line: i32 = 1 + ((index - 1) / (nb_delta_v * 2));
    let colon: i32 = 1 + ((index - 1) % (nb_delta_v * 2));
    let colpnt: i32 = (colon + 1) / 2;
    [
        (line - 1) * (nb_delta_v + 1) + colpnt,
        line * (nb_delta_v + 1) + colpnt + ((colon - 1) % 2),
        (line - 1 + (colon % 2)) * (nb_delta_v + 1) + colpnt + 1,
    ]
}

/// `PlaneEquation`: the unit normal and polar distance of a triangle; a null edge
/// gives `(1, 0, 0)` and 0; a normal below gp::Resolution() keeps the raw normal
/// with the distance 0; `plane_equation(p1, p2, p3)`.
pub fn plane_equation(p1: Point3, p2: Point3, p3: Point3) -> (Xyz, f64) {
    let v1: Xyz = p2.xyz() - p1.xyz();
    let v2: Xyz = p3.xyz() - p2.xyz();
    let v3: Xyz = p1.xyz() - p3.xyz();
    let null_edge = |v: Xyz| v.square_modulus() <= THE_MIN_EDGE_LENGTH_SQUARED;
    if null_edge(v1) || null_edge(v2) || null_edge(v3) {
        return (Xyz::new(1.0, 0.0, 0.0), 0.0);
    }
    let mut normal: Xyz = (v1 ^ v2) + (v2 ^ v3) + (v3 ^ v1);
    let norm_len: f64 = normal.modulus();
    if norm_len < gp::resolution() {
        return (normal, 0.0);
    }
    normal /= norm_len;
    (normal, normal * p1.xyz())
}

/// `Contain`: whether `test` lies inside the triangle (the three cross products
/// agree in direction); `contain(p1, p2, p3, test)`.
pub fn contain(p1: Point3, p2: Point3, p3: Point3, test: Point3) -> bool {
    let v1: Xyz = (p2.xyz() - p1.xyz()) ^ (test.xyz() - p1.xyz());
    let v2: Xyz = (p3.xyz() - p2.xyz()) ^ (test.xyz() - p2.xyz());
    let v3: Xyz = (p1.xyz() - p3.xyz()) ^ (test.xyz() - p3.xyz());
    v1 * v2 >= 0.0 && v2 * v3 >= 0.0 && v3 * v1 >= 0.0
}

/// Whether the three edges of a triangle are longer than the null length.
fn has_edges(p1: Point3, p2: Point3, p3: Point3) -> bool {
    p1.square_distance(&p2) > THE_MIN_EDGE_LENGTH_SQUARED
        && p1.square_distance(&p3) > THE_MIN_EDGE_LENGTH_SQUARED
        && p2.square_distance(&p3) > THE_MIN_EDGE_LENGTH_SQUARED
}

/// `FillBounding`: the box of each triangle enlarged by `deflection` (a void box
/// with that gap for a triangle with a null edge), indexed like the triangles;
/// `fill_bounding(&grid.pnts, (3, 3), 1e-4)?`.
pub fn fill_bounding(
    pnts: &[Point3],
    (nb_delta_u, nb_delta_v): (i32, i32),
    deflection: f64,
) -> Result<Array1<'static, BoundingBox3>, Failure> {
    let count: i32 = nb_triangles(nb_delta_u, nb_delta_v);
    let mut boxes: Array1<'static, BoundingBox3> =
        Array1::with_bounds(1, count).map_err(|e| e.into_failure("NCollection_HArray1"))?;
    for i_tri in 1..=count {
        let [np1, np2, np3] = triangle(i_tri, nb_delta_v);
        let (p1, p2, p3) = (pnts[np1 as usize], pnts[np2 as usize], pnts[np3 as usize]);
        let mut boite = BoundingBox3::default();
        if has_edges(p1, p2, p3) {
            boite.add_point(p1);
            boite.add_point(p2);
            boite.add_point(p3);
            boite.enlarge(deflection);
        }
        boite.enlarge(deflection);
        boxes[i_tri] = boite;
    }
    Ok(boxes)
}

/// `ComputeDeflectionWithCenter`: the distance of `center` to the triangle plane, 0
/// for a null edge or normal; `compute_deflection_with_center(p1, p2, p3, c)`.
pub fn compute_deflection_with_center(p1: Point3, p2: Point3, p3: Point3, center: Point3) -> f64 {
    if !has_edges(p1, p2, p3) {
        return 0.0;
    }
    let xyz1: Xyz = p2.xyz() - p1.xyz();
    let xyz2: Xyz = p3.xyz() - p2.xyz();
    let xyz3: Xyz = p1.xyz() - p3.xyz();
    let mut normal: Xyz = (xyz1 ^ xyz2) + (xyz2 ^ xyz3) + (xyz3 ^ xyz1);
    let norm_len: f64 = normal.modulus();
    if norm_len < gp::resolution() {
        return 0.0;
    }
    normal /= norm_len;
    ((center.xyz() - p1.xyz()).dot(normal)).abs()
}

/// `ComputeMaxDeflection`: the largest distance of the surface point at a
/// triangle's parameter centroid to the triangle plane, over the first
/// `nb_triangles` triangles of `grid`; `compute_max_deflection(&*surface, &grid, 3, 18)?`.
pub fn compute_max_deflection(
    surface: &dyn Adaptor3dSurface,
    grid: &PolyhedronGrid,
    nb_delta_v: i32,
    nb_triangles: i32,
) -> Result<f64, Failure> {
    let mut tol: f64 = 0.0;
    for i in 1..=nb_triangles {
        let [i1, i2, i3] = triangle(i, nb_delta_v).map(|k| k as usize);
        let u_center: f64 = (grid.u[i1] + grid.u[i2] + grid.u[i3]) / 3.0;
        let v_center: f64 = (grid.v[i1] + grid.v[i2] + grid.v[i3]) / 3.0;
        let center: Point3 = surface.d0(u_center, v_center)?;
        let tol1: f64 =
            compute_deflection_with_center(grid.pnts[i1], grid.pnts[i2], grid.pnts[i3], center);
        if tol1 > tol {
            tol = tol1;
        }
    }
    Ok(tol)
}

/// `IsOnBound`: whether the edge `index1 index2` joins two boundary points of the
/// same grid line or column; `is_on_bound(1, 2, &grid.is_on_bounds, (3, 3))`.
pub fn is_on_bound(
    index1: i32,
    index2: i32,
    is_on_bounds: &[bool],
    (nb_delta_u, nb_delta_v): (i32, i32),
) -> bool {
    let diff: i32 = (index1 - index2).abs();
    if diff != 1 && diff != nb_delta_v + 1 {
        return false;
    }
    for i in 0..=nb_delta_u {
        if index1 == 1 + i * (nb_delta_v + 1) && index2 == index1 - 1 {
            return false;
        }
        if index1 == (1 + i) * (nb_delta_v + 1) && index2 == index1 + 1 {
            return false;
        }
    }
    let flag = |i: i32| {
        usize::try_from(i)
            .ok()
            .and_then(|at| is_on_bounds.get(at))
            .copied()
            .unwrap_or(false)
    };
    flag(index1) && flag(index2)
}

/// `SetDeflectionOverEstimation`: the deflection, at least 1e-4, and the box
/// enlarged by it; `set_deflection_over_estimation(0.5, &mut deflection, &mut bnd)`.
pub fn set_deflection_over_estimation(flec: f64, deflection: &mut f64, bnd: &mut BoundingBox3) {
    const THE_MIN_DEFLECTION: f64 = 0.0001;
    if flec < THE_MIN_DEFLECTION {
        *deflection = THE_MIN_DEFLECTION;
        bnd.enlarge(THE_MIN_DEFLECTION);
    } else {
        *deflection = flec;
        bnd.enlarge(flec);
    }
}
