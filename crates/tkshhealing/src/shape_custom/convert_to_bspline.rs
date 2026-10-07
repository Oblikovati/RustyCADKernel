// Rust translation of ModelingAlgorithms/TKShHealing/ShapeCustom/ShapeCustom_ConvertToBSpline.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeCustom_ConvertToBSpline`: extrusion, revolution, offset and (optionally) plane
//! surfaces converted to B-spline surfaces (docs/shhealing-design.md section 7).
use super::direct_modification::copied_pcurve;
use super::edge_copy::copy_edge_curve;
use super::modification::{ShapeCustomModification, ShapeCustomModificationFields};
use crate::shape_construct::{
    ShapeConstruct, ShapeConstructApproxSettings, ShapeConstructSurfaceBounds,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::{BRepToolsModification, NewCurve, NewSurface, uv_bounds};
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::message::{Gravity, Msg};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{
    GeomOffsetSurface, GeomPlane, GeomRectangularTrimmedSurface, GeomSurface,
    GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution,
};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point3;

/// The conversion switches of `ShapeCustom_ConvertToBSpline` (myExtrMode, myRevolMode,
/// myOffsetMode, myPlaneMode).
#[derive(Clone, Copy)]
struct ConvertModes {
    extrusion: bool,
    revolution: bool,
    offset: bool,
    plane: bool,
}

/// `ShapeCustom_ConvertToBSpline`: a face on a surface of linear extrusion, of revolution,
/// an offset surface (all on by default) or a plane (off by default), possibly trimmed,
/// gets a B-spline surface from `ShapeConstruct::ConvertSurfaceToBSpline` (with the offset
/// kept as an offset of the converted basis when the offset mode is off); edges with a
/// p-curve on such a surface are copied.
///
/// ```
/// use rustycad_tkbrep::brep_tools::{BRepToolsModification, NewSurface};
/// use rustycad_tkbrep::topods::Face;
/// use rustycad_tkshhealing::shape_custom::ShapeCustomConvertToBSpline;
/// let modification = ShapeCustomConvertToBSpline::new();
/// modification.set_plane_mode(true);
/// assert!(modification.new_surface(&Face::default(), &mut NewSurface::default()).is_err());
/// ```
pub struct ShapeCustomConvertToBSpline {
    parent: StandardTransient,
    messages: ShapeCustomModificationFields,
    modes: SharedState<ConvertModes>,
}
impl_transient!(
    ShapeCustomConvertToBSpline,
    "ShapeCustom_ConvertToBSpline",
    dyn ShapeCustomModification,
    parent,
    interfaces: [dyn BRepToolsModification, dyn ShapeCustomModification]
);

impl Default for ShapeCustomConvertToBSpline {
    fn default() -> Self {
        Self {
            parent: StandardTransient::default(),
            messages: ShapeCustomModificationFields::default(),
            modes: SharedState::new(ConvertModes {
                extrusion: true,
                revolution: true,
                offset: true,
                plane: false,
            }),
        }
    }
}

impl ShapeCustomConvertToBSpline {
    /// `ShapeCustom_ConvertToBSpline()`: extrusion, revolution and offset modes on, plane
    /// mode off; `ShapeCustomConvertToBSpline::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `SetExtrusionMode(extrMode)`; `modification.set_extrusion_mode(false)`.
    pub fn set_extrusion_mode(&self, mode: bool) {
        self.modes.write().extrusion = mode;
    }

    /// `SetRevolutionMode(revolMode)`; `modification.set_revolution_mode(false)`.
    pub fn set_revolution_mode(&self, mode: bool) {
        self.modes.write().revolution = mode;
    }

    /// `SetOffsetMode(offsetMode)`: off, an offset surface keeps its offset and only its
    /// basis is converted; `modification.set_offset_mode(false)`.
    pub fn set_offset_mode(&self, mode: bool) {
        self.modes.write().offset = mode;
    }

    /// `SetPlaneMode(planeMode)`; `modification.set_plane_mode(true)`.
    pub fn set_plane_mode(&self, mode: bool) {
        self.modes.write().plane = mode;
    }

    /// The protected `IsToConvert(S, SS)`: whether `surface` (the basis of a trimmed one,
    /// returned as the second value) is converted under the current modes; an offset
    /// surface with the offset mode off is decided by its basis.
    fn is_to_convert(
        &self,
        surface: &Handle<dyn GeomSurface>,
    ) -> Result<(bool, Handle<dyn GeomSurface>), Failure> {
        if surface.is_null() {
            // Natively `S->IsKind(...)` dereferences the null surface.
            return Err(Failure::with_type(
                "Standard_NullObject",
                "ShapeCustom_ConvertToBSpline::IsToConvert: null surface; expected a Geom_Surface",
                "",
            ));
        }
        let mut basis = surface.clone();
        if let Some(trimmed) = surface.cast::<GeomRectangularTrimmedSurface>().get() {
            basis = trimmed.basis_surface();
        }
        let modes = *self.modes.read();
        if let Some(offset) = basis.cast::<GeomOffsetSurface>().get() {
            if modes.offset {
                return Ok((true, basis));
            }
            return Ok((self.is_to_convert(&offset.basis_surface())?.0, basis));
        }
        Ok((kind_mode(&basis, &modes), basis))
    }
}

fn kind_mode(surface: &Handle<dyn GeomSurface>, modes: &ConvertModes) -> bool {
    if !surface.cast::<GeomSurfaceOfLinearExtrusion>().is_null() {
        return modes.extrusion;
    }
    if !surface.cast::<GeomSurfaceOfRevolution>().is_null() {
        return modes.revolution;
    }
    if !surface.cast::<GeomPlane>().is_null() {
        return modes.plane;
    }
    false
}

impl ShapeCustomModification for ShapeCustomConvertToBSpline {
    fn modification_messages(&self) -> &ShapeCustomModificationFields {
        &self.messages
    }
}

/// The conversion box: the surface bounds, an infinite direction replaced by the UV
/// bounds of the face.
fn conversion_bounds(
    surface: &Handle<dyn GeomSurface>,
    face: &Face,
) -> Result<ShapeConstructSurfaceBounds, Failure> {
    let (mut u1, mut u2, mut v1, mut v2) = surface.bounds();
    let (umin, umax, vmin, vmax) = uv_bounds(face)?;
    if precision::is_infinite(u1) || precision::is_infinite(u2) {
        (u1, u2) = (umin, umax);
    }
    if precision::is_infinite(v1) || precision::is_infinite(v2) {
        (v1, v2) = (vmin, vmax);
    }
    Ok(ShapeConstructSurfaceBounds {
        uf: u1,
        ul: u2,
        vf: v1,
        vl: v2,
    })
}

fn approx_settings(continuity: Continuity) -> ShapeConstructApproxSettings {
    ShapeConstructApproxSettings {
        tol3d: precision::approximation(),
        continuity,
        max_segments: 10000,
        max_degree: 15,
    }
}

impl ShapeCustomConvertToBSpline {
    // The offset of a converted basis (offset mode off), or the converted surface itself,
    // whose continuity is C0 for an offset ("pdn 30.06.99 because of hang-up in
    // GeomConvert_ApproxSurface").
    fn converted(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: ShapeConstructSurfaceBounds,
    ) -> Result<Handle<dyn GeomSurface>, Failure> {
        let offset = surface.cast::<GeomOffsetSurface>();
        if let Some(offset) = offset.get().filter(|_| !self.modes.read().offset) {
            let settings = approx_settings(surface.continuity());
            let spline: Handle<dyn GeomSurface> = ShapeConstruct::convert_surface_to_bspline(
                &offset.basis_surface(),
                bounds,
                settings,
            )?
            .cast();
            return Ok(
                Handle::new(GeomOffsetSurface::new(&spline, offset.offset(), false)?).cast(),
            );
        }
        let continuity = if offset.is_null() {
            surface.continuity()
        } else {
            Continuity::C0
        };
        let spline = ShapeConstruct::convert_surface_to_bspline(
            surface,
            bounds,
            approx_settings(continuity),
        )?;
        Ok(spline.cast())
    }
}

/// The converted surface trimmed again to the bounds of a trimmed original.
fn retrimmed(
    original: &Handle<dyn GeomSurface>,
    converted: Handle<dyn GeomSurface>,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let trimmed = original.cast::<GeomRectangularTrimmedSurface>();
    let Some(trimmed) = trimmed.get() else {
        return Ok(converted);
    };
    let (uf, ul, vf, vl) = trimmed.bounds();
    let restored =
        GeomRectangularTrimmedSurface::new(&converted, (uf, ul), (vf, vl), (true, true))?;
    Ok(Handle::new(restored).cast())
}

fn send(
    modification: &ShapeCustomConvertToBSpline,
    shape: &rustycad_tkbrep::topods::Shape,
    key: &[u8],
) {
    let key = AsciiString::from_bytes(key);
    modification.send_msg(shape, &Msg::from_key(&key), Gravity::Info);
}

impl BRepToolsModification for ShapeCustomConvertToBSpline {
    /// The B-spline surface (re-trimmed to a trimmed original), the face tolerance, no
    /// reversal and the message `ConvertToBSpline.NewSurface.MSG0`; false (the face surface
    /// and location written) when the surface is not converted.
    fn new_surface(&self, face: &Face, result: &mut NewSurface) -> Result<bool, Failure> {
        let (surface, location) = BRepTool::surface_with_location(face)?;
        (result.surface, result.location) = (surface, location);
        let surface = result.surface.clone();
        let bounds = conversion_bounds(&surface, face)?;
        let (convert, basis) = self.is_to_convert(&surface)?;
        if !convert {
            return Ok(false);
        }
        result.surface = retrimmed(&surface, self.converted(&basis, bounds)?)?;
        send(
            self,
            &face.clone().into_shape(),
            b"ConvertToBSpline.NewSurface.MSG0",
        );
        result.tolerance = BRepTool::face_tolerance(face)?;
        result.rev_wires = false;
        result.rev_face = false;
        Ok(true)
    }

    /// The copied 3D curve of an edge with a p-curve on a surface to convert, and the
    /// message `ConvertToBSpline.NewCurve.MSG0`.
    fn new_curve(&self, edge: &Edge, result: &mut NewCurve) -> Result<bool, Failure> {
        let copied = copy_edge_curve(
            edge,
            result,
            "ShapeCustom_ConvertToBSpline::NewCurve",
            |surface, _location| Ok(self.is_to_convert(surface)?.0),
        )?;
        if copied {
            send(
                self,
                &edge.clone().into_shape(),
                b"ConvertToBSpline.NewCurve.MSG0",
            );
        }
        Ok(copied)
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

    /// A copy of the p-curve when the surface is converted or the edge was copied, with
    /// the edge tolerance; false otherwise.
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
        if !self.is_to_convert(&surface)?.0 && edge.is_same(new_edge) {
            return Ok(false);
        }
        *curve = copied_pcurve(&BRepTool::curve_on_face(edge, face)?.pcurve)?;
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
