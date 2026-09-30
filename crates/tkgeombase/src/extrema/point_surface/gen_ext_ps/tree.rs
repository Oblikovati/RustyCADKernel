// Rust translation of Bnd_SphereUBTreeSelector(Min|Max), LengthOfIso,
// CorrectNbSamples, BuildTree and the tree part of Extrema_GenExtPS::Perform at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The tree algorithm of Extrema_GenExtPS: the grid nodes as null spheres in a
//! UB-tree, and one search from the nearest node and one from the farthest.
use super::GenExtPS;
use super::grid::{param_array, uniform_params};
use crate::extrema::pon::POnSurfParams;
use crate::native_failure::{array_failure, failure};
use rustycad_tkernel::ncollection::{Array2, BoundingVolume, TreeSelector, UBTree, UBTreeFiller};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom_grid_eval::GeomGridEvalSurface;
use rustycad_tkmath::bnd::BoundingSphere;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point3, Xyz};
use std::cell::Cell;

/// `Bnd_Sphere` as the bounding volume of `NCollection_UBTree<int, Bnd_Sphere>`.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct TreeSphere(BoundingSphere);

impl BoundingVolume for TreeSphere {
    fn add(&mut self, other: &Self) {
        self.0.add(&other.0);
    }
    fn is_out(&self, other: &Self) -> bool {
        self.0.is_out_sphere(&other.0)
    }
    fn square_extent(&self) -> f64 {
        self.0.square_extent()
    }
}

/// `Bnd_SphereUBTreeSelectorMin` (`nearest`) and `Bnd_SphereUBTreeSelectorMax`: the
/// node sphere nearest to (or farthest from) the check point. The bound is
/// tightened by `Reject` as natively (through a `const_cast`).
struct SphereSelector<'s> {
    xyz: Xyz,
    spheres: &'s [BoundingSphere],
    sol: BoundingSphere,
    bound: Cell<f64>,
    nearest: bool,
}

impl TreeSelector<i32, TreeSphere> for SphereSelector<'_> {
    fn reject(&self, bounds: &TreeSphere) -> bool {
        let mut bound: f64 = self.bound.get();
        let out: bool = bounds.0.is_out_point(self.xyz, &mut bound);
        self.bound.set(bound);
        out
    }
    fn accept(&mut self, object: &i32) -> bool {
        let sphere: BoundingSphere = self.spheres[*object as usize];
        let cur_dist: f64 = sphere.distance(self.xyz);
        let sol_dist: f64 = self.sol.distance(self.xyz);
        let better: bool = if self.nearest {
            cur_dist < sol_dist
        } else {
            cur_dist > sol_dist
        };
        if !better {
            return false;
        }
        self.sol = sphere;
        let bound: f64 = self.bound.get();
        if (self.nearest && cur_dist < bound) || (!self.nearest && cur_dist > bound) {
            self.bound.set(cur_dist);
        }
        true
    }
    fn stop(&self) -> bool {
        false
    }
}

// `LengthOfIso`: the length of the polyline of `nb_pnts` points of the U (`is_u`)
// or V iso at `par`, from `par1` to `par2`.
fn length_of_iso(
    surface: &dyn Adaptor3dSurface,
    is_u: bool,
    (par1, par2): (f64, f64),
    nb_pnts: i32,
    par: f64,
) -> Result<f64, Failure> {
    let value = |t: f64| {
        if is_u {
            surface.value(par, t)
        } else {
            surface.value(t, par)
        }
    };
    let d_par: f64 = (par2 - par1) / f64::from(nb_pnts - 1);
    let mut len: f64 = 0.;
    let mut p1: Point3 = value(par1)?;
    let mut t: f64 = par1 + d_par;
    for _ in 2..=nb_pnts {
        let p2: Point3 = value(t)?;
        len += p1.distance(&p2);
        p1 = p2;
        t += d_par;
    }
    Ok(len)
}

// The iso length at `par`, or at `inner` when it is not above 1e-3.
fn iso_length(
    surface: &dyn Adaptor3dSurface,
    is_u: bool,
    range: (f64, f64),
    nbp: i32,
    (par, inner): (f64, f64),
) -> Result<f64, Failure> {
    let min_len: f64 = 1.0e-3;
    let len: f64 = length_of_iso(surface, is_u, range, nbp, par)?;
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    if !(len <= min_len) {
        return Ok(len);
    }
    Ok(std_max(
        length_of_iso(surface, is_u, range, nbp, inner)?,
        len,
    ))
}

// `CorrectNbSamples`: multiplies the V samples by the logarithm of the ratio of
// the steps when it is above 10 or below 0.1 (natively the U samples never change).
fn correct_nb_samples(
    surface: &dyn Adaptor3dSurface,
    (u1, u2): (f64, f64),
    nb_u: i32,
    (v1, v2): (f64, f64),
    nb_v: &mut i32,
) -> Result<(), Failure> {
    let nbp: i32 = 23.min(*nb_v);
    let len_u1 = iso_length(surface, true, (v1, v2), nbp, (u1, 0.7 * u1 + 0.3 * u2))?;
    let len_u2 = iso_length(surface, true, (v1, v2), nbp, (u2, 0.3 * u1 + 0.7 * u2))?;
    let len_v1 = iso_length(surface, false, (u1, u2), nbp, (v1, 0.7 * v1 + 0.3 * v2))?;
    let len_v2 = iso_length(surface, false, (u1, u2), nbp, (v2, 0.3 * v1 + 0.7 * v2))?;
    let max_step_v: f64 = std_max(len_u1 / f64::from(*nb_v), len_u2 / f64::from(*nb_v));
    let max_step_u: f64 = std_max(len_v1 / f64::from(nb_u), len_v2 / f64::from(nb_u));
    let ratio: f64 = max_step_v / max_step_u;
    let mult: i32 = if ratio > 10. {
        real::real_to_int(ratio.ln())
    } else if ratio < 0.1 {
        real::real_to_int(-ratio.ln())
    } else {
        return Ok(());
    };
    if mult > 1 {
        *nb_v *= mult;
    }
    Ok(())
}

impl GenExtPS<'_> {
    /// `BuildTree()`: once until the next Initialize, the samples (raised for a
    /// B-spline, corrected by CorrectNbSamples), the uniform parameters and the tree
    /// of the grid nodes.
    pub(super) fn build_tree(&mut self) -> Result<(), Failure> {
        if self.sphere_tree.is_some() {
            return Ok(());
        }
        let surface_ref = self.surface_ref()?;
        let surface: &dyn Adaptor3dSurface = surface_ref.get();
        if surface.get_type()? == SurfaceType::BSplineSurface {
            let bspline = surface.bspline()?;
            let u_value: i32 = bspline.u_degree() * bspline.nb_u_knots();
            let v_value: i32 = bspline.v_degree() * bspline.nb_v_knots();
            // 300 is the value used for singular points (Extrema_ExtPS::Initialize).
            if u_value > self.usample {
                self.usample = u_value.min(300);
            }
            if v_value > self.vsample {
                self.vsample = v_value.min(300);
            }
        }
        correct_nb_samples(
            surface,
            (self.umin, self.usup),
            self.usample,
            (self.vmin, self.vsup),
            &mut self.vsample,
        )?;
        let u_params: Vec<f64> = uniform_params(self.umin, self.usup, self.usample);
        let v_params: Vec<f64> = uniform_params(self.vmin, self.vsup, self.vsample);
        let grid: Array2<'static, Point3> = GeomGridEvalSurface::from_adaptor(surface)?
            .evaluate_grid(&param_array(&u_params)?, &param_array(&v_params)?)?;
        (self.u_params, self.v_params) = (Some(u_params), Some(v_params));
        self.fill_tree(&grid)
    }

    // The node spheres, in the array (`0 .. NbU NbV`, the last one a default sphere)
    // and in the tree by UBTreeFiller.
    fn fill_tree(&mut self, grid: &Array2<'static, Point3>) -> Result<(), Failure> {
        let mut tree: UBTree<i32, TreeSphere> = UBTree::default();
        self.sphere_array =
            vec![BoundingSphere::default(); (self.usample * self.vsample + 1) as usize];
        {
            let mut filler = UBTreeFiller::new(&mut tree, None, true);
            let mut i: i32 = 0;
            for no_u in 1..=self.usample {
                for no_v in 1..=self.vsample {
                    let node: Point3 = *grid
                        .value(no_u, no_v)
                        .map_err(|error| array_failure(error, "NCollection_Array2::Value"))?;
                    let sphere = BoundingSphere::new(node.xyz(), 0., no_u, no_v);
                    filler.add(i, TreeSphere(sphere));
                    self.sphere_array[i as usize] = sphere;
                    i += 1;
                }
            }
            filler.fill();
        }
        self.sphere_tree = Some(tree);
        Ok(())
    }

    /// The search from the node nearest to (`nearest`) or farthest from `point`.
    pub(super) fn find_tree_extremum(
        &mut self,
        point: Point3,
        nearest: bool,
    ) -> Result<(), Failure> {
        let sol: BoundingSphere = self.sphere_array[0];
        let sphere: BoundingSphere = {
            let mut selector = SphereSelector {
                xyz: point.xyz(),
                spheres: &self.sphere_array,
                sol,
                bound: Cell::new(if nearest { f64::MAX } else { 0. }),
                nearest,
            };
            if let Some(tree) = &self.sphere_tree {
                tree.select(&mut selector);
            }
            selector.sol
        };
        let u: f64 = param_value(self.u_params.as_deref(), sphere.u())?;
        let v: f64 = param_value(self.v_params.as_deref(), sphere.v())?;
        let mut params = POnSurfParams::new(u, v, self.surface()?.value(u, v)?);
        params.set_sqr_distance(point.square_distance(params.value()));
        params.set_indices(sphere.u(), sphere.v());
        self.find_solution(&params)
    }
}

// `myUParams->Value(index)`: Standard_OutOfRange outside `[1, Length]`.
fn param_value(params: Option<&[f64]>, index: i32) -> Result<f64, Failure> {
    let params: &[f64] = params.unwrap_or_default();
    if index < 1 || index as usize > params.len() {
        return Err(failure(
            "Standard_OutOfRange",
            &format!(
                "NCollection_Array1::Value - index {index} is outside [1, {}]",
                params.len()
            ),
        ));
    }
    Ok(params[(index - 1) as usize])
}
