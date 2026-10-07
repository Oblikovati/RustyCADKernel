// Rust port of ApplicationFramework/TKCAF/TNaming/TNaming_TranslateTool.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TNaming_TranslateTool`: the shape factory of `TNaming_CopyShape`; it makes empty BRep
//! shapes and copies the BRep data of one TShape into another.
use super::TNamingCopyShape;
use super::copy_shape::TNamingTransientMap;
use rustycad_tkbrep::brep::{
    BRepBuilder, BRepCurveRepresentation, BRepGCurve, BRepPointOnCurve, BRepPointOnCurveOnSurface,
    BRepPointOnSurface, BRepPointRepresentation, BRepTEdge, BRepTFace, BRepTVertex,
};
use rustycad_tkbrep::topods::{self, Shape, TShape};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, RuntimeTyped, StandardTransient, exceptions},
};
use rustycad_tkmath::poly::mesh_purpose;

/// `TNaming_TranslateTool`.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkcaf::tnaming::TNamingTranslateTool;
/// use rustycad_tkg3d::top_abs::ShapeEnum;
/// let mut shape = Shape::default();
/// TNamingTranslateTool::new().make_wire(&mut shape).unwrap();
/// assert_eq!(shape.shape_type().unwrap(), ShapeEnum::Wire);
/// ```
#[derive(Default)]
pub struct TNamingTranslateTool {
    transient: StandardTransient,
}
impl_transient!(
    TNamingTranslateTool,
    "TNaming_TranslateTool",
    StandardTransient,
    transient
);

/// Declares the `Make*` members: the BRep builder makes an empty shape of the type.
macro_rules! make_member {
    ($(#[$doc:meta])* $name:ident, $cast:ident, $builder:ident) => {
        $(#[$doc])*
        pub fn $name(&self, shape: &mut Shape) -> Result<(), Failure> {
            let mut typed = topods::$cast(shape)?;
            BRepBuilder::default().$builder(&mut typed);
            *shape = typed.into_shape();
            Ok(())
        }
    };
}

impl TNamingTranslateTool {
    /// `new TNaming_TranslateTool()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Add(S1, S2)`: `BRep_Builder::Add`.
    pub fn add(&self, shape: &mut Shape, component: &Shape) -> Result<(), Failure> {
        BRepBuilder::default().add(shape, component)
    }
    make_member!(/// `MakeVertex`.
        make_vertex, vertex, make_vertex);
    /// `MakeEdge`.
    pub fn make_edge(&self, shape: &mut Shape) -> Result<(), Failure> {
        let mut edge = topods::edge(shape)?;
        BRepBuilder::default().make_edge(&mut edge)?;
        *shape = edge.into_shape();
        Ok(())
    }
    make_member!(/// `MakeWire`.
        make_wire, wire, make_wire);
    make_member!(/// `MakeFace`.
        make_face, face, make_face);
    make_member!(/// `MakeShell`.
        make_shell, shell, make_shell);
    make_member!(/// `MakeSolid`.
        make_solid, solid, make_solid);
    make_member!(/// `MakeCompSolid`.
        make_comp_solid, comp_solid, make_comp_solid);
    make_member!(/// `MakeCompound`.
        make_compound, compound, make_compound);
    /// `UpdateVertex(S1, S2, M)`: the point, the tolerance and the point representations
    /// (their locations translated), then the flags.
    pub fn update_vertex(
        &self,
        source: &Shape,
        target: &mut Shape,
        map: &mut TNamingTransientMap,
    ) -> Result<(), Failure> {
        let from = brep_tshape::<BRepTVertex>(source, "TNaming_TranslateTool::UpdateVertex")?;
        let into = brep_tshape::<BRepTVertex>(target, "TNaming_TranslateTool::UpdateVertex")?;
        let (from, into) = (live(&from)?, live(&into)?);
        into.set_pnt(from.pnt());
        into.set_tolerance(from.tolerance());
        let mut points = Vec::new();
        for point in from.points() {
            points.push(translate_point(live(&point)?, map)?);
        }
        into.change_points(|list| *list = points);
        self.update_shape(source, target)
    }
    /// `UpdateEdge(S1, S2, M)`: the tolerance, the flags of the edge and copies of its
    /// curve representations (locations translated; an unknown representation is
    /// skipped), then the shape flags.
    pub fn update_edge(
        &self,
        source: &Shape,
        target: &mut Shape,
        map: &mut TNamingTransientMap,
    ) -> Result<(), Failure> {
        let from = brep_tshape::<BRepTEdge>(source, "TNaming_TranslateTool::UpdateEdge")?;
        let into = brep_tshape::<BRepTEdge>(target, "TNaming_TranslateTool::UpdateEdge")?;
        let (from, into) = (live(&from)?, live(&into)?);
        into.set_tolerance(from.tolerance());
        into.set_same_parameter(from.same_parameter());
        into.set_same_range(from.same_range());
        into.set_degenerated(from.degenerated());
        let mut curves = Vec::new();
        for curve in from.curves() {
            if let Some(copy) = translate_curve(&curve, map)? {
                curves.push(copy);
            }
        }
        into.change_curves(|list| *list = curves);
        self.update_shape(source, target)
    }
    /// `UpdateFace(S1, S2, M)`: the natural restriction, the tolerance, the translated
    /// location, the surface and the triangulation, then the flags.
    pub fn update_face(
        &self,
        source: &Shape,
        target: &mut Shape,
        map: &mut TNamingTransientMap,
    ) -> Result<(), Failure> {
        let from = brep_tshape::<BRepTFace>(source, "TNaming_TranslateTool::UpdateFace")?;
        let into = brep_tshape::<BRepTFace>(target, "TNaming_TranslateTool::UpdateFace")?;
        let (from, into) = (live(&from)?, live(&into)?);
        into.set_natural_restriction(from.natural_restriction());
        into.set_tolerance(from.tolerance());
        into.set_location(&TNamingCopyShape::translate_location(
            &from.location(),
            map,
        )?);
        into.set_surface(&from.surface());
        into.set_triangulation(&from.triangulation(mesh_purpose::NONE), true);
        self.update_shape(source, target)
    }
    /// `UpdateShape(S1, S2)`: the TShape flags of `S1` (free, modified, checked,
    /// orientable, closed, infinite, convex, in that order) copied onto `S2`.
    pub fn update_shape(&self, source: &Shape, target: &mut Shape) -> Result<(), Failure> {
        let from = source.tshape().clone();
        let into = target.tshape().clone();
        let who = "TNaming_TranslateTool::UpdateShape";
        let (from, into) = (null_checked(&from, who)?, null_checked(&into, who)?);
        into.set_free(from.free());
        into.set_modified(from.modified());
        into.set_checked(from.checked());
        into.set_orientable(from.orientable());
        into.set_closed(from.closed());
        into.set_infinite(from.infinite());
        into.set_convex(from.convex());
        Ok(())
    }
}

/// The native C-style cast `*((occ::handle<BRep_TX>*)&S.TShape())`: a TShape of another
/// class is `Standard_TypeMismatch` (undefined behaviour natively).
fn brep_tshape<T: RuntimeTyped>(shape: &Shape, who: &str) -> Result<Handle<T>, Failure> {
    let tshape = shape.tshape().cast::<T>();
    if tshape.is_null() {
        return Err(exceptions::type_mismatch(&format!(
            "{who}: the TShape is not a {}; expected a BRep shape of that type",
            T::TYPE_NAME
        )));
    }
    Ok(tshape)
}

fn live<T: RuntimeTyped + ?Sized>(handle: &Handle<T>) -> Result<&T, Failure> {
    rustycad_tklcaf::live(handle, "TNaming_TranslateTool")
}

fn null_checked<'a>(tshape: &'a Handle<dyn TShape>, who: &str) -> Result<&'a dyn TShape, Failure> {
    rustycad_tklcaf::live(tshape, who)
}

/// One point representation of `UpdateVertex`: a new representation on the same geometry
/// with a translated location (`None` for a kind the native code leaves null).
fn translate_point(
    point: &dyn BRepPointRepresentation,
    map: &mut TNamingTransientMap,
) -> Result<Handle<dyn BRepPointRepresentation>, Failure> {
    let location = TNamingCopyShape::translate_location(&point.location(), map)?;
    let parameter = point.parameter();
    if point.is_point_on_curve() {
        let copy = BRepPointOnCurve::new(parameter, &point.curve()?, &location);
        return Ok(Handle::new(copy).cast());
    }
    if point.is_point_on_curve_on_surface() {
        let copy = BRepPointOnCurveOnSurface::new(
            parameter,
            &point.pcurve()?,
            &point.surface()?,
            &location,
        );
        return Ok(Handle::new(copy).cast());
    }
    if point.is_point_on_surface() {
        let surface = point.surface()?;
        let copy = BRepPointOnSurface::new(parameter, point.parameter2()?, &surface, &location);
        return Ok(Handle::new(copy).cast());
    }
    // The native list then holds a null representation.
    Ok(Handle::default())
}

/// One curve representation of `UpdateEdge`; `None` for a representation it skips.
fn translate_curve(
    curve: &Handle<dyn BRepCurveRepresentation>,
    map: &mut TNamingTransientMap,
) -> Result<Option<Handle<dyn BRepCurveRepresentation>>, Failure> {
    let source = live(curve)?;
    let gcurve = curve.cast::<dyn BRepGCurve>();
    let copy = if let Some(gcurve) = gcurve.get() {
        let (first, last) = gcurve.range();
        let copy = source.copy()?;
        live(&copy.cast::<dyn BRepGCurve>())?.set_range(first, last)?;
        copy
    } else if source.is_regularity() {
        let copy = source.copy()?;
        // Natively the translated second location goes to the first one, which the
        // common code below then overwrites: the second location stays as copied.
        let second = TNamingCopyShape::translate_location(&source.location2()?, map)?;
        live(&copy)?.set_location(&second);
        copy
    } else if is_polygon(source) {
        source.copy()?
    } else {
        return Ok(None);
    };
    let location = TNamingCopyShape::translate_location(&source.location(), map)?;
    live(&copy)?.set_location(&location);
    Ok(Some(copy))
}

/// The polygon representations `UpdateEdge` copies.
fn is_polygon(curve: &dyn BRepCurveRepresentation) -> bool {
    curve.is_polygon3d() || curve.is_polygon_on_surface() || curve.is_polygon_on_triangulation()
}
