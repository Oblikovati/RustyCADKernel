// Rust translation of Disc2dContour and Disc3dContour of
// ModelingAlgorithms/TKGeomAlgo/GeomPlate/GeomPlate_BuildPlateSurface.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The discretised contour of the constraints, in the initial surface parameters and
//! in space, for the criteria of `GeomPlate_MakeApprox`.
use super::GeomPlateBuildPlateSurface;
use crate::geom_plate::{GeomPlateCurveConstraint, live};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Xy, Xyz};

/// The parameters of curve `par` that the contour samples: the first one, then a
/// quarter, the middle, three quarters and the end of each later interval.
fn contour_parameters(par: &[f64]) -> Vec<f64> {
    let mut params: Vec<f64> = Vec::with_capacity(4 * par.len());
    let Some(&first) = par.first() else {
        return params;
    };
    params.push(first);
    for j in 2..par.len() {
        let (uj, ujp1) = (par[j - 1], par[j]);
        params.extend([
            (ujp1 + 3.0 * uj) / 4.0,
            (ujp1 + uj) / 2.0,
            (3.0 * ujp1 + uj) / 4.0,
            ujp1,
        ]);
    }
    params
}

impl GeomPlateBuildPlateSurface {
    /// The parameters of curve `i` that the contour samples; Standard_OutOfRange
    /// for a curve without parameter, as `Value(1)` natively.
    fn contour_of(&self, i: i32) -> Result<Vec<f64>, Failure> {
        let par: &[f64] = self.par_cont_of(i)?;
        if par.is_empty() {
            return Err(rustycad_tkg2d::geom2d::errors::out_of_range(&format!(
                "GeomPlate_BuildPlateSurface: curve {i} has no discretisation parameter; expected at least one"
            )));
        }
        Ok(contour_parameters(par))
    }

    /// The contour in the parameters of the initial surface (`Disc2dContour`; the
    /// count `nbp` is unused): the point constraints, then the samples of each curve,
    /// projected when a curve has no 2d curve. The projector is initialized with the
    /// current parametric tolerances. `builder.disc2d_contour(4, &mut seq2d)?`.
    pub fn disc2d_contour(&mut self, _nbp: i32, seq2d: &mut Sequence<Xy>) -> Result<(), Failure> {
        seq2d.clear(None);
        let (tol_u, tol_v) = (self.tol_u, self.tol_v);
        self.projector_on_init(|_, _| Ok((tol_u, tol_v)))?;
        for i in 1..=self.nt_pnt_cont() {
            let pnt = self.pnt(i)?;
            if pnt.order() != -1 {
                seq2d.append(pnt.pnt2d_on_surf().xy());
            }
        }
        for i in 1..=self.nt_lin_cont() {
            let lin_handle = self.lin_cont[(i - 1) as usize].clone();
            let lin: &GeomPlateCurveConstraint = live(&lin_handle, "a curve constraint")?;
            if lin.order() == -1 {
                continue;
            }
            for u in self.contour_of(i)? {
                let p2d: Point2 = self.uv_on_init(lin, u, || lin.d0(u))?;
                seq2d.append(p2d.xy());
            }
        }
        Ok(())
    }

    /// The contour in space (`Disc3dContour`; the count `nbp` is unused): the points
    /// (`iordre` 0) or the normals `D1U ^ D1V` (otherwise) of the point constraints,
    /// then of the samples of each curve. The projector is initialized with the
    /// resolutions of `Tol3d`, as natively. `builder.disc3d_contour(4, 0, &mut seq3d)?`.
    pub fn disc3d_contour(
        &mut self,
        _nbp: i32,
        iordre: i32,
        seq3d: &mut Sequence<Xyz>,
    ) -> Result<(), Failure> {
        seq3d.clear(None);
        self.projector_on_init(|surface, tol3d| {
            Ok((surface.u_resolution(tol3d)?, surface.v_resolution(tol3d)?))
        })?;
        for i in 1..=self.nt_pnt_cont() {
            let pnt = self.pnt(i)?;
            if pnt.order() == -1 {
                continue;
            }
            if iordre == 0 {
                seq3d.append(pnt.d0().xyz());
            } else {
                let (_, v1h, v2h) = pnt.d1();
                seq3d.append(*(v1h ^ v2h).xyz());
            }
        }
        for i in 1..=self.nt_lin_cont() {
            let lin = self.lin(i)?;
            if lin.order() == -1 {
                continue;
            }
            for u in self.contour_of(i)? {
                if iordre == 0 {
                    seq3d.append(lin.d0(u)?.xyz());
                } else {
                    let d1 = lin.d1(u)?;
                    seq3d.append(*(d1.d1u ^ d1.d1v).xyz());
                }
            }
        }
        Ok(())
    }
}
