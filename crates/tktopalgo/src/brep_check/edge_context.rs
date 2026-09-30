// Rust translation of BRepCheck_Edge::InContext (ModelingAlgorithms/TKTopAlgo/BRepCheck/BRepCheck_Edge.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepCheck_Edge::InContext`: the edge in a face (its curves on the face surface
//! against its reference curve) and in a solid (shared by two faces).
use super::edge::{BRepCheckEdge, pcurve_range_fits};
use super::errors::null_object;
use super::explore::{holds_same, sub_shapes};
use super::result::{BRepCheckResult, SharedStatusList, add_status, close_status_list};
use super::status::BRepCheckStatus;
use crate::brep_lib::BRepLibValidateEdge;
use rustycad_tkbrep::brep::access::{edge_tshape, face_tshape};
use rustycad_tkbrep::brep::{BRepCurveRepresentation, BRepGCurve, BRepTool};
use rustycad_tkbrep::topods::{Face, Shape, face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::{Geom2dAdaptorCurve, make_curve};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomCurve, GeomElementarySurface, GeomGeometry, GeomPlane, GeomRectangularTrimmedSurface,
    GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::geom_proj_lib;
use rustycad_tkgeombase::proj_lib::ProjectedCurve;
use rustycad_tkmath::gp::Transform3;
use rustycad_tkmath::toploc::Location;

/// The frame of the face checks: the reference curve, the face, the list and the
/// tolerance and flags the validations use.
struct FaceCheck<'a> {
    list: &'a SharedStatusList,
    hcurve: Handle<dyn Adaptor3dCurve>,
    cref: Handle<dyn BRepCurveRepresentation>,
    /// `Floc * TFloc`: the face location composed with its surface location.
    face_location: Location,
    surface: Handle<dyn GeomSurface>,
    tol: f64,
    same_parameter: bool,
    gctrl: bool,
    exact: bool,
    parallel: bool,
}

impl FaceCheck<'_> {
    /// One `BRepLib_ValidateEdge` of the reference curve against `other`, with the
    /// edge tolerance as limit; true when it is done and within the tolerance.
    fn validate(&self, other: &Handle<Adaptor3dCurveOnSurface>) -> Result<bool, Failure> {
        let mut validate = BRepLibValidateEdge::new(&self.hcurve, other, self.same_parameter);
        validate.set_exit_if_tolerance_exceeded(self.tol)?;
        validate.set_exact_method(self.exact);
        validate.set_parallel(self.parallel);
        validate.process()?;
        Ok(validate.is_done() && validate.check_tolerance(self.tol)?)
    }

    /// The face surface moved by the face location (eap occ332).
    fn moved_surface(&self) -> Result<Handle<dyn GeomSurface>, Failure> {
        let transform: Transform3 = self.face_location.transformation();
        Ok(self
            .surface
            .get()
            .ok_or_else(|| null_object("BRepCheck_Edge::InContext: face without surface"))?
            .transformed(transform)?
            .cast::<dyn GeomSurface>())
    }

    /// The checks of one curve on the face surface (not the reference): the same range
    /// as the reference, the range within the pcurve, the distances. False when the
    /// check must leave InContext (an invalid range).
    fn check_pcurve(
        &self,
        cr: &Handle<dyn BRepCurveRepresentation>,
        etrsf: Transform3,
    ) -> Result<bool, Failure> {
        let r = cr.get().expect("checked representation");
        let gcurve = cr.cast::<dyn BRepGCurve>();
        let (f, l) = gcurve
            .get()
            .ok_or_else(|| null_object("BRepCheck_Edge::InContext: not a BRep_GCurve"))?
            .range();
        let (first, last) = hcurve_range(&self.hcurve)?;
        let (ff, ll) = self.reference_parameters((f, l), etrsf)?;
        let eps: f64 = precision::p_confusion();
        if (ff - first).abs() > eps || (ll - last).abs() > eps {
            add_status(self.list, BRepCheckStatus::InvalidSameRangeFlag);
            add_status(self.list, BRepCheckStatus::InvalidSameParameterFlag);
        }
        if !pcurve_range_fits(&r.pcurve()?, (f, l))? {
            add_status(self.list, BRepCheckStatus::InvalidRange);
            return Ok(false);
        }
        if self.gctrl {
            self.check_distances(r, (f, l))?;
        }
        Ok(true)
    }

    /// `(f, l)` through the reference 3d curve's parameter transformation.
    fn reference_parameters(
        &self,
        (f, l): (f64, f64),
        etrsf: Transform3,
    ) -> Result<(f64, f64), Failure> {
        let Some(r) = self.cref.get().filter(|r| r.is_curve3d()) else {
            return Ok((f, l));
        };
        let c3d = r.curve3d()?;
        let curve = c3d
            .get()
            .ok_or_else(|| null_object("BRepCheck_Edge::InContext: null 3d curve"))?;
        Ok((
            curve.transformed_parameter(f, etrsf),
            curve.transformed_parameter(l, etrsf),
        ))
    }

    /// The distances of the pcurve (and of the second one of a closed surface) to the
    /// reference curve.
    fn check_distances(
        &self,
        r: &dyn BRepCurveRepresentation,
        range: (f64, f64),
    ) -> Result<(), Failure> {
        let sb = self.moved_surface()?;
        let gahs: Handle<dyn Adaptor3dSurface> =
            Handle::new(GeomAdaptorSurface::with_surface(&sb)?).cast();
        let ghpc = Handle::new(Geom2dAdaptorCurve::with_range(
            &r.pcurve()?,
            range.0,
            range.1,
        )?);
        let ghpc_curve: Handle<dyn Adaptor2dCurve2d> = ghpc.clone().cast();
        let acs = Handle::new(Adaptor3dCurveOnSurface::with_curve(&ghpc_curve, &gahs)?);
        let closed: bool = r.is_curve_on_closed_surface();
        if !self.validate(&acs)? {
            let status = if closed {
                BRepCheckStatus::InvalidCurveOnClosedSurface
            } else {
                BRepCheckStatus::InvalidCurveOnSurface
            };
            add_status(self.list, status);
            add_status(self.list, BRepCheckStatus::InvalidSameParameterFlag);
        }
        if !closed {
            return Ok(());
        }
        // The same adaptors are reloaded with the second pcurve, on the same bounds.
        ghpc.get()
            .expect("fresh adaptor")
            .load_range(&r.pcurve2()?, range.0, range.1)?;
        acs.get().expect("fresh adaptor").load(&ghpc_curve, &gahs)?;
        if !self.validate(&acs)? {
            add_status(self.list, BRepCheckStatus::InvalidCurveOnClosedSurface);
            if self.same_parameter {
                add_status(self.list, BRepCheckStatus::InvalidSameParameterFlag);
            }
        }
        Ok(())
    }

    /// No curve on the face surface: `NoCurveOnSurface` unless the surface is a plane,
    /// where the reference curve is projected on the fly (as BRep_Tool does) and checked.
    fn check_projection(&self) -> Result<(), Failure> {
        let plane = plane_of(&self.surface);
        if plane.is_null() {
            add_status(self.list, BRepCheckStatus::NoCurveOnSurface);
            return Ok(());
        }
        if !self.gctrl {
            return Ok(());
        }
        let moved = plane
            .get()
            .expect("checked plane")
            .transformed(self.face_location.transformation())?
            .cast::<GeomPlane>();
        let acs = self.projected_curve(&moved)?;
        if !self.validate(&acs)? {
            add_status(self.list, BRepCheckStatus::InvalidCurveOnSurface);
        }
        Ok(())
    }

    /// The reference 3d curve projected on `plane`, as a curve on the plane.
    fn projected_curve(
        &self,
        plane: &Handle<GeomPlane>,
    ) -> Result<Handle<Adaptor3dCurveOnSurface>, Failure> {
        let gahs: Handle<dyn Adaptor3dSurface> =
            Handle::new(GeomAdaptorSurface::with_surface(&plane.clone().cast())?).cast();
        // "Normalement myHCurve est une GeomAdaptor_Curve": a reference curve on surface
        // is a null dereference natively.
        let gac = self.hcurve.cast::<GeomAdaptorCurve>();
        let c3d = gac
            .get()
            .ok_or_else(|| {
                null_object("BRepCheck_Edge::InContext: myHCurve is not a GeomAdaptor_Curve")
            })?
            .curve();
        let (first, last) = hcurve_range(&self.hcurve)?;
        let trimmed: Handle<dyn GeomCurve> =
            Handle::new(GeomTrimmedCurve::new(&c3d, first, last, true, true)?).cast();
        let direction = plane.get().expect("moved plane").position().direction();
        let on_plane = geom_proj_lib::project_on_plane(&trimmed, plane, direction, true)?;
        let curve: Handle<dyn Adaptor3dCurve> =
            Handle::new(GeomAdaptorCurve::with_curve(&on_plane)?).cast();
        let projection = ProjectedCurve::with_curve(&gahs, &curve)?;
        let pc = make_curve(&projection)?;
        let ghpc: Handle<dyn Adaptor2dCurve2d> =
            Handle::new(Geom2dAdaptorCurve::with_range(&pc, first, last)?).cast();
        Ok(Handle::new(Adaptor3dCurveOnSurface::with_curve(
            &ghpc, &gahs,
        )?))
    }
}

/// The range of the reference adaptor.
fn hcurve_range(hcurve: &Handle<dyn Adaptor3dCurve>) -> Result<(f64, f64), Failure> {
    let curve = hcurve
        .get()
        .ok_or_else(|| null_object("BRepCheck_Edge: no reference curve adaptor"))?;
    Ok((curve.first_parameter()?, curve.last_parameter()?))
}

/// The plane of a surface or of its rectangular trimmed basis, null for other surfaces.
fn plane_of(surface: &Handle<dyn GeomSurface>) -> Handle<GeomPlane> {
    match surface.cast::<GeomRectangularTrimmedSurface>().get() {
        Some(trimmed) => trimmed.basis_surface().cast::<GeomPlane>(),
        None => surface.cast::<GeomPlane>(),
    }
}

impl BRepCheckEdge {
    /// `InContext(S)`: `SubshapeNotInShape` when the edge is not in `context`; in a face,
    /// the flags and the curves on the face surface; in a solid, free and
    /// multi-connected edges.
    pub(super) fn check_in_context(&self, context: &Shape) -> Result<(), Failure> {
        let Some(list) = self.result_base().bind_context_list(context) else {
            return Ok(());
        };
        let shape = self.edge();
        let tol: f64 = BRepTool::edge_tolerance(&shape)?;
        if !holds_same(context, ShapeEnum::Edge, &shape)? {
            add_status(&list, BRepCheckStatus::SubshapeNotInShape);
            return Ok(());
        }
        match context.shape_type()? {
            ShapeEnum::Face if self.fields.read().cref.is_some() => {
                if !self.check_in_face(&face(context)?, &list, tol)? {
                    return Ok(());
                }
            }
            ShapeEnum::Solid => self.check_in_solid(context, &list)?,
            _ => {}
        }
        close_status_list(&list);
        Ok(())
    }

    /// The face checks; false when InContext leaves early (the flags or a range).
    fn check_in_face(
        &self,
        context: &Face,
        list: &SharedStatusList,
        tol: f64,
    ) -> Result<bool, Failure> {
        let shape = self.edge();
        let tedge = edge_tshape(&shape, "BRepCheck_Edge::InContext")?;
        let (same_parameter, same_range) = (tedge.same_parameter(), tedge.same_range());
        if !same_parameter || !same_range {
            if !same_parameter {
                add_status(list, BRepCheckStatus::InvalidSameParameterFlag);
            }
            if !same_range {
                add_status(list, BRepCheckStatus::InvalidSameRangeFlag);
            }
            return Ok(false);
        }
        let tface = face_tshape(context, "BRepCheck_Edge::InContext")?;
        let face_location = (context.location() * &tface.location())?;
        let location = face_location.predivided(shape.location())?;
        let (cref, hcurve, gctrl, exact) = {
            let fields = self.fields.read();
            (
                fields.cref.clone(),
                fields.hcurve.clone(),
                fields.gctrl,
                fields.is_exact_method,
            )
        };
        let cref_location = cref.get().expect("checked reference").location();
        let etrsf = (shape.location() * &cref_location)?.transformation();
        let check = FaceCheck {
            list,
            hcurve,
            cref: cref.clone(),
            face_location,
            surface: tface.surface(),
            tol,
            same_parameter,
            gctrl,
            exact,
            parallel: self.result_base().is_parallel(),
        };
        let mut pcurve_found: bool = false;
        for cr in tedge.curves() {
            let Some(r) = cr.get() else {
                continue;
            };
            if cr.object_address() == cref.object_address()
                || !r.is_curve_on_surface_at(&check.surface, &location)
            {
                continue;
            }
            pcurve_found = true;
            if !check.check_pcurve(&cr, etrsf)? {
                return Ok(false);
            }
        }
        if !pcurve_found {
            check.check_projection()?;
        }
        Ok(true)
    }

    /// The solid check: the edge must be shared by two faces of `context` (by two
    /// occurrences at most), unless it is degenerated.
    fn check_in_solid(&self, context: &Shape, list: &SharedStatusList) -> Result<(), Failure> {
        let shape = self.edge();
        let mut connections: i32 = 0;
        for face in sub_shapes(context, ShapeEnum::Face)? {
            for edge in sub_shapes(&face, ShapeEnum::Edge)? {
                if edge.is_same(&shape) {
                    connections += 1;
                }
            }
        }
        let degenerated = edge_tshape(&shape, "BRepCheck_Edge::InContext")?.degenerated();
        let status = if connections < 2 && !degenerated {
            BRepCheckStatus::FreeEdge
        } else if connections > 2 {
            BRepCheckStatus::InvalidMultiConnexity
        } else {
            BRepCheckStatus::NoError
        };
        add_status(list, status);
        Ok(())
    }
}
