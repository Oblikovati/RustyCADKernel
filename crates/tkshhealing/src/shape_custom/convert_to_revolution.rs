// Rust translation of ModelingAlgorithms/TKShHealing/ShapeCustom/ShapeCustom_ConvertToRevolution.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeCustom_ConvertToRevolution`: spheres, tori, cylinders and cones as surfaces of
//! revolution (docs/shhealing-design.md section 7).
use super::direct_modification::copied_pcurve;
use super::edge_copy::copy_edge_curve;
use super::modification::{ShapeCustomModification, ShapeCustomModificationFields};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::{BRepToolsModification, NewCurve, NewSurface};
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::message::{Gravity, Msg};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{
    GeomCircle, GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomElementarySurface,
    GeomLine, GeomOffsetSurface, GeomRectangularTrimmedSurface, GeomSphericalSurface, GeomSurface,
    GeomSurfaceOfRevolution, GeomToroidalSurface, GeomTrimmedCurve,
};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Axis1, Axis2, Axis3, Direction3, Point3, Vector2};
use std::f64::consts::PI;

/// `ShapeCustom_ConvertToRevolution`: a face on a sphere, a torus, a cylinder or a cone
/// (possibly trimmed or offset) gets the equivalent `Geom_SurfaceOfRevolution` of a
/// meridian about the surface axis, with the trimming or offset restored; edges with a
/// p-curve on such a surface are copied (the p-curves of a sphere moved by 2*PI in V).
///
/// ```
/// use rustycad_tkbrep::brep_tools::{BRepToolsModification, NewSurface};
/// use rustycad_tkbrep::topods::Face;
/// use rustycad_tkshhealing::shape_custom::ShapeCustomConvertToRevolution;
/// let modification = ShapeCustomConvertToRevolution::new();
/// assert!(modification.new_surface(&Face::default(), &mut NewSurface::default()).is_err());
/// ```
#[derive(Default)]
pub struct ShapeCustomConvertToRevolution {
    parent: StandardTransient,
    messages: ShapeCustomModificationFields,
}
impl_transient!(
    ShapeCustomConvertToRevolution,
    "ShapeCustom_ConvertToRevolution",
    dyn ShapeCustomModification,
    parent,
    interfaces: [dyn BRepToolsModification, dyn ShapeCustomModification]
);

impl ShapeCustomConvertToRevolution {
    /// `ShapeCustom_ConvertToRevolution()`; `ShapeCustomConvertToRevolution::new()`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ShapeCustomModification for ShapeCustomConvertToRevolution {
    fn modification_messages(&self) -> &ShapeCustomModificationFields {
        &self.messages
    }
}

/// The static `IsToConvert(S, ES)`: the elementary surface (`S` itself, or the basis of
/// a rectangular trimmed or offset surface) when it is a sphere, a torus, a cylinder or
/// a cone; the elementary surface found is returned in both cases, as the native output
/// argument.
fn is_to_convert(
    surface: &Handle<dyn GeomSurface>,
) -> Result<(bool, Handle<dyn GeomElementarySurface>), Failure> {
    if surface.is_null() {
        // Natively `S->IsKind(...)` dereferences the null surface.
        return Err(Failure::with_type(
            "Standard_NullObject",
            "ShapeCustom_ConvertToRevolution: null surface",
            "",
        ));
    }
    let mut elementary = surface.cast::<dyn GeomElementarySurface>();
    if elementary.is_null() {
        if let Some(trimmed) = surface.cast::<GeomRectangularTrimmedSurface>().get() {
            elementary = trimmed.basis_surface().cast::<dyn GeomElementarySurface>();
        } else if let Some(offset) = surface.cast::<GeomOffsetSurface>().get() {
            elementary = offset.basis_surface().cast::<dyn GeomElementarySurface>();
        }
        if elementary.is_null() {
            return Ok((false, elementary));
        }
    }
    let revolved = !elementary.cast::<GeomSphericalSurface>().is_null()
        || !elementary.cast::<GeomToroidalSurface>().is_null()
        || !elementary.cast::<GeomCylindricalSurface>().is_null()
        || !elementary.cast::<GeomConicalSurface>().is_null();
    Ok((revolved, elementary))
}

/// The meridian rotated about the axis: a half circle for a sphere, the minor circle for
/// a torus, the generating line for a cylinder and a cone, all in the plane of the main
/// and X directions of the surface frame.
fn basis_curve(
    elementary: &Handle<dyn GeomElementarySurface>,
    frame: &Axis3,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let (pos, dir, x) = (frame.location(), frame.direction(), frame.x_direction());
    if let Some(sphere) = elementary.cast::<GeomSphericalSurface>().get() {
        let ax2 = Axis2::with_x_direction(pos, x.crossed(dir)?, x)?;
        let circle: Handle<dyn GeomCurve> =
            Handle::new(GeomCircle::new(ax2, sphere.radius())?).cast();
        return Ok(Handle::new(GeomTrimmedCurve::new(
            &circle,
            -PI / 2.0,
            PI / 2.0,
            true,
            true,
        )?)
        .cast());
    }
    if let Some(torus) = elementary.cast::<GeomToroidalSurface>().get() {
        let center = Point3::from(pos.xyz() + x.xyz() * torus.major_radius());
        let ax2 = Axis2::with_x_direction(center, x.crossed(dir)?, x)?;
        return Ok(Handle::new(GeomCircle::new(ax2, torus.minor_radius())?).cast());
    }
    if let Some(cylinder) = elementary.cast::<GeomCylindricalSurface>().get() {
        let origin = Point3::from(pos.xyz() + x.xyz() * cylinder.radius());
        return Ok(Handle::new(GeomLine::new(Axis1::new(origin, dir))).cast());
    }
    let cone = elementary.cast::<GeomConicalSurface>();
    let Some(cone) = cone.get() else {
        return Ok(Handle::default());
    };
    let n = Direction3::from_xyz(dir.xyz() + x.xyz() * cone.semi_angle().tan())?;
    let origin = Point3::from(pos.xyz() + x.xyz() * cone.ref_radius());
    Ok(Handle::new(GeomLine::new(Axis1::new(origin, n))).cast())
}

/// The surface of revolution with the trimming or the offset of the original surface
/// restored (the native `ES == S` test compares the handles).
fn restored_surface(
    surface: &Handle<dyn GeomSurface>,
    elementary: &Handle<dyn GeomElementarySurface>,
    revolution: Handle<dyn GeomSurface>,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    if elementary.clone().cast::<dyn GeomSurface>() == *surface {
        return Ok(revolution);
    }
    if let Some(trimmed) = surface.cast::<GeomRectangularTrimmedSurface>().get() {
        let (u1, u2, v1, v2) = trimmed.bounds();
        let restored =
            GeomRectangularTrimmedSurface::new(&revolution, (u1, u2), (v1, v2), (true, true))?;
        return Ok(Handle::new(restored).cast());
    }
    if let Some(offset) = surface.cast::<GeomOffsetSurface>().get() {
        return Ok(
            Handle::new(GeomOffsetSurface::new(&revolution, offset.offset(), false)?).cast(),
        );
    }
    Ok(revolution)
}

impl BRepToolsModification for ShapeCustomConvertToRevolution {
    /// The surface of revolution (its axis reversed for an indirect frame), the face
    /// tolerance, no reversal, and the message `ConvertToRevolution.NewSurface.MSG0` to
    /// the registrator; false (the face surface and location written) for another
    /// surface.
    fn new_surface(&self, face: &Face, result: &mut NewSurface) -> Result<bool, Failure> {
        let (surface, location) = BRepTool::surface_with_location(face)?;
        (result.surface, result.location) = (surface, location);
        let (convert, elementary) = is_to_convert(&result.surface)?;
        let Some(es) = elementary.get().filter(|_| convert) else {
            return Ok(false);
        };
        let frame = es.position();
        let basis = basis_curve(&elementary, &frame)?;
        let mut axis = frame.axis();
        if !frame.direct()? {
            axis.reverse();
        }
        let revolution: Handle<dyn GeomSurface> =
            Handle::new(GeomSurfaceOfRevolution::new(&basis, axis)?).cast();
        result.surface = restored_surface(&result.surface, &elementary, revolution)?;
        let key = AsciiString::from_bytes(b"ConvertToRevolution.NewSurface.MSG0");
        self.send_msg(face, &Msg::from_key(&key), Gravity::Info);
        result.tolerance = BRepTool::face_tolerance(face)?;
        result.rev_wires = false;
        result.rev_face = false;
        Ok(true)
    }

    /// The copied 3D curve of an edge with a p-curve on a surface to convert.
    fn new_curve(&self, edge: &Edge, result: &mut NewCurve) -> Result<bool, Failure> {
        copy_edge_curve(
            edge,
            result,
            "ShapeCustom_ConvertToRevolution::NewCurve",
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

    /// A copy of the p-curve when the surface changes or the edge was copied, moved by
    /// `(0, 2*PI)` on a sphere ("surface of revolution since based on TrimmedCurve has V
    /// parametrisation shifted on 2PI"), with the edge tolerance; false otherwise.
    fn new_curve2d(
        &self,
        edge: &Edge,
        face: &Face,
        new_edge: &Edge,
        _new_face: &Face,
        curve: &mut Handle<dyn Geom2dCurve>,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        let (surface, _) = BRepTool::surface_with_location(face)?;
        let (convert, elementary) = is_to_convert(&surface)?;
        if !convert && edge.is_same(new_edge) {
            return Ok(false);
        }
        *curve = copied_pcurve(&BRepTool::curve_on_face(edge, face)?.pcurve)?;
        let on_sphere = !elementary.cast::<GeomSphericalSurface>().is_null();
        if let Some(copy) = curve.get().filter(|_| on_sphere) {
            copy.translate(Vector2::new(0.0, 2.0 * PI))?;
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
