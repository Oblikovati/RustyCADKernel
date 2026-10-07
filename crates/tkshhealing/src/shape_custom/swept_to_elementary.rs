// Rust translation of ModelingAlgorithms/TKShHealing/ShapeCustom/ShapeCustom_SweptToElementary.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeCustom_SweptToElementary`: surfaces of revolution and of linear extrusion that
//! are cylinders, cones, spheres or tori become those elementary surfaces
//! (docs/shhealing-design.md section 7).
use super::direct_modification::copied_pcurve;
use super::edge_copy::copy_edge_curve;
use super::modification::{ShapeCustomModification, ShapeCustomModificationFields};
use crate::shape_analysis::ShapeAnalysisSurface;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::{BRepToolsModification, NewCurve, NewSurface};
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::message::{Gravity, Msg};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomElementarySurface,
    GeomSphericalSurface, GeomSurface, GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution,
    GeomSweptSurface, GeomToroidalSurface,
};
use rustycad_tkg3d::geom_adaptor::{
    GeomAdaptorCurve, GeomAdaptorSurfaceOfLinearExtrusion, GeomAdaptorSurfaceOfRevolution,
};
use rustycad_tkmath::geom_abs::{Shape as Continuity, SurfaceType};
use rustycad_tkmath::gp::{Point3, Vector2, Vector3};

/// `ShapeCustom_SweptToElementary`: a face on a bare surface of revolution (or of linear
/// extrusion) whose adaptor recognizes a cylinder, a cone, a sphere or a torus (a cylinder)
/// gets that elementary surface; edges with a p-curve on a swept surface are copied, and
/// the p-curves are shifted onto the parametrization of a new torus or sphere.
///
/// ```
/// use rustycad_tkbrep::brep_tools::{BRepToolsModification, NewSurface};
/// use rustycad_tkbrep::topods::Face;
/// use rustycad_tkshhealing::shape_custom::ShapeCustomSweptToElementary;
/// let modification = ShapeCustomSweptToElementary::new();
/// assert!(modification.new_surface(&Face::default(), &mut NewSurface::default()).is_err());
/// ```
#[derive(Default)]
pub struct ShapeCustomSweptToElementary {
    parent: StandardTransient,
    messages: ShapeCustomModificationFields,
}
impl_transient!(
    ShapeCustomSweptToElementary,
    "ShapeCustom_SweptToElementary",
    dyn ShapeCustomModification,
    parent,
    interfaces: [dyn BRepToolsModification, dyn ShapeCustomModification]
);

impl ShapeCustomSweptToElementary {
    /// `ShapeCustom_SweptToElementary()`; `ShapeCustomSweptToElementary::new()`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ShapeCustomModification for ShapeCustomSweptToElementary {
    fn modification_messages(&self) -> &ShapeCustomModificationFields {
        &self.messages
    }
}

/// The static `IsToConvert(S, SS)`: true, with the surface, for a swept surface. Native
/// quirk (kept): for a trimmed or offset surface it unwraps the basis but then tests the
/// wrapper itself for Geom_SweptSurface, so a wrapped swept surface is never converted.
fn is_to_convert(
    surface: &Handle<dyn GeomSurface>,
) -> Result<(bool, Handle<dyn GeomSweptSurface>), Failure> {
    if surface.is_null() {
        // Natively `S->IsKind(...)` dereferences the null surface.
        return Err(Failure::with_type(
            "Standard_NullObject",
            "ShapeCustom_SweptToElementary: null surface; expected a Geom_Surface",
            "",
        ));
    }
    let swept = surface.cast::<dyn GeomSweptSurface>();
    Ok((!swept.is_null(), swept))
}

fn basis_adaptor(curve: &Handle<dyn GeomCurve>) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
    let adaptor =
        GeomAdaptorCurve::with_range(curve, curve.first_parameter(), curve.last_parameter())?;
    Ok(Handle::new(adaptor).cast())
}

/// The elementary surface of a surface of revolution, if its adaptor recognizes one (the
/// plane case is disabled natively: "skl 18.12.2003 - plane not used, problems in
/// PRO14665.igs").
fn revolution_elementary(
    revolution: &GeomSurfaceOfRevolution,
) -> Result<Option<Handle<dyn GeomSurface>>, Failure> {
    let curve = basis_adaptor(&revolution.basis_curve())?;
    let adaptor = GeomAdaptorSurfaceOfRevolution::with_curve_and_axis(&curve, revolution.axis())?;
    Ok(match adaptor.get_type()? {
        SurfaceType::Cylinder => {
            Some(Handle::new(GeomCylindricalSurface::from_cylinder(adaptor.cylinder()?)).cast())
        }
        SurfaceType::Sphere => {
            Some(Handle::new(GeomSphericalSurface::from_sphere(adaptor.sphere()?)).cast())
        }
        SurfaceType::Cone => {
            Some(Handle::new(GeomConicalSurface::from_cone(adaptor.cone()?)).cast())
        }
        SurfaceType::Torus => {
            Some(Handle::new(GeomToroidalSurface::from_torus(adaptor.torus()?)).cast())
        }
        _ => None,
    })
}

/// The cylinder of a surface of linear extrusion, if its adaptor recognizes one ("skl
/// 18.12.2003 - plane not used, problems in ims013.igs").
fn extrusion_elementary(
    extrusion: &GeomSurfaceOfLinearExtrusion,
) -> Result<Option<Handle<dyn GeomSurface>>, Failure> {
    let curve = basis_adaptor(&extrusion.basis_curve())?;
    let adaptor = GeomAdaptorSurfaceOfLinearExtrusion::with_curve_and_direction(
        &curve,
        extrusion.direction(),
    );
    if adaptor.get_type()? != SurfaceType::Cylinder {
        return Ok(None);
    }
    Ok(Some(
        Handle::new(GeomCylindricalSurface::from_cylinder(adaptor.cylinder()?)).cast(),
    ))
}

fn elementary_of(
    swept: &Handle<dyn GeomSweptSurface>,
) -> Result<Option<Handle<dyn GeomSurface>>, Failure> {
    if let Some(revolution) = swept.cast::<GeomSurfaceOfRevolution>().get() {
        return revolution_elementary(revolution);
    }
    if let Some(extrusion) = swept.cast::<GeomSurfaceOfLinearExtrusion>().get() {
        return extrusion_elementary(extrusion);
    }
    // Native keeps S unchanged and still reports success for another swept surface.
    Ok(Some(swept.clone().cast()))
}

/// The torus shift: the new p-curve parameters of the revolution origin S(U1, V1).
fn torus_shift(
    revolution: &GeomSurfaceOfRevolution,
    new_surface: &Handle<dyn GeomSurface>,
) -> Result<Vector2, Failure> {
    let (u1, _, v1, _) = revolution.bounds();
    let origin = revolution.value(u1, v1)?;
    let analysis = ShapeAnalysisSurface::new(new_surface)?;
    let uv = analysis.value_of_uv(origin, precision::confusion())?;
    Ok(Vector2::new(uv.x() - u1, uv.y() - v1))
}

/// The sphere shift in V: the angle between the sphere and revolution origins seen from
/// the sphere centre, corrected by the first V parameters.
fn sphere_shift(
    revolution: &GeomSurfaceOfRevolution,
    sphere: &GeomSphericalSurface,
) -> Result<Vector2, Failure> {
    let (us1, _, vs1, _) = sphere.bounds();
    let on_sphere = sphere.value(us1, vs1)?;
    let (ur1, _, vr1, _) = revolution.bounds();
    let on_revolution = revolution.value(ur1, vr1)?;
    let centre = sphere.location();
    let angle =
        Vector3::between(centre, on_sphere).angle(Vector3::between(centre, on_revolution))?;
    Ok(Vector2::new(0., vs1 - vr1 + angle))
}

fn shifted_onto_new_surface(
    curve: &Handle<dyn Geom2dCurve>,
    swept: &Handle<dyn GeomSweptSurface>,
    new_surface: &Handle<dyn GeomSurface>,
) -> Result<(), Failure> {
    if new_surface.is_null() {
        return Ok(());
    }
    let to_torus = !new_surface.cast::<GeomToroidalSurface>().is_null();
    let sphere = new_surface.cast::<GeomSphericalSurface>();
    if !to_torus && sphere.is_null() {
        return Ok(());
    }
    // Natively `SS->IsKind(...)` dereferences a null swept surface here.
    if swept.is_null() {
        return Err(null_swept());
    }
    let revolution = swept.cast::<GeomSurfaceOfRevolution>();
    let Some(revolution) = revolution.get() else {
        return Ok(());
    };
    if to_torus {
        curve.translate(torus_shift(revolution, new_surface)?)?;
    }
    if let Some(sphere) = sphere.get() {
        curve.translate(sphere_shift(revolution, sphere)?)?;
    }
    Ok(())
}

fn null_swept() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ShapeCustom_SweptToElementary::NewCurve2d: the face surface is not swept while the new \
         face is a torus or a sphere; native dereferences the null swept surface",
        "",
    )
}

impl BRepToolsModification for ShapeCustomSweptToElementary {
    /// The recognized elementary surface, the face tolerance, no reversal and the message
    /// `SweptToElementary.NewSurface.MSG0`; false (the face surface and location written)
    /// otherwise.
    fn new_surface(&self, face: &Face, result: &mut NewSurface) -> Result<bool, Failure> {
        let (surface, location) = BRepTool::surface_with_location(face)?;
        (result.surface, result.location) = (surface, location);
        let (convert, swept) = is_to_convert(&result.surface)?;
        if !convert {
            return Ok(false);
        }
        let Some(elementary) = elementary_of(&swept)? else {
            return Ok(false);
        };
        result.surface = elementary;
        let key = AsciiString::from_bytes(b"SweptToElementary.NewSurface.MSG0");
        self.send_msg(
            &face.clone().into_shape(),
            &Msg::from_key(&key),
            Gravity::Info,
        );
        result.tolerance = BRepTool::face_tolerance(face)?;
        result.rev_wires = false;
        result.rev_face = false;
        Ok(true)
    }

    /// The copied 3D curve of an edge with a p-curve on a swept surface.
    fn new_curve(&self, edge: &Edge, result: &mut NewCurve) -> Result<bool, Failure> {
        copy_edge_curve(
            edge,
            result,
            "ShapeCustom_SweptToElementary::NewCurve",
            |surface, _location| Ok(is_to_convert(surface)?.0),
        )
    }

    /// 3D points are never modified.
    fn new_point(
        &self,
        _vertex: &Vertex,
        _point: &mut Point3,
        _tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        Ok(false)
    }

    /// A copy of the p-curve when the surface is swept or the edge was copied, shifted
    /// onto a new torus or sphere made from a surface of revolution, with the edge
    /// tolerance; false otherwise.
    fn new_curve2d(
        &self,
        edge: &Edge,
        face: &Face,
        new_edge: &Edge,
        new_face: &Face,
        curve: &mut Handle<dyn Geom2dCurve>,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        let (surface, _) = BRepTool::surface_with_location(face)?;
        let (convert, swept) = is_to_convert(&surface)?;
        if !convert && edge.is_same(new_edge) {
            return Ok(false);
        }
        *curve = copied_pcurve(&BRepTool::curve_on_face(edge, face)?.pcurve)?;
        if !curve.is_null() {
            let (new_surface, _) = BRepTool::surface_with_location(new_face)?;
            shifted_onto_new_surface(curve, &swept, &new_surface)?;
        }
        *tolerance = BRepTool::edge_tolerance(edge)?;
        Ok(true)
    }

    fn new_parameter(
        &self,
        _vertex: &Vertex,
        _edge: &Edge,
        _parameter: &mut f64,
        _tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        Ok(false)
    }

    /// The continuity of the edge between the original faces.
    fn continuity(
        &self,
        edge: &Edge,
        face1: &Face,
        face2: &Face,
        _new_edge: &Edge,
        _new_face1: &Face,
        _new_face2: &Face,
    ) -> Result<Continuity, Failure> {
        BRepTool::continuity_on_faces(edge, face1, face2)
    }
}
