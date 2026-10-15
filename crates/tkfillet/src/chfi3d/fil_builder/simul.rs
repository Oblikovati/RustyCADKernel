// Rust translation of ChFi3d_FilBuilder::SimulKPart, the two-surface
// ChFi3d_FilBuilder::SimulSurf and the file statics MaxRad and SimulParams
// (ChFi3d_FilBuilder.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The simulation of the fillets: their circular sections without the surfaces.
use super::{ChFi3dFilBuilder, fil_spine};
use crate::blend::BlendPoint;
use crate::blend_func::{ConstRad, ConstRadInv, EvolRad, EvolRadInv};
use crate::brep_blend::BRepBlendLineHandle;
use crate::chfi_ds::{CircSection, ElSpine, SpineHandle, SurfDataHandle, lock};
use crate::chfi3d::builder::{ComputeCall, SurfCall, SurfOut};
use crate::chfi3d::fil_common_point;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::law::LawFunction;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Circle3, Point2};

/// The sections of a simulation and the 2d points of its ends on the two faces.
pub(super) struct SimulSections {
    pub sections: Vec<CircSection>,
    pub pf: [Point2; 4],
}

/// The sections as a shared array (`NCollection_HArray1<ChFiDS_CircSection>`, 1-based),
/// seen as the simulation object of a surface data.
pub(super) fn sections_transient(
    sections: Vec<CircSection>,
) -> Result<Handle<StandardTransient>, Failure> {
    if sections.is_empty() {
        return Ok(Handle::default());
    }
    let range = |e| Failure::with_message("Standard_RangeError", &format!("{e:?}"));
    let mut a = Array1::with_bounds(1, sections.len() as i32).map_err(range)?;
    for (i, s) in (1..).zip(sections) {
        a.set_value(i, s).map_err(range)?;
    }
    Ok(Handle::new(HArray1::from_owned(a)).cast())
}

/// `MaxRad(fsp, E)`: the radius of the edge when constant, else the largest radius of the
/// pairs and laws of the spine.
pub(super) fn max_rad(spine: &SpineHandle, e: &Edge) -> Result<f64, Failure> {
    let g = lock(spine);
    let fsp = fil_spine(&g, "MaxRad")?;
    let ie = fsp.index_of_edge(e);
    if fsp.is_constant_on_edge(ie)? {
        return fsp.radius_on_edge_index(ie);
    }
    fsp.max_rad_from_seq_and_laws()
}

/// `SimulParams(HGuide, fsp, MaxStep, Fleche)`: the step (a twentieth of the guide) and
/// the deflection (a twentieth of the largest radius, sampled at six points of the law).
pub(super) fn simul_params(
    hguide: &Handle<ElSpine>,
    spine: &SpineHandle,
) -> Result<(f64, f64), Failure> {
    let (la, fi) = (hguide.last_parameter()?, hguide.first_parameter()?);
    let longueur = la - fi;
    let max_step = longueur * 0.05;
    let g = lock(spine);
    let fsp = fil_spine(&g, "SimulParams")?;
    let radiussect = if fsp.is_constant() {
        fsp.radius()?
    } else {
        let lc = fsp.law(hguide)?;
        let lc = lc
            .get()
            .ok_or_else(|| Failure::with_message("Standard_NullObject", "SimulParams: no law"))?;
        let mut r = 0.;
        for i in 0..=5 {
            let w = fi + f64::from(i) * longueur * 0.2;
            let temp = lc.value(w)?;
            if temp > r {
                r = temp;
            }
        }
        r
    };
    Ok((max_step, radiussect * 0.05))
}

/// The law of the elementary spine as a general law (`fsp->Law(HGuide)`).
pub(super) fn law_of(
    spine: &SpineHandle,
    hguide: &Handle<ElSpine>,
) -> Result<Handle<dyn LawFunction>, Failure> {
    let g = lock(spine);
    let fsp = fil_spine(&g, "PerformSurf : this is not the spine of a fillet")?;
    Ok(fsp.law(hguide)?.cast())
}

/// The sections of a line of two surfaces from the section function `section`.
pub(super) fn surface_sections(
    lin: &BRepBlendLineHandle,
    mut section: impl FnMut(f64, [f64; 4]) -> Result<(f64, f64, Circle3), Failure>,
) -> Result<SimulSections, Failure> {
    let l = lock(lin);
    let nbp = l.nb_points();
    let mut out = SimulSections {
        sections: Vec::with_capacity(nbp.max(0) as usize),
        pf: [Point2::default(); 4],
    };
    for i in 1..=nbp {
        let p: &BlendPoint = l.point(i)?;
        let (u1, v1) = p.parameters_on_s1()?;
        let (u2, v2) = p.parameters_on_s2()?;
        let (p1, p2, ci) = section(p.parameter(), [u1, v1, u2, v2])?;
        let mut isec = CircSection::new();
        isec.set_circle(ci, p1, p2);
        out.sections.push(isec);
        if i == 1 {
            out.pf[0] = Point2::new(u1, v1);
            out.pf[2] = Point2::new(u2, v2);
        }
        if i == nbp {
            out.pf[1] = Point2::new(u1, v1);
            out.pf[3] = Point2::new(u2, v2);
        }
    }
    Ok(out)
}

impl ChFi3dFilBuilder {
    /// `SimulKPart(SD)`: the sections of a particular case (two on a cylinder, a section
    /// every five degrees on a torus or a sphere).
    pub(super) fn simul_kpart_impl(&self, sd: &SurfDataHandle) -> Result<(), Failure> {
        let (s, p1f, p1l, p2f, p2l) = {
            let g = lock(sd);
            let at = |on_s: i32, first: bool| -> Result<Point2, Failure> {
                let fi = g.interference(on_s);
                let pc = fi
                    .pcurve_on_surf()
                    .get()
                    .ok_or_else(|| Failure::with_message("Standard_NullObject", "SimulKPart"))?;
                pc.value(fi.parameter(first))
            };
            (
                self.base.ds.surface(g.surf()).surface().clone(),
                at(1, true)?,
                at(1, false)?,
                at(2, true)?,
                at(2, false)?,
            )
        };
        let a_s = GeomAdaptorSurface::with_surface(&s)?;
        let sections = match a_s.get_type()? {
            SurfaceType::Cylinder => {
                let (u1, u2) = (p1f.x(), p2f.x());
                let v1 = p1f.y().max(p2f.y());
                let v2 = p1l.y().min(p2l.y());
                let cy = a_s.cylinder()?;
                let mut sec1 = CircSection::new();
                let mut sec2 = CircSection::new();
                sec1.set_circle(
                    elslib::cylinder_v_iso(cy.position(), cy.radius(), v1),
                    u1,
                    u2,
                );
                sec2.set_circle(
                    elslib::cylinder_v_iso(cy.position(), cy.radius(), v2),
                    u1,
                    u2,
                );
                vec![sec1, sec2]
            }
            SurfaceType::Torus => {
                let to = a_s.torus()?;
                let (majr, minr) = (to.major_radius(), to.minor_radius());
                revolution_sections((p1f, p1l, p2f, p2l), |u| {
                    elslib::torus_u_iso(to.position(), majr, minr, u)
                })
            }
            SurfaceType::Sphere => {
                let sp = a_s.sphere()?;
                let rad = sp.radius();
                revolution_sections((p1f, p1l, p2f, p2l), |u| {
                    elslib::sphere_u_iso(sp.position(), rad, u)
                })
            }
            _ => Vec::new(),
        };
        lock(sd).set_simul(&sections_transient(sections)?);
        Ok(())
    }

    /// The two-surface `SimulSurf`: the walking of the simulation with the fillet
    /// function (constant or evolutive), its sections stored, the ends set and the
    /// obstacles at the ends kept only when no face continues the contour there.
    pub(super) fn simul_surf_impl(
        &mut self,
        data: &mut SurfDataHandle,
        call: &SurfCall<'_>,
        out: &mut SurfOut,
    ) -> Result<bool, Failure> {
        const MSG: &str = "SimulSurf : this is not the spine of the fillet";
        let radius = super::constant_radius(call.spine, MSG)?;
        let (max_step, locfleche) = simul_params(call.guide, call.spine)?;
        let pfirst = out.first;
        self.extend_ends_to_spine(call.spine, out)?;
        let s1: Handle<dyn Adaptor3dSurface> = call.s1.clone().cast();
        let s2: Handle<dyn Adaptor3dSurface> = call.s2.clone().cast();
        let guide: Handle<dyn Adaptor3dCurve> = call.guide.clone().cast();
        let mut walk = call.walk;
        walk.max_step = max_step;
        walk.fleche = locfleche;
        let ccall = ComputeCall {
            hguide: call.guide,
            spine: None,
            s1: &s1,
            i1: call.i1,
            s2: &s2,
            i2: call.i2,
            pfirst,
            walk,
            soldep: call.soldep,
            rec: (call.rec_on_s1, call.rec_on_s2),
        };
        let mut lin = None;
        let secs = if let Some(rad) = radius {
            let mut func = ConstRad::new(&s1, &s2, &guide)?;
            let mut finv = ConstRadInv::new(&s1, &s2, &guide);
            func.set_radius(rad, call.choix);
            finv.set_radius(rad, call.choix);
            func.set_shape(self.section_shape);
            self.base.done =
                self.base
                    .simul_data(&ccall, None, (&mut func, &mut finv), 4, out, &mut lin)?;
            if !self.base.done {
                return Ok(false);
            }
            let lin = lin.clone().ok_or_else(|| Failure::new("SimulSurf"))?;
            surface_sections(&lin, |w, uv| func.section_circle(w, uv))?
        } else {
            let law = law_of(call.spine, call.guide)?;
            let mut func = EvolRad::new(&s1, &s2, &guide, &law)?;
            let mut finv = EvolRadInv::new(&s1, &s2, &guide, &law);
            func.set_choix(call.choix);
            finv.set_choix(call.choix);
            func.set_shape(self.section_shape);
            self.base.done =
                self.base
                    .simul_data(&ccall, None, (&mut func, &mut finv), 4, out, &mut lin)?;
            if !self.base.done {
                return Ok(false);
            }
            let lin = lin.clone().ok_or_else(|| Failure::new("SimulSurf"))?;
            surface_sections(&lin, |w, uv| func.section_circle(w, uv))?
        };
        let lin = lin.ok_or_else(|| Failure::new("SimulSurf"))?;
        self.store_simulation(data, secs, &lin, [1, 2])?;
        let reverse = !call.walk.forward || call.walk.inside;
        if out.intf != 0 && reverse {
            out.intf = self.obstacle_at_end(data, call, true)?;
        }
        if out.intl != 0 {
            out.intl = self.obstacle_at_end(data, call, false)?;
        }
        Ok(true)
    }

    /// `if (intf) First = fsp->FirstParameter(1); if (intl) Last = ...`: the ends of a
    /// walking meeting obstacles taken at the ends of the spine.
    pub(super) fn extend_ends_to_spine(
        &self,
        spine: &SpineHandle,
        out: &mut SurfOut,
    ) -> Result<(), Failure> {
        let g = lock(spine);
        if out.intf != 0 {
            out.first = g.first_parameter_of(1)?;
        }
        if out.intl != 0 {
            out.last = g.last_parameter_of(g.nb_edges())?;
        }
        Ok(())
    }

    /// The sections and the 2d points stored in the surface data, and the common points
    /// of the ends of the line: the line's first surface on the side `sides[0]` of the
    /// data, its second on `sides[1]`.
    pub(super) fn store_simulation(
        &self,
        data: &SurfDataHandle,
        secs: SimulSections,
        lin: &BRepBlendLineHandle,
        sides: [i32; 2],
    ) -> Result<(), Failure> {
        let simul = sections_transient(secs.sections)?;
        {
            let mut d = lock(data);
            d.set_simul(&simul);
            d.set_2d_points(secs.pf);
        }
        self.store_line_ends(data, lin, sides)
    }

    /// The common points of the ends of the line: its first surface on the side
    /// `sides[0]` of the data, its second on `sides[1]` (`ChFi3d_FilCommonPoint`).
    pub(super) fn store_line_ends(
        &self,
        data: &SurfDataHandle,
        lin: &BRepBlendLineHandle,
        sides: [i32; 2],
    ) -> Result<(), Failure> {
        let l = lock(lin);
        let mut d = lock(data);
        let tol = self.base.tolapp3d;
        fil_common_point(
            l.start_point_on_first(),
            l.transition_on_s1()?,
            true,
            d.change_vertex(true, sides[0]),
            tol,
        )?;
        fil_common_point(
            l.end_point_on_first(),
            l.transition_on_s1()?,
            false,
            d.change_vertex(false, sides[0]),
            tol,
        )?;
        fil_common_point(
            l.start_point_on_second(),
            l.transition_on_s2()?,
            true,
            d.change_vertex(true, sides[1]),
            tol,
        )?;
        fil_common_point(
            l.end_point_on_second(),
            l.transition_on_s2()?,
            false,
            d.change_vertex(false, sides[1]),
            tol,
        )
    }

    /// Whether an end of the simulation stays an obstacle: false when a face continues
    /// the contour at the common point on an arc of the first, else of the second face
    /// (`intf = !SearchFace(...)`).
    fn obstacle_at_end(
        &self,
        data: &SurfDataHandle,
        call: &SurfCall<'_>,
        first: bool,
    ) -> Result<i32, Failure> {
        let (cp1, cp2) = {
            let d = lock(data);
            (d.vertex(first, 1).clone(), d.vertex(first, 2).clone())
        };
        let mut result = 1;
        let mut ok = false;
        if cp1.is_on_arc() {
            let mut bid = Face::default();
            result = i32::from(!self.base.search_face(
                Some(call.spine),
                &cp1,
                &call.s1.face(),
                &mut bid,
            )?);
            ok = result != 0;
        }
        if cp2.is_on_arc() && !ok {
            let mut bid = Face::default();
            result = i32::from(!self.base.search_face(
                Some(call.spine),
                &cp2,
                &call.s2.face(),
                &mut bid,
            )?);
        }
        Ok(result)
    }
}

/// The sections of a torus or a sphere every five degrees between the common range of
/// the two sides (at least two).
fn revolution_sections(
    (p1f, p1l, p2f, p2l): (Point2, Point2, Point2, Point2),
    iso: impl Fn(f64) -> Circle3,
) -> Vec<CircSection> {
    let (v1, v2) = (p1f.y(), p2f.y());
    let u1 = p1f.x().max(p2f.x());
    let u2 = p1l.x().min(p2l.x());
    let ang = u2 - u1;
    let n = ((36. * ang / std::f64::consts::PI + 1.) as i32).max(2);
    (1..=n)
        .map(|i| {
            let u = u1 + f64::from(i - 1) * (u2 - u1) / f64::from(n - 1);
            let mut isec = CircSection::new();
            isec.set_circle(iso(u), v1, v2);
            isec
        })
        .collect()
}
