// Rust translation of CheckTgte, MinTgte, CheckCoonsAlgPatch, CheckTgteField, CheckApprox and
// CheckResult of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_ConstrainedFilling.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The checks of the tangent fields and of the result of a constrained filling.
use super::{GeomFillConstrainedFilling, field, live, required};
use crate::geom_fill::{GeomFillBoundary, GeomFillTgtField};
use rustycad_tkernel::standard::{Failure, Handle, real};
use rustycad_tkernel::storage::format_general;
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomSurface};
use rustycad_tkmath::bsplclib::{self, CurveSpan};
use rustycad_tkmath::gp::{Point3, Vector3};
use std::f64::consts::PI;
use std::fmt;

/// `std::cout << double` with the default stream flags.
fn stream_real(value: f64) -> String {
    format_general(value, 6)
}

fn text_error(_: fmt::Error) -> Failure {
    Failure::with_message(
        "Standard_Failure",
        "GeomFill_ConstrainedFilling: text output failed",
    )
}

/// The start, step and parameter of the 31 samples along side `i` of the surface
/// (`uu`, `vv`, `ww` and their steps), and whether the side runs along V.
struct SideSamples {
    uv: (f64, f64),
    duv: (f64, f64),
    ww: f64,
    dww: f64,
    along_v: bool,
}

fn side_samples(i: i32, (u1, u2, v1, v2): (f64, f64, f64, f64), nbp: f64) -> SideSamples {
    let (du, dv) = ((u2 - u1) / nbp, (v2 - v1) / nbp);
    match i {
        0 => SideSamples {
            uv: (u1, v1),
            duv: (du, 0.),
            ww: u1,
            dww: du,
            along_v: false,
        },
        1 => SideSamples {
            uv: (u2, v1),
            duv: (0., dv),
            ww: v1,
            dww: dv,
            along_v: true,
        },
        2 => SideSamples {
            uv: (u1, v2),
            duv: (du, 0.),
            ww: u1,
            dww: du,
            along_v: false,
        },
        3 => SideSamples {
            uv: (u1, v1),
            duv: (0., dv),
            ww: v1,
            dww: dv,
            along_v: true,
        },
        _ => SideSamples {
            uv: (0., 0.),
            duv: (0., 0.),
            ww: 0.,
            dww: 0.,
            along_v: false,
        },
    }
}

impl GeomFillConstrainedFilling {
    fn bound_handle(&self, i: i32) -> Result<Handle<dyn GeomFillBoundary>, Failure> {
        self.patch()?.bound(i)
    }
    /// `tgalg[I]`, Standard_OutOfRange outside 0..3 (an unchecked read natively).
    fn field_slot(&self, i: i32) -> Result<&Handle<dyn GeomFillTgtField>, Failure> {
        usize::try_from(i)
            .ok()
            .and_then(|k| self.tgalg.get(k))
            .ok_or_else(|| {
                errors::out_of_range(&format!(
                    "GeomFill_ConstrainedFilling: side {i}; expected 0..4"
                ))
            })
    }
    fn surface_ref(&self) -> Result<&GeomBSplineSurface, Failure> {
        self.surf
            .get()
            .ok_or_else(|| errors::null_object("GeomFill_ConstrainedFilling: no surface"))
    }
    /// `CheckTgte(I)`: false when the tangent field of side `i` changes side of the
    /// boundary's tangent plane at one of 13 points.
    pub(super) fn check_tgte(&self, i: i32) -> Result<bool, Failure> {
        let bou = self.bound_handle(i)?;
        let bou = live(&bou)?;
        if !bou.has_normals() {
            return Ok(true);
        }
        let (ll, mut pmix) = (1. / 12., 0.);
        for iu in 0..13 {
            let uu: f64 = f64::from(iu) * ll;
            let tgte: Vector3 = bou.d1(uu)?.1;
            let norm: Vector3 = bou.norm(uu)?;
            let vfield: Vector3 = field(self.field_slot(i)?)?.value(uu)?;
            let cur: f64 = vfield.dot(tgte.crossed(norm));
            if iu == 0 {
                pmix = cur;
            } else if pmix * cur < 0. {
                return Ok(false);
            }
        }
        Ok(true)
    }
    /// `MinTgte(I)`: the smallest magnitude of the tangent field of side `i` over 31
    /// points of [0.2, 0.8], into `mig[i]`.
    pub(super) fn min_tgte(&mut self, i: i32) -> Result<(), Failure> {
        let bou = self.bound_handle(i)?;
        if !live(&bou)?.has_normals() {
            return Ok(());
        }
        let mut minmag: f64 = real::real_last();
        for iu in 0..=30 {
            let uu: f64 = 0.2 + f64::from(iu) * 0.02;
            let temp: f64 = field(self.field_slot(i)?)?.value(uu)?.square_magnitude();
            if temp < minmag {
                minmag = temp;
            }
        }
        self.mig[i as usize] = minmag.sqrt();
        Ok(())
    }
    /// `CheckCoonsAlgPatch(I)`: evaluates the boundary and the patch's cross
    /// derivatives at 31 points of side `i` (the native member computes and discards
    /// them); its only effect is the exceptions; `f.check_coons_alg_patch(0)?`.
    pub fn check_coons_alg_patch(&self, i: i32) -> Result<(), Failure> {
        let samples = side_samples(i, self.surface_ref()?.bounds(), 30.);
        let bou = self.bound_handle(i)?;
        let (mut uv, mut ww) = (samples.uv, samples.ww);
        for _ in 0..=30 {
            live(&bou)?.value(ww)?;
            if samples.along_v {
                self.patch()?.d1u(uv.0, uv.1)?;
            } else {
                self.patch()?.d1v(uv.0, uv.1)?;
            }
            uv = (uv.0 + samples.duv.0, uv.1 + samples.duv.1);
            ww += samples.dww;
        }
        Ok(())
    }
    /// `CheckTgteField(I)`: the largest angle between the tangent field of side `i`
    /// and the plane normal to the boundary's normal, over 31 points, and whether the
    /// field changes side, written to `out` where the native member writes to
    /// `std::cout`; nothing without a field; `f.check_tgte_field(0, &mut text)?`.
    pub fn check_tgte_field(&self, i: i32, out: &mut dyn fmt::Write) -> Result<(), Failure> {
        let slot = self.field_slot(i)?;
        if slot.is_null() {
            return Ok(());
        }
        let bou = self.bound_handle(i)?;
        let bou = live(&bou)?;
        let (mut caplisse, mut maxang, mut pmix) = (false, 0., 0.);
        for iu in 0..=30 {
            let uu: f64 = f64::from(iu) / 30.;
            let d1: Vector3 = bou.d1(uu)?.1;
            let vtg: Vector3 = field(slot)?.value(uu)?;
            let vnor: Vector3 = bou.norm(uu)?;
            let mut vcros: Vector3 = d1.crossed(vnor);
            vcros.normalize()?;
            let cur: f64 = vtg.dot(vcros);
            if iu == 0 {
                pmix = cur;
            } else if pmix * cur < 0. {
                caplisse = true;
            }
            if vnor.magnitude() > 1.0e-15 && vtg.magnitude() > 1.0e-15 {
                let alpha: f64 = (PI / 2. - vnor.angle(vtg)?.abs()).abs();
                if alpha.abs() > maxang {
                    maxang = alpha.abs();
                }
            }
        }
        writeln!(
            out,
            "KAlgo angle max sur bord {i} : {}",
            stream_real(maxang)
        )
        .map_err(text_error)?;
        if caplisse {
            writeln!(out, "sur bord {i} le champ tangent change de cote!").map_err(text_error)?;
        }
        Ok(())
    }
    /// `CheckApprox(I)`: the largest distance between boundary `i` and its
    /// approximation, and (with a tangent field) the largest angle between the
    /// approximated field and the normal plane, over 31 points, written to `out`;
    /// `f.check_approx(0, &mut text)?`.
    pub fn check_approx(&self, i: i32, out: &mut dyn fmt::Write) -> Result<(), Failure> {
        let donor: bool = self.field_slot(i)?.is_some();
        let bou = self.bound_handle(i)?;
        let bou = live(&bou)?;
        let (mut maxang, mut maxdist) = (0., 0.);
        for iu in 0..=30 {
            let uu: f64 = f64::from(iu) / 30.;
            let pbound: Point3 = bou.value(uu)?;
            let papp: Point3 = self.approximated(&self.ncpol, i, uu)?;
            if donor {
                let vapp = Vector3::from(self.approximated(&self.ntpol, i, uu)?.xyz());
                let vbound: Vector3 = bou.norm(uu)?;
                if vapp.magnitude() > 1.0e-15 && vbound.magnitude() > 1.0e-15 {
                    let alpha: f64 = (PI / 2. - vbound.angle(vapp)?.abs()).abs();
                    maxang = if alpha.abs() > maxang {
                        alpha.abs()
                    } else {
                        maxang
                    };
                }
            }
            if papp.distance(&pbound) > maxdist {
                maxdist = papp.distance(&pbound);
            }
        }
        writeln!(out, "Controle approx/contrainte sur bord {i} : ").map_err(text_error)?;
        writeln!(out, "Distance max : {}", stream_real(maxdist)).map_err(text_error)?;
        if donor {
            let degrees: f64 = maxang * 180. / PI;
            writeln!(out, "Angle max    : {} deg", stream_real(degrees)).map_err(text_error)?;
        }
        Ok(())
    }
    /// `BSplCLib::D0` of pole row `i` of `rows` on the matched knots of its direction.
    fn approximated(&self, rows: &[super::PoleRow; 4], i: i32, u: f64) -> Result<Point3, Failure> {
        let side: usize = usize::try_from(i).ok().filter(|&k| k < 4).ok_or_else(|| {
            errors::out_of_range(&format!(
                "GeomFill_ConstrainedFilling: side {i}; expected 0..4"
            ))
        })?;
        let d: usize = side % 2;
        let span = CurveSpan {
            u,
            index: 0,
            degree: self.degree[d],
            periodic: false,
            knots: required(&self.nk[d], "nk")?,
            mults: Some(required(&self.nm[d], "nm")?),
        };
        bsplclib::d0(&span, required(&rows[side], "pole row")?, None)
    }
    /// `CheckResult(I)`: the largest distance between boundary `i` and the side of
    /// the surface, and (with a tangent field) the largest angle between their
    /// normals, over 31 points, written to `out`; `f.check_result(0, &mut text)?`.
    pub fn check_result(&self, i: i32, out: &mut dyn fmt::Write) -> Result<(), Failure> {
        let donor: bool = self.field_slot(i)?.is_some();
        let surf: &GeomBSplineSurface = self.surface_ref()?;
        let samples = side_samples(i, surf.bounds(), 30.);
        let bou = self.bound_handle(i)?;
        let bou = live(&bou)?;
        let (mut maxang, mut maxdist) = (0., 0.);
        let (mut uv, mut ww) = (samples.uv, samples.ww);
        for _ in 0..=30 {
            let pbound: Point3 = bou.value(ww)?;
            let pres: Point3 = if donor {
                let (pres, alpha) = normal_gap(surf, bou, uv, ww)?;
                maxang = alpha.filter(|&a| a > maxang).unwrap_or(maxang);
                pres
            } else {
                surf.eval_d0(uv.0, uv.1)?
            };
            if pres.distance(&pbound) > maxdist {
                maxdist = pres.distance(&pbound);
            }
            uv = (uv.0 + samples.duv.0, uv.1 + samples.duv.1);
            ww += samples.dww;
        }
        writeln!(out, "Controle resultat/contrainte sur bord {i} : ").map_err(text_error)?;
        writeln!(out, "Distance max : {}", stream_real(maxdist)).map_err(text_error)?;
        if donor {
            let angdeg: f64 = maxang * 180. / PI;
            writeln!(out, "Angle max    : {} deg", stream_real(angdeg)).map_err(text_error)?;
        }
        Ok(())
    }
}

/// The surface point at `uv` and the angle between the surface normal and the
/// boundary normal at `ww` (folded below a right angle; none when a normal is null).
fn normal_gap(
    surf: &GeomBSplineSurface,
    bou: &dyn GeomFillBoundary,
    uv: (f64, f64),
    ww: f64,
) -> Result<(Point3, Option<f64>), Failure> {
    let vbound: Vector3 = bou.norm(ww)?;
    let d1 = surf.eval_d1(uv.0, uv.1)?;
    let vres: Vector3 = d1.d1u.crossed(d1.d1v);
    if vres.magnitude() > 1.0e-15 && vbound.magnitude() > 1.0e-15 {
        let alpha: f64 = vres.angle(vbound)?.abs();
        return Ok((d1.point, Some(alpha.min((PI - alpha).abs()))));
    }
    Ok((d1.point, None))
}
