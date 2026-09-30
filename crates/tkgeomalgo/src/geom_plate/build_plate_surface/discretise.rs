// Rust translation of Discretise of
// ModelingAlgorithms/TKGeomAlgo/GeomPlate/GeomPlate_BuildPlateSurface.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The discretisation of the curve constraints.
use super::GeomPlateBuildPlateSurface;
use super::intersect::ZonesPerCurve;
use crate::geom_plate::{GeomPlateCurveConstraint, live};
use crate::law::{LawFunction, LawInterpol};
use crate::plate::PlatePinpointConstraint;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeombase::gcpnts::AbscissaPoint;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Point2, Point3, Xyz};
use std::f64::consts::PI;

/// The intervals of the curvilinear abscissa law.
const NBINT: i32 = 20;

/// The state `Discretise` carries from one curve to the next, as natively: the
/// adaptor and the abscissa law are declared once, before the loop.
struct DiscretiseState {
    ac2d: Option<Geom2dAdaptorCurve>,
    acrlaw: LawInterpol,
}

/// Whether `inter` is strictly inside one of the `(zones[l], zones[l + 1])`.
fn zone_of(zones: &[f64], inter: f64) -> Option<usize> {
    zones
        .chunks(2)
        .position(|z| z.len() == 2 && inter > z[0] && inter < z[1])
}

/// `(1 - cos(k * PI / n)) / 2`.
fn cosine_ratio(k: i32, n: i32) -> f64 {
    (1.0 - (f64::from(k) * PI / f64::from(n)).cos()) / 2.0
}

impl GeomPlateBuildPlateSurface {
    /// The law from the 2d arc length to the parameter of `c2d`, sampled at 21
    /// cosine-distributed parameters; returns the total 2d length.
    fn arc_length_law(
        state: &mut DiscretiseState,
        c2d: Geom2dAdaptorCurve,
        uinit: f64,
        ufinal: f64,
    ) -> Result<f64, Failure> {
        let mut tab_p2d: Array1<'static, Point2> =
            Array1::with_bounds(1, NBINT + 1).map_err(errors::array_bounds)?;
        let length2d: f64 = AbscissaPoint::length_between(&c2d, uinit, ufinal)?;
        tab_p2d[1] = Point2::new(0.0, uinit);
        tab_p2d[NBINT + 1] = Point2::new(length2d, ufinal);
        for ii in 2..=NBINT {
            let u: f64 = uinit + (ufinal - uinit) * cosine_ratio(ii - 1, NBINT);
            tab_p2d[ii] = Point2::new(AbscissaPoint::length_between(&c2d, uinit, u)?, u);
        }
        state.acrlaw.set(&tab_p2d, false)?;
        state.ac2d = Some(c2d);
        Ok(length2d)
    }

    /// Loads a G0 pinpoint constraint at the parameter `inter` of an incompatible G1
    /// zone.
    fn load_zone_pinpoint(
        &mut self,
        state: &DiscretiseState,
        lin: &GeomPlateCurveConstraint,
        inter: f64,
    ) -> Result<(), Failure> {
        let ac2d = state.ac2d.as_ref().ok_or_else(|| {
            errors::null_object("GeomPlate_BuildPlateSurface::Discretise: no 2d curve loaded; expected a curve with a 2d curve")
        })?;
        let p2d: Point2 = ac2d.value(inter)?;
        let p3d: Point3 = lin.d0(inter)?;
        let pp: Point3 = self.surf_init_ref()?.d0(p2d.x(), p2d.y())?;
        let pdif = Xyz::new(-pp.x() + p3d.x(), -pp.y() + p3d.y(), -pp.z() + p3d.z());
        self.plate
            .load_pinpoint(&PlatePinpointConstraint::new(p2d.xy(), pdif, 0, 0));
        Ok(())
    }

    /// Keeps `inter` for the plate unless it is in an incompatible G1 zone, where a G0
    /// pinpoint constraint is loaded instead (the native loop over `k`).
    fn keep_outside_g1(
        &mut self,
        state: &DiscretiseState,
        i: usize,
        g1g1: &[f64],
        inter: f64,
    ) -> Result<(), Failure> {
        if zone_of(g1g1, inter).is_some() {
            let lin = self.lin_cont[i].clone();
            return self.load_zone_pinpoint(state, live(&lin, "a curve constraint")?, inter);
        }
        self.plate_cont_mut(i).push(inter);
        Ok(())
    }

    fn plate_cont_mut(&mut self, i: usize) -> &mut Vec<f64> {
        &mut self.plate_cont.get_or_insert_with(Vec::new)[i]
    }

    /// The parameter of point `j` of `nb_pnt` on curve `lin`: cosine distributed in
    /// 2d arc length when the curve has a 2d curve, in parameter otherwise.
    fn discretisation_point(
        state: &DiscretiseState,
        range: (f64, f64),
        length2d: Option<f64>,
        j: i32,
        nb_pnt: i32,
    ) -> Result<f64, Failure> {
        let (uinit, ufinal) = range;
        if j == nb_pnt {
            // "to avoid bug on Sun"
            return Ok(ufinal);
        }
        match length2d {
            Some(length2d) => {
                let cur_length: f64 =
                    length2d * (1.0 - (f64::from(j - 1) * PI / f64::from(nb_pnt - 1)).cos()) / 2.0;
                state.acrlaw.value(cur_length)
            }
            None => Ok(uinit + (ufinal - uinit) * cosine_ratio(j - 1, nb_pnt - 1)),
        }
    }

    /// Whether an end point of a curve without zone goes to the plate: always for a
    /// computed initial surface unless the 2d curve is a circle, never for a given one.
    fn keeps_ends(&self, lin: &GeomPlateCurveConstraint) -> Result<bool, Failure> {
        if self.surf_init_is_give {
            return Ok(false);
        }
        let adaptor = Geom2dAdaptorCurve::with_curve(&lin.curve2d_on_surf()?)?;
        Ok(adaptor.get_type()? != CurveType::Circle)
    }

    /// Discretises curve `i` (0-based) into `myParCont` and `myPlateCont`.
    fn discretise_curve(
        &mut self,
        state: &mut DiscretiseState,
        i: usize,
        zones: (&[f64], &[f64]),
    ) -> Result<(), Failure> {
        let lin_handle = self.lin_cont[i].clone();
        let lin = live(&lin_handle, "a curve constraint")?;
        let range = (lin.first_parameter()?, lin.last_parameter()?);
        let c2d = lin.curve2d_on_surf()?;
        let length2d: Option<f64> = match c2d.is_some() {
            true => Some(Self::arc_length_law(
                state,
                Geom2dAdaptorCurve::with_curve(&c2d)?,
                range.0,
                range.1,
            )?),
            false => None,
        };
        let nb_pnt: i32 = lin.nb_points();
        let (inter_zones, g1_zones) = zones;
        for j in 1..=nb_pnt {
            let inter: f64 = Self::discretisation_point(state, range, length2d, j, nb_pnt)?;
            if let Some(par) = self.par_cont.as_mut() {
                par[i].push(inter);
            }
            if !inter_zones.is_empty() {
                if zone_of(inter_zones, inter).is_none() {
                    self.keep_outside_g1(state, i, g1_zones, inter)?;
                }
            } else if !g1_zones.is_empty() {
                self.keep_outside_g1(state, i, g1_zones, inter)?;
            } else if self.keeps_ends(lin)? || (j > 1 && j < nb_pnt) {
                self.plate_cont_mut(i).push(inter);
            }
        }
        Ok(())
    }

    /// Discretises the curves (`Discretise`): `myParCont` gets `NbPoints` parameters
    /// per curve, cosine distributed (in 2d arc length when possible); `myPlateCont`
    /// keeps those outside the duplicated zones, and the points in incompatible G1
    /// zones become G0 pinpoint constraints of the plate.
    pub(super) fn discretise(
        &mut self,
        pnt_inter: &ZonesPerCurve,
        pnt_g1g1: &ZonesPerCurve,
    ) -> Result<(), Failure> {
        let n: usize = self.lin_cont.len();
        self.plate_cont = Some(vec![Vec::new(); n]);
        self.par_cont = Some(vec![Vec::new(); n]);
        let mut state = DiscretiseState {
            ac2d: None,
            acrlaw: LawInterpol::new(),
        };
        for i in 0..n {
            let zones = (pnt_inter[i].as_slice(), pnt_g1g1[i].as_slice());
            self.discretise_curve(&mut state, i, zones)?;
        }
        Ok(())
    }
}
