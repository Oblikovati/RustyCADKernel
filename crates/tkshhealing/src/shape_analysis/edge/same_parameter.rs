// Rust translation of the two CheckSameParameter members of
// ModelingAlgorithms/TKShHealing/ShapeAnalysis/ShapeAnalysis_Edge.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.
//
// :s4 abv 26.04.99: sim6049.igs 21677: a copy of the curve is necessary to get a true
// SameParameter check.

//! The deviation between the 3D curve and the p-curves of an edge.
use super::ShapeAnalysisEdge;
use crate::shape_extend::ShapeExtendStatus;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_lib::BRepLibValidateEdge;

/// The 3D curve of the check and the flags it compares with.
struct SameParameterCheck {
    curve: Handle<dyn Adaptor3dCurve>,
    same_parameter: bool,
    nb_control: i32,
    range: (f64, f64),
    tolerance: f64,
}

impl SameParameterCheck {
    /// One `BRepLib_ValidateEdge` of the 3D curve against the p-curve on the surface
    /// (moved by its location): widens `maxdev`, false when the validation failed.
    fn validate(
        &self,
        pcurve: &Handle<dyn Geom2dCurve>,
        range: (f64, f64),
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
        maxdev: &mut f64,
    ) -> Result<bool, Failure> {
        let moved = located_surface(surface, location)?;
        let hpc: Handle<dyn Adaptor2dCurve2d> =
            Handle::new(Geom2dAdaptorCurve::with_range(pcurve, range.0, range.1)?).cast();
        let hs: Handle<dyn Adaptor3dSurface> =
            Handle::new(GeomAdaptorSurface::with_surface(&moved)?).cast();
        let acs = Handle::new(Adaptor3dCurveOnSurface::with_curve(&hpc, &hs)?);
        let mut validate = BRepLibValidateEdge::new(&self.curve, &acs, self.same_parameter);
        validate.set_control_points_number(self.nb_control - 1);
        validate.process()?;
        validate.update_tolerance(maxdev);
        Ok(validate.is_done())
    }
}

impl ShapeAnalysisEdge {
    /// `CheckSameParameter(edge, maxdev, NbControl)`: [`Self::check_same_parameter_on_face`]
    /// against every p-curve; `sae.check_same_parameter(&edge, 23)?`.
    pub fn check_same_parameter(
        &mut self,
        edge: &Edge,
        nb_control: i32,
    ) -> Result<(bool, f64), Failure> {
        self.check_same_parameter_on_face(edge, &Face::default(), nb_control)
    }

    /// `CheckSameParameter(theEdge, theFace, theMaxdev, theNbControl)`: the largest
    /// deviation between the 3D curve and each p-curve (those on the face only, or the
    /// projection on a planar face without p-curve) at `nb_control` points, as
    /// `BRepCheck` measures it: DONE1 when above the edge tolerance, DONE2 without the
    /// SameParameter flag, FAIL1 without 3D curve, FAIL2 when a validation failed; true
    /// for DONE. A degenerated edge gives false with the deviation untouched natively (0
    /// here); `sae.check_same_parameter_on_face(&edge, &face, 23)?`.
    pub fn check_same_parameter_on_face(
        &mut self,
        edge: &Edge,
        face: &Face,
        nb_control: i32,
    ) -> Result<(bool, f64), Failure> {
        self.set_status(ShapeExtendStatus::Ok);
        if BRepTool::degenerated(edge)? {
            return Ok((false, 0.0));
        }
        let Some(check) = SameParameterCheck::for_edge(edge, nb_control)? else {
            self.add_status(ShapeExtendStatus::Fail1);
            return Ok((false, 0.0));
        };
        let mut deviation = 0.0;
        self.check_face_representations(edge, face, &check, &mut deviation)?;
        if deviation > check.tolerance {
            self.add_status(ShapeExtendStatus::Done1);
        }
        if !check.same_parameter {
            self.add_status(ShapeExtendStatus::Done2);
        }
        Ok((self.status_flag(ShapeExtendStatus::Done), deviation))
    }

    /// The loop over the curves on surfaces of the edge (those on the face when it is
    /// given); true when one was checked.
    fn check_stored_pcurves(
        &mut self,
        edge: &Edge,
        check: &SameParameterCheck,
        (face_surface, face_location): (&Handle<dyn GeomSurface>, &Location),
        maxdev: &mut f64,
    ) -> Result<bool, Failure> {
        let mut found = false;
        let mut index = 1;
        loop {
            let curve = BRepTool::curve_on_surface_by_index(edge, index)?;
            if curve.pcurve.is_null() {
                return Ok(found);
            }
            index += 1;
            if !on_selected_surface(&curve, face_surface, face_location) {
                continue;
            }
            found = true;
            self.validate_stored_pcurve(check, &curve, maxdev)?;
        }
    }
}

/// The 3D curve moved by its location (a transformed copy), with its range transformed
/// by the moved curve (`TransformedParameter`); a null curve stays null.
pub(super) fn moved_curve(
    curve: &Handle<dyn GeomCurve>,
    location: &Location,
    first: f64,
    last: f64,
) -> Result<(Handle<dyn GeomCurve>, f64, f64), Failure> {
    if curve.is_null() || location.is_identity() {
        return Ok((curve.clone(), first, last));
    }
    let trsf = location.transformation();
    let moved = curve.transformed(trsf)?.cast::<dyn GeomCurve>();
    let Some(c) = moved.get() else {
        return Ok((moved.clone(), first, last));
    };
    let (f, l) = (
        c.transformed_parameter(first, trsf),
        c.transformed_parameter(last, trsf),
    );
    Ok((moved, f, l))
}

impl SameParameterCheck {
    fn for_edge(edge: &Edge, nb_control: i32) -> Result<Option<Self>, Failure> {
        let tedge = edge_tshape(edge, "ShapeAnalysis_Edge::CheckSameParameter")?;
        let same_parameter = tedge.same_parameter();
        let found = BRepTool::curve_with_location(edge)?;
        if found.curve.is_null() {
            return Ok(None);
        }
        let (curve, first, last) =
            moved_curve(&found.curve, &found.location, found.first, found.last)?;
        let curve = Handle::new(GeomAdaptorCurve::with_range(&curve, first, last)?).cast();
        Ok(Some(Self {
            curve,
            same_parameter,
            nb_control,
            range: (first, last),
            tolerance: tedge.tolerance(),
        }))
    }
}

impl ShapeAnalysisEdge {
    fn check_face_representations(
        &mut self,
        edge: &Edge,
        face: &Face,
        check: &SameParameterCheck,
        deviation: &mut f64,
    ) -> Result<(), Failure> {
        let (surface, location) = if face.is_null() {
            (Handle::default(), Location::default())
        } else {
            BRepTool::surface_with_location(face)?
        };
        let found = self.check_stored_pcurves(edge, check, (&surface, &location), deviation)?;
        if found || surface.is_null() {
            return Ok(());
        }
        let (pcurve, _, _) = BRepTool::curve_on_plane(edge, &surface, &location)?;
        // The projection is checked on the range of the 3D curve.
        if !pcurve.is_null()
            && !check.validate(&pcurve, check.range, &surface, &location, deviation)?
        {
            self.add_status(ShapeExtendStatus::Fail2);
        }
        Ok(())
    }

    fn validate_stored_pcurve(
        &mut self,
        check: &SameParameterCheck,
        curve: &rustycad_tkbrep::brep::EdgeCurveOnSurface,
        deviation: &mut f64,
    ) -> Result<(), Failure> {
        let range = (curve.first, curve.last);
        if !check.validate(
            &curve.pcurve,
            range,
            &curve.surface,
            &curve.location,
            deviation,
        )? {
            self.add_status(ShapeExtendStatus::Fail2);
        }
        Ok(())
    }
}

fn on_selected_surface(
    curve: &rustycad_tkbrep::brep::EdgeCurveOnSurface,
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
) -> bool {
    surface.is_null() || (*surface == curve.surface && *location == curve.location)
}

fn located_surface(
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let original = surface.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "ShapeAnalysis_Edge::CheckSameParameter: null surface; expected a Geom_Surface",
        )
    })?;
    let moved = original
        .transformed(location.transformation())?
        .cast::<dyn GeomSurface>();
    Ok(moved)
}
