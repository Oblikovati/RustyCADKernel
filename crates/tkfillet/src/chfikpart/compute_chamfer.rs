//! The chamfers and the corners of ChFiKPart_ComputeData.
use super::ch_pln_con::{make_ch_asym_pln_con, make_chamfer_pln_con};
use super::ch_pln_cyl::{make_ch_asym_pln_cyl_circle, make_chamfer_pln_cyl_circle};
use super::ch_pln_cyl_line::{make_ch_asym_pln_cyl_line, make_chamfer_pln_cyl_line};
use super::compute_data::Adaptors3;
use super::compute_data::Case;
use super::compute_data::ChFiKPartComputeData;
use super::compute_data::Orientations4;
use super::compute_data::SpineCurve;
use super::compute_data::not_written;
use super::compute_data::null_adaptor;
use super::compute_data::plane_case;
use super::corner_spine::corner_spine;
use super::fcts::pcurve;
use super::fil_pln_cyl::make_fillet_pln_cyl_circle;
use super::pln_pln::{make_ch_asym_pln_pln, make_chamfer_pln_pln};
use super::sphere_rotule::{make_rotule, sphere};
use crate::chfi_ds::{ChamfMethod, ChamfMode, SpineHandle, SurfDataHandle, lock};
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSDataStructure;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::Point2;

impl ChFiKPartComputeData {
    /// The chamfer cases.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn chamfer(
        dstr: &mut TopOpeBRepDSDataStructure,
        data: &SurfDataHandle,
        s: (&dyn Adaptor3dSurface, &dyn Adaptor3dSurface),
        types: (SurfaceType, SurfaceType),
        ors: (Orientation, Orientation),
        ofs: (Orientation, Orientation),
        sp: &SpineHandle,
        spine: &SpineCurve<'_>,
    ) -> Result<bool, Failure> {
        let (method, mode) = {
            let g = lock(sp);
            let cs = g.as_chamf_spine().expect("chamfer spine");
            (cs.is_chamfer(), g.mode())
        };
        let dists = match method {
            ChamfMethod::Sym => {
                let d = lock(sp)
                    .as_chamf_spine()
                    .expect("chamfer spine")
                    .get_dist()?;
                (d, d)
            }
            ChamfMethod::TwoDist => lock(sp).as_chamf_spine().expect("chamfer spine").dists()?,
            ChamfMethod::DistAngle => {
                let da = lock(sp)
                    .as_chamf_spine()
                    .expect("chamfer spine")
                    .get_dist_angle()?;
                return Self::ch_asym(dstr, data, s, types, ors, ofs, da, spine);
            }
        };
        if types == (SurfaceType::Plane, SurfaceType::Plane) {
            return make_chamfer_pln_pln(
                dstr,
                data,
                mode,
                (&s.0.plane()?, &s.1.plane()?),
                ors,
                dists,
                &spine.line()?,
                0.,
                ofs.0,
            );
        }
        let Some(c) = plane_case(s, types, ors, ofs)? else {
            return Err(not_written());
        };
        Self::chamfer_on(dstr, data, mode, &c, dists, spine)
    }

    /// The chamfer of a plane and a cylinder or a cone.
    fn chamfer_on(
        dstr: &mut TopOpeBRepDSDataStructure,
        data: &SurfDataHandle,
        mode: ChamfMode,
        c: &Case<'_>,
        dists: (f64, f64),
        spine: &SpineCurve<'_>,
    ) -> Result<bool, Failure> {
        let range = (c.other.first_u_parameter()?, c.other.last_u_parameter()?);
        if c.other.get_type()? == SurfaceType::Cylinder {
            let cyl = c.other.cylinder()?;
            let dists = if c.plandab { dists } else { (dists.1, dists.0) };
            if spine.ctyp == CurveType::Circle {
                return make_chamfer_pln_cyl_circle(
                    dstr,
                    data,
                    mode,
                    (&c.pln, &cyl),
                    range,
                    c.ors,
                    dists,
                    &spine.circle()?,
                    0.,
                    c.of,
                    c.plandab,
                );
            }
            return make_chamfer_pln_cyl_line(
                dstr,
                data,
                mode,
                (&c.pln, &cyl),
                c.ors,
                dists,
                &spine.line()?,
                0.,
                c.of,
                c.plandab,
            );
        }
        make_chamfer_pln_con(
            dstr,
            data,
            mode,
            (&c.pln, &c.other.cone()?),
            range,
            c.ors,
            dists,
            &spine.circle()?,
            0.,
            c.of,
            c.plandab,
        )
    }

    /// The distance and angle chamfer cases (the distance on the plane).
    #[allow(clippy::too_many_arguments)]
    fn ch_asym(
        dstr: &mut TopOpeBRepDSDataStructure,
        data: &SurfDataHandle,
        s: (&dyn Adaptor3dSurface, &dyn Adaptor3dSurface),
        types: (SurfaceType, SurfaceType),
        ors: (Orientation, Orientation),
        ofs: (Orientation, Orientation),
        da: (f64, f64),
        spine: &SpineCurve<'_>,
    ) -> Result<bool, Failure> {
        let dis_on_p = true;
        if types == (SurfaceType::Plane, SurfaceType::Plane) {
            return make_ch_asym_pln_pln(
                dstr,
                data,
                (&s.0.plane()?, &s.1.plane()?),
                ors,
                da,
                &spine.line()?,
                0.,
                ofs.0,
                dis_on_p,
            );
        }
        let Some(c) = plane_case(s, types, ors, ofs)? else {
            return Err(not_written());
        };
        let range = (c.other.first_u_parameter()?, c.other.last_u_parameter()?);
        if c.other.get_type()? == SurfaceType::Cylinder {
            let cyl = c.other.cylinder()?;
            if spine.ctyp == CurveType::Circle {
                return make_ch_asym_pln_cyl_circle(
                    dstr,
                    data,
                    (&c.pln, &cyl),
                    range,
                    c.ors,
                    da,
                    &spine.circle()?,
                    0.,
                    c.of,
                    c.plandab,
                    dis_on_p,
                );
            }
            return make_ch_asym_pln_cyl_line(
                dstr,
                data,
                (&c.pln, &cyl),
                c.ors,
                da,
                &spine.line()?,
                0.,
                c.of,
                c.plandab,
                dis_on_p,
            );
        }
        make_ch_asym_pln_con(
            dstr,
            data,
            (&c.pln, &c.other.cone()?),
            range,
            c.ors,
            da,
            &spine.circle()?,
            0.,
            c.of,
            c.plandab,
            dis_on_p,
        )
    }

    /// `ComputeCorner(DStr, Data, S1, S2, OrFace1, OrFace2, Or1, Or2, minRad, majRad, P1S1,
    /// P2S1, P1S2, P2S2)`: the torus fillet of a corner between a plane and a surface
    /// along the circle of the corner spine (Standard_ConstructionError when the first
    /// surface is not a plane); `ChFiKPartComputeData::compute_corner_torus(&mut ds, &data, (&s1, &s2), (of1, or1, or2), (0.5, 1.0), [p1s1, p2s1, p1s2, p2s2])?`.
    pub fn compute_corner_torus(
        dstr: &mut TopOpeBRepDSDataStructure,
        data: &SurfDataHandle,
        (s1, s2): (&Handle<dyn Adaptor3dSurface>, &Handle<dyn Adaptor3dSurface>),
        (or_face1, or1, or2): (Orientation, Orientation, Orientation),
        (min_rad, maj_rad): (f64, f64),
        [p1s1, p2s1, p1s2, p2s2]: [Point2; 4],
    ) -> Result<bool, Failure> {
        let (a1, a2) = (
            s1.get().ok_or_else(null_adaptor)?,
            s2.get().ok_or_else(null_adaptor)?,
        );
        let typ2 = a2.get_type()?;
        if a1.get_type()? != SurfaceType::Plane {
            return Err(Failure::with_message(
                "Standard_ConstructionError",
                "la face du conge torique doit etre plane",
            ));
        }
        let (mut cyl, circ, first, last) = corner_spine((s1, s2), (p1s1, p1s2, p2s2), maj_rad)?;
        let (fu, lu) = if typ2 == SurfaceType::Cylinder {
            cyl = a2.cylinder()?;
            (p1s2.x(), p2s2.x())
        } else {
            (first, last)
        };
        if !make_fillet_pln_cyl_circle(
            dstr,
            data,
            (&a1.plane()?, &cyl),
            (fu, lu),
            (or1, or2),
            min_rad,
            &circ,
            first,
            or_face1,
            true,
        )? {
            return Ok(false);
        }
        if typ2 != SurfaceType::Cylinder {
            *lock(data)
                .change_interference_on_s2()
                .change_pcurve_on_face() = pcurve(p1s2, p2s2, first, last)?.cast();
        }
        let mut d = lock(data);
        d.change_vertex_first_on_s1()
            .set_point(a1.value(p1s1.x(), p1s1.y())?);
        d.change_vertex_last_on_s1()
            .set_point(a1.value(p2s1.x(), p2s1.y())?);
        d.change_vertex_first_on_s2()
            .set_point(a2.value(p1s2.x(), p1s2.y())?);
        d.change_vertex_last_on_s2()
            .set_point(a2.value(p2s2.x(), p2s2.y())?);
        for on_s in [1, 2] {
            d.change_interference(on_s).set_first_parameter(first);
            d.change_interference(on_s).set_last_parameter(last);
        }
        Ok(true)
    }

    /// `ComputeCorner(DStr, Data, S1, S2, OrFace1, OrFace2, Or1, Or2, Rad, PS1, P1S2, P2S2)`:
    /// the sphere of a corner (`ChFiKPart_Sphere`);
    /// `ChFiKPartComputeData::compute_corner_sphere(&mut ds, &data, (&s1, &s2), (of1, or1), 1.0, (ps1, p1s2, p2s2))?`.
    pub fn compute_corner_sphere(
        dstr: &mut TopOpeBRepDSDataStructure,
        data: &SurfDataHandle,
        (s1, s2): (&Handle<dyn Adaptor3dSurface>, &Handle<dyn Adaptor3dSurface>),
        (or_face1, or1): (Orientation, Orientation),
        rad: f64,
        pts: (Point2, Point2, Point2),
    ) -> Result<bool, Failure> {
        sphere(
            dstr,
            data,
            (
                s1.get().ok_or_else(null_adaptor)?,
                s2.get().ok_or_else(null_adaptor)?,
            ),
            (or_face1, or1),
            rad,
            pts,
        )
    }

    /// `ComputeCorner(DStr, Data, S, S1, S2, OfS, OS, OS1, OS2, Radius)`: the torus joint
    /// between three planes (Standard_ConstructionError otherwise);
    /// `ChFiKPartComputeData::compute_corner_rotule(&mut ds, &data, (&s, &s1, &s2), (ofs, os, os1, os2), 1.0)?`.
    pub fn compute_corner_rotule(
        dstr: &mut TopOpeBRepDSDataStructure,
        data: &SurfDataHandle,
        (s, s1, s2): Adaptors3<'_>,
        (ofs, os, os1, os2): Orientations4,
        radius: f64,
    ) -> Result<bool, Failure> {
        let (a, a1, a2) = (
            s.get().ok_or_else(null_adaptor)?,
            s1.get().ok_or_else(null_adaptor)?,
            s2.get().ok_or_else(null_adaptor)?,
        );
        if a.get_type()? != SurfaceType::Plane
            || a1.get_type()? != SurfaceType::Plane
            || a2.get_type()? != SurfaceType::Plane
        {
            return Err(Failure::with_message(
                "Standard_ConstructionError",
                "torus joint only between the planes",
            ));
        }
        make_rotule(
            dstr,
            data,
            (&a.plane()?, &a1.plane()?, &a2.plane()?),
            (os, os1, os2),
            radius,
            ofs,
        )
    }
}
