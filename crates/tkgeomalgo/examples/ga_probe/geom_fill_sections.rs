//! GeomFill section law readers (the uniform, evolved and N-section laws) shared
//! by the GeomFill probe operations, the Rust twin of the readers of
//! tools/oracle/tkgeomalgo_probe_geomfill_laws.cxx.
use super::common::{Reader, int, pnt, real, show};
use super::ics_geometry::{geom_curve, geom_surface};
use super::law::bspline;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomCurve};
use rustycad_tkgeomalgo::geom_fill::*;
use rustycad_tkgeomalgo::law::{LawBSpFunc, LawConstant, LawFunction, LawLinear};
use rustycad_tkmath::gp::Transform3;

/// `ga_gfl_law`.
pub fn gfl_law(r: &mut Reader) -> Result<Handle<dyn LawFunction>, Failure> {
    Ok(match r.integer() {
        0 => {
            let c = LawConstant::new();
            let (v, f, l) = (r.real(), r.real(), r.real());
            c.set(v, f, l);
            Handle::new(c).cast()
        }
        1 => {
            let law = LawLinear::new();
            let (a, b, c, d) = (r.real(), r.real(), r.real(), r.real());
            law.set(a, b, c, d);
            Handle::new(law).cast()
        }
        _ => {
            let b = bspline(r)?;
            let (f, l) = (r.real(), r.real());
            Handle::new(LawBSpFunc::with_curve(&b, f, l)).cast()
        }
    })
}

/// `ga_gfl_section`.
pub fn gfl_section(r: &mut Reader) -> Result<Handle<dyn GeomFillSectionLaw>, Failure> {
    let kind = r.integer();
    if kind == 2 {
        return gfn_nsections(r);
    }
    let c = geom_curve(r);
    if kind == 0 {
        let (f, l) = (r.real(), r.real());
        return Ok(Handle::new(GeomFillUniformSection::new(&c, f, l)?).cast());
    }
    let law = gfl_law(r)?;
    Ok(Handle::new(GeomFillEvolvedSection::new(&c, &law)?).cast())
}

/// `ga_gfl_curve`.
pub fn gfl_curve(c: Handle<dyn GeomCurve>) {
    let Some(curve) = c.get() else {
        int(-1);
        return;
    };
    let (f, l) = (curve.first_parameter(), curve.last_parameter());
    real(f);
    real(l);
    show(curve.value(0.5 * (f + l)), pnt);
}

/// `ga_gfn_trsf`: a transformation from 12 reals (`SetValues`).
fn gfn_trsf(r: &mut Reader) -> Result<Transform3, Failure> {
    let mut rows = [[0.0; 4]; 3];
    rows.iter_mut().flatten().for_each(|v| *v = r.real());
    let mut t = Transform3::default();
    t.set_values(rows)?;
    Ok(t)
}

/// `ga_gfn_nsections`: `ctor n curves... np params... [extras]`, ctor 0 `NC`, 1
/// `NC NP`, 2 `NC NP UF UL`, 3 `NC NP UF UL VF VL`, 4 `NC Trsfs NP UF UL VF VL
/// Surf` (the transformations read after the parameters, as `ntrsf [12 reals]...`).
fn gfn_nsections(r: &mut Reader) -> Result<Handle<dyn GeomFillSectionLaw>, Failure> {
    let ctor = r.integer();
    let n = r.integer();
    let nc: Vec<Handle<dyn GeomCurve>> = (0..n).map(|_| geom_curve(r)).collect();
    let np = r.reals();
    let law = match ctor {
        0 => GeomFillNSections::new(&nc)?,
        1 => GeomFillNSections::with_params(&nc, &np)?,
        2 => {
            let (uf, ul) = (r.real(), r.real());
            GeomFillNSections::with_section_domain(&nc, &np, uf, ul)?
        }
        3 => {
            let (u, v) = ((r.real(), r.real()), (r.real(), r.real()));
            GeomFillNSections::with_domain(&nc, &np, u, v)?
        }
        _ => gfn_reference(r, &nc, &np)?,
    };
    Ok(Handle::new(law).cast())
}

fn gfn_reference(
    r: &mut Reader,
    nc: &[Handle<dyn GeomCurve>],
    np: &[f64],
) -> Result<GeomFillNSections, Failure> {
    let nt = r.integer();
    let trsfs = (0..nt)
        .map(|_| gfn_trsf(r))
        .collect::<Result<Vec<_>, _>>()?;
    let (u, v) = ((r.real(), r.real()), (r.real(), r.real()));
    let surf: Handle<GeomBSplineSurface> = geom_surface(r).cast();
    GeomFillNSections::with_reference(nc, &trsfs, np, u, v, &surf)
}
