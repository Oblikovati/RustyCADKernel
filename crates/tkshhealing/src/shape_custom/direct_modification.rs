// Rust translation of ModelingAlgorithms/TKShHealing/ShapeCustom/ShapeCustom_DirectModification.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeCustom_DirectModification`: indirect elementary surfaces and cones of negative
//! semi-angle made direct (docs/shhealing-design.md section 7).
use super::edge_copy::copy_edge_curve;
use super::modification::{ShapeCustomModification, ShapeCustomModificationFields};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{BRepToolsModification, NewCurve, NewSurface};
use rustycad_tkbrep::topods::{self, Edge, Face, Vertex};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::message::{Gravity, Msg};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomElementarySurface, GeomRectangularTrimmedSurface, GeomSurface,
};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{self, Point3, Transform2};
use rustycad_tkmath::toploc::Location;

/// The static `IsIndirectSurface(S, L)` (S4181) result: which reversal makes the surface
/// direct.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Indirection {
    /// 0: nothing to do.
    None,
    /// 1: an indirect (left-handed) elementary surface.
    Indirect,
    /// 2: a direct cone of negative semi-angle.
    NegativeCone,
    /// 3: an indirect cone of negative semi-angle.
    IndirectNegativeCone,
}

/// The basis of nested `Geom_RectangularTrimmedSurface`s (`IsKind`), the surface itself
/// otherwise.
fn untrimmed_surface(surface: &Handle<dyn GeomSurface>) -> Handle<dyn GeomSurface> {
    let mut current = surface.clone();
    while let Some(trimmed) = current.cast::<GeomRectangularTrimmedSurface>().get() {
        let basis = trimmed.basis_surface();
        current = basis;
    }
    current
}

/// `IsIndirectSurface(S, L)`: the kind of the untrimmed elementary surface (indirect
/// when `(IsNegative != (det < 0)) == Direct()`, which the location never changes, plus
/// 2 for a cone of negative semi-angle); when it is not `None`, `surface` becomes the
/// untrimmed surface.
fn indirection(
    surface: &mut Handle<dyn GeomSurface>,
    location: &Location,
) -> Result<Indirection, Failure> {
    if surface.is_null() {
        // Natively `TS->IsKind(...)` dereferences the null surface of the face.
        return Err(Failure::with_type(
            "Standard_NullObject",
            "ShapeCustom_DirectModification: the face has no surface",
            "",
        ));
    }
    let basis = untrimmed_surface(surface);
    let elementary = basis.cast::<dyn GeomElementarySurface>();
    let Some(es) = elementary.get() else {
        return Ok(Indirection::None);
    };
    let trsf = location.transformation();
    let neg = trsf.is_negative();
    let det = trsf.vectorial_part().determinant() < 0.0;
    let direct = es.position().direct()?;
    let indirect = (neg != det) == direct;
    let negative_cone = basis
        .cast::<GeomConicalSurface>()
        .get()
        .is_some_and(|cone| cone.semi_angle() < 0.0);
    let kind = match (indirect, negative_cone) {
        (false, false) => return Ok(Indirection::None),
        (true, false) => Indirection::Indirect,
        (false, true) => Indirection::NegativeCone,
        (true, true) => Indirection::IndirectNegativeCone,
    };
    *surface = basis;
    Ok(kind)
}

/// `ShapeCustom_DirectModification`: a face on an indirect elementary surface gets the
/// U-reversed surface and is reversed with its wires; a face on a cone of negative
/// semi-angle gets the cone reversed in U and V (and, indirect, only in V), so that the
/// normals keep their direction. Edges with a p-curve on such a surface are copied with
/// mirrored p-curves.
///
/// ```
/// use rustycad_tkbrep::brep_tools::{BRepToolsModification, NewSurface};
/// use rustycad_tkbrep::topods::Face;
/// use rustycad_tkshhealing::shape_custom::ShapeCustomDirectModification;
/// let modification = ShapeCustomDirectModification::new();
/// assert!(modification.new_surface(&Face::default(), &mut NewSurface::default()).is_err());
/// ```
#[derive(Default)]
pub struct ShapeCustomDirectModification {
    parent: StandardTransient,
    messages: ShapeCustomModificationFields,
}
impl_transient!(
    ShapeCustomDirectModification,
    "ShapeCustom_DirectModification",
    dyn ShapeCustomModification,
    parent,
    interfaces: [dyn BRepToolsModification, dyn ShapeCustomModification]
);

impl ShapeCustomDirectModification {
    /// `ShapeCustom_DirectModification()`; `ShapeCustomDirectModification::new()`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ShapeCustomModification for ShapeCustomDirectModification {
    fn modification_messages(&self) -> &ShapeCustomModificationFields {
        &self.messages
    }
}

/// The surface of `NewSurface` for an indirection with the value of `RevWires` and `RevFace`.
type ReversedSurface = Option<(Handle<dyn GeomSurface>, bool)>;

/// The surface of `NewSurface` for an indirection, with `RevWires`/`RevFace`.
fn reversed_surface(
    surface: &Handle<dyn GeomSurface>,
    kind: Indirection,
) -> Result<ReversedSurface, Failure> {
    let Some(stored) = surface.get() else {
        return Ok(None);
    };
    Ok(match kind {
        Indirection::None => None,
        Indirection::Indirect => Some((stored.u_reversed()?, true)),
        Indirection::NegativeCone => {
            let reversed = stored.v_reversed()?;
            if let Some(copy) = reversed.get() {
                copy.u_reverse()?;
            }
            Some((reversed, false))
        }
        Indirection::IndirectNegativeCone => Some((stored.v_reversed()?, true)),
    })
}

/// The p-curve mirrored for the reversal of its surface: about the V axis for an
/// indirect surface, about both axes for a negative cone, about the U axis for an
/// indirect negative cone.
fn mirrored_pcurve(
    curve: &Handle<dyn Geom2dCurve>,
    kind: Indirection,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let stored = curve.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "ShapeCustom_DirectModification::NewCurve2d: the edge has no p-curve on the face",
            "",
        )
    })?;
    let mut trsf = Transform2::default();
    let first_axis = if kind == Indirection::Indirect {
        gp::oy2d()
    } else {
        gp::ox2d()
    };
    trsf.set_mirror_axis(first_axis);
    let mirrored = stored.transformed(trsf)?.cast::<dyn Geom2dCurve>();
    if kind == Indirection::NegativeCone {
        trsf.set_mirror_axis(gp::oy2d());
        if let Some(copy) = mirrored.get() {
            copy.transform(trsf)?;
        }
    }
    Ok(mirrored)
}

impl BRepToolsModification for ShapeCustomDirectModification {
    /// The untrimmed surface reversed for its indirection, the face tolerance, the
    /// reversal flags and the message `DirectModification.NewSurface.MSG0` to the
    /// registrator; false (with the face surface and location written, the flags left
    /// unchanged) for a direct surface.
    fn new_surface(&self, face: &Face, result: &mut NewSurface) -> Result<bool, Failure> {
        let (mut surface, location) = BRepTool::surface_with_location(face)?;
        result.location = location;
        let kind = indirection(&mut surface, &result.location)?;
        result.surface = surface;
        let Some((reversed, rev_face)) = reversed_surface(&result.surface, kind)? else {
            return Ok(false);
        };
        result.surface = reversed;
        result.rev_wires = rev_face;
        result.rev_face = rev_face;
        let key = AsciiString::from_bytes(b"DirectModification.NewSurface.MSG0");
        self.send_msg(face, &Msg::from_key(&key), Gravity::Info);
        result.tolerance = BRepTool::face_tolerance(face)?;
        Ok(true)
    }

    /// The copied 3D curve of an edge with a p-curve on an indirect surface.
    fn new_curve(&self, edge: &Edge, result: &mut NewCurve) -> Result<bool, Failure> {
        copy_edge_curve(
            edge,
            result,
            "ShapeCustom_DirectModification::NewCurve",
            |surface, location| {
                let mut surface = surface.clone();
                Ok(indirection(&mut surface, location)? != Indirection::None)
            },
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

    /// On an indirect surface, the mirrored p-curve (and, for a seam whose new edge has
    /// no second p-curve yet, both p-curves put on the new edge: "#26 rln When seam edge
    /// contains triangulations trimming is lost by BRep_Builder::UpdateEdge"); on another
    /// surface, a copy of the p-curve when the edge was copied (":p5 abv 26 Feb 99"),
    /// false when it was not.
    fn new_curve2d(
        &self,
        edge: &Edge,
        face: &Face,
        new_edge: &Edge,
        new_face: &Face,
        curve: &mut Handle<dyn Geom2dCurve>,
        tolerance: &mut f64,
    ) -> Result<bool, Failure> {
        let (mut surface, location) = BRepTool::surface_with_location(face)?;
        let kind = indirection(&mut surface, &location)?;
        if kind == Indirection::None && edge.is_same(new_edge) {
            return Ok(false);
        }
        *curve = BRepTool::curve_on_face(edge, face)?.pcurve;
        *tolerance = BRepTool::edge_tolerance(edge)?;
        if kind == Indirection::None {
            *curve = copied_pcurve(curve)?;
            return Ok(true);
        }
        *curve = mirrored_pcurve(curve, kind)?;
        keep_seam_trimming(edge, face, new_edge, new_face, curve, *tolerance)?;
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

/// `occ::down_cast<Geom2d_Curve>(C->Copy())`, a null curve staying null.
pub(super) fn copied_pcurve(
    curve: &Handle<dyn Geom2dCurve>,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    match curve.get() {
        Some(stored) => Ok(stored.copy()?.cast::<dyn Geom2dCurve>()),
        None => Ok(Handle::default()),
    }
}

/// For a really closed edge, when the reversed new edge has no p-curve on the new face,
/// the original p-curve and the mirrored one are set as the seam of the new edge, with
/// the original range ("anyway, tmp will be removed later by BRepTools_Modifier").
fn keep_seam_trimming(
    edge: &Edge,
    face: &Face,
    new_edge: &Edge,
    new_face: &Face,
    mirrored: &Handle<dyn Geom2dCurve>,
    tolerance: f64,
) -> Result<(), Failure> {
    if !rustycad_tkbrep::brep_tools::is_really_closed(edge, face)? {
        return Ok(());
    }
    let reversed = topods::edge(&new_edge.reversed())?;
    if !BRepTool::curve_on_face(&reversed, new_face)?
        .pcurve
        .is_null()
    {
        return Ok(());
    }
    let original = BRepTool::curve_on_face(edge, face)?;
    let builder = BRepBuilder::default();
    builder.update_edge_pcurves_on_face(
        new_edge,
        (&original.pcurve, mirrored),
        new_face,
        tolerance,
    )?;
    builder.range_on_face(new_edge, new_face, original.first, original.last)
}
