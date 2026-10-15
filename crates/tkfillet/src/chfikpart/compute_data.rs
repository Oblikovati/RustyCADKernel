// Rust translation of ModelingAlgorithms/TKFillet/ChFiKPart/ChFiKPart_ComputeData.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ChFiKPart_ComputeData`: the fillets and chamfers of the particular cases (planes,
//! cylinders and cones) dispatched to the `Make*` functions, and the corners.
use super::fil_pln_con::make_fillet_pln_con;
use super::fil_pln_cyl::{make_fillet_pln_cyl_circle, make_fillet_pln_cyl_line};
use super::pln_pln::make_fillet_pln_pln;
use crate::chfi_ds::{SpineHandle, SurfDataHandle, lock};
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSDataStructure;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Circle3, Line3};

/// The particular cases of the fillets and chamfers (`ChFiKPart_ComputeData`, static
/// members); `ChFiKPartComputeData::compute(&mut ds, &data, &s1, &s2, or1, or2, &spine, 1)?`.
pub struct ChFiKPartComputeData;

/// The spine of the particular case: the handle and the type of the edge curve; its
/// line or circle is read when a case asks for it (`ChFiDS_Spine::Line`, `Circle`).
pub(super) struct SpineCurve<'a> {
    pub(super) sp: &'a SpineHandle,
    pub(super) ctyp: CurveType,
}

impl SpineCurve<'_> {
    pub(super) fn line(&self) -> Result<Line3, Failure> {
        lock(self.sp).line()
    }

    pub(super) fn circle(&self) -> Result<Circle3, Failure> {
        lock(self.sp).circle()
    }
}

/// The surfaces of a case (plane first) with their bounds and orientations.
pub(super) struct Case<'a> {
    pub(super) pln: rustycad_tkmath::gp::Plane3,
    pub(super) other: &'a dyn Adaptor3dSurface,
    pub(super) ors: (Orientation, Orientation),
    pub(super) of: Orientation,
    pub(super) plandab: bool,
}

/// Standard_NotImplemented for the cases without closed form.
pub(super) fn not_written() -> Failure {
    Failure::with_message("Standard_NotImplemented", "particular case not written")
}

/// Standard_NullObject for a missing adaptor.
pub(super) fn null_adaptor() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ChFiKPart_ComputeData: null surface adaptor",
    )
}

/// The orientation of the face of a BRep adaptor (FORWARD for another adaptor).
fn face_orientation(s: &Handle<dyn Adaptor3dSurface>) -> Orientation {
    s.cast::<BRepAdaptorSurface>()
        .get()
        .map_or(Orientation::Forward, |hs| hs.face().orientation())
}

/// The plane-first case of two surfaces (None when neither is a plane with the other a
/// plane, a cylinder or a cone).
pub(super) fn plane_case<'a>(
    s: (&'a dyn Adaptor3dSurface, &'a dyn Adaptor3dSurface),
    (t1, t2): (SurfaceType, SurfaceType),
    ors: (Orientation, Orientation),
    ofs: (Orientation, Orientation),
) -> Result<Option<Case<'a>>, Failure> {
    if t1 == SurfaceType::Plane && matches!(t2, SurfaceType::Cylinder | SurfaceType::Cone) {
        return Ok(Some(Case {
            pln: s.0.plane()?,
            other: s.1,
            ors,
            of: ofs.0,
            plandab: true,
        }));
    }
    if t2 == SurfaceType::Plane && matches!(t1, SurfaceType::Cylinder | SurfaceType::Cone) {
        return Ok(Some(Case {
            pln: s.1.plane()?,
            other: s.0,
            ors: (ors.1, ors.0),
            of: ofs.1,
            plandab: false,
        }));
    }
    Ok(None)
}

/// The spine and the type of the curve of its edge.
fn spine_curve(sp: &SpineHandle, iedge: i32) -> Result<SpineCurve<'_>, Failure> {
    let ctyp = lock(sp)
        .spine_mut()
        .current_elementary_spine(iedge)?
        .get_type()?;
    Ok(SpineCurve { sp, ctyp })
}

/// Three surface adaptors of a corner.
pub(super) type Adaptors3<'a> = (
    &'a Handle<dyn Adaptor3dSurface>,
    &'a Handle<dyn Adaptor3dSurface>,
    &'a Handle<dyn Adaptor3dSurface>,
);
/// The orientations of the corner surfaces.
pub(super) type Orientations4 = (Orientation, Orientation, Orientation, Orientation);

impl ChFiKPartComputeData {
    /// `Compute(DStr, Data, S1, S2, Or1, Or2, Sp, Iedge)`: the fillet (radius of the edge)
    /// or the chamfer (symmetric, two distances or distance and angle) of a plane with a
    /// plane, a cylinder or a cone along the edge; Standard_NotImplemented for the other
    /// surfaces; `ChFiKPartComputeData::compute(&mut ds, &data, &s1, &s2, or1, or2, &spine, 1)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn compute(
        dstr: &mut TopOpeBRepDSDataStructure,
        data: &SurfDataHandle,
        s1: &Handle<dyn Adaptor3dSurface>,
        s2: &Handle<dyn Adaptor3dSurface>,
        or1: Orientation,
        or2: Orientation,
        sp: &SpineHandle,
        iedge: i32,
    ) -> Result<bool, Failure> {
        let (a1, a2) = (
            s1.get().ok_or_else(null_adaptor)?,
            s2.get().ok_or_else(null_adaptor)?,
        );
        let types = (a1.get_type()?, a2.get_type()?);
        let spine = spine_curve(sp, iedge)?;
        let ofs = (face_orientation(s1), face_orientation(s2));
        let is_fil = lock(sp).as_fil_spine().is_some();
        if is_fil {
            let radius = lock(sp)
                .as_fil_spine()
                .expect("fillet spine")
                .radius_on_edge_index(iedge)?;
            return Self::fillet(dstr, data, (a1, a2), types, (or1, or2), ofs, radius, &spine);
        }
        if lock(sp).as_chamf_spine().is_none() {
            return Ok(false);
        }
        Self::chamfer(dstr, data, (a1, a2), types, (or1, or2), ofs, sp, &spine)
    }

    /// The fillet cases.
    #[allow(clippy::too_many_arguments)]
    fn fillet(
        dstr: &mut TopOpeBRepDSDataStructure,
        data: &SurfDataHandle,
        s: (&dyn Adaptor3dSurface, &dyn Adaptor3dSurface),
        types: (SurfaceType, SurfaceType),
        ors: (Orientation, Orientation),
        ofs: (Orientation, Orientation),
        radius: f64,
        spine: &SpineCurve<'_>,
    ) -> Result<bool, Failure> {
        if types == (SurfaceType::Plane, SurfaceType::Plane) {
            return make_fillet_pln_pln(
                dstr,
                data,
                (&s.0.plane()?, &s.1.plane()?),
                ors,
                radius,
                &spine.line()?,
                0.,
                ofs.0,
            );
        }
        let Some(c) = plane_case(s, types, ors, ofs)? else {
            return Err(not_written());
        };
        let range = (c.other.first_u_parameter()?, c.other.last_u_parameter()?);
        if c.other.get_type()? == SurfaceType::Cylinder {
            let cyl = c.other.cylinder()?;
            if spine.ctyp == CurveType::Line {
                return make_fillet_pln_cyl_line(
                    dstr,
                    data,
                    (&c.pln, &cyl),
                    range,
                    c.ors,
                    radius,
                    &spine.line()?,
                    0.,
                    c.of,
                    c.plandab,
                );
            }
            return make_fillet_pln_cyl_circle(
                dstr,
                data,
                (&c.pln, &cyl),
                range,
                c.ors,
                radius,
                &spine.circle()?,
                0.,
                c.of,
                c.plandab,
            );
        }
        make_fillet_pln_con(
            dstr,
            data,
            (&c.pln, &c.other.cone()?),
            range,
            c.ors,
            radius,
            &spine.circle()?,
            0.,
            c.of,
            c.plandab,
        )
    }
}
