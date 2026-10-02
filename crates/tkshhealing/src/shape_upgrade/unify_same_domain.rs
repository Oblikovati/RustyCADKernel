//! Focused same-domain face merge and no-merge behavior for `ShapeUpgrade_UnifySameDomain`.
mod internal_wires;
use crate::shape_build::ShapeBuildReShape;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Face, Shape, ShapeIterator, Wire};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeWire};

#[derive(Clone)]
struct UnifyFields {
    input: Shape,
    result: Shape,
    linear_tolerance: f64,
    angular_tolerance: f64,
    unify_edges: bool,
    unify_faces: bool,
    _concat_bspline_edges: bool,
    allow_internal_edges: bool,
}

/// `ShapeUpgrade_UnifySameDomain` on shapes with no mergeable face or edge neighbors.
/// `let unify = ShapeUpgradeUnifySameDomain::new(&box_shape, true, true, true);`.
pub struct ShapeUpgradeUnifySameDomain {
    parent: StandardTransient,
    fields: SharedState<UnifyFields>,
}
impl_transient!(
    ShapeUpgradeUnifySameDomain,
    "ShapeUpgrade_UnifySameDomain",
    StandardTransient,
    parent
);

impl ShapeUpgradeUnifySameDomain {
    /// Initializes the supported no-candidate Build path.
    pub fn new(
        shape: &Shape,
        unify_edges: bool,
        unify_faces: bool,
        concat_bspline_edges: bool,
    ) -> Self {
        Self {
            parent: StandardTransient::default(),
            fields: SharedState::new(UnifyFields {
                input: shape.clone(),
                result: shape.clone(),
                linear_tolerance: precision::confusion(),
                angular_tolerance: precision::angular(),
                unify_edges,
                unify_faces,
                _concat_bspline_edges: concat_bspline_edges,
                allow_internal_edges: false,
            }),
        }
    }

    /// `Initialize` replaces the input and operation flags; `unify.initialize(&shape, true, true, true)`.
    pub fn initialize(
        &self,
        shape: &Shape,
        unify_edges: bool,
        unify_faces: bool,
        concat_bspline_edges: bool,
    ) {
        let mut fields = self.fields.write();
        fields.input = shape.clone();
        fields.result = shape.clone();
        fields.unify_edges = unify_edges;
        fields.unify_faces = unify_faces;
        fields._concat_bspline_edges = concat_bspline_edges;
    }

    /// `SetLinearTolerance`; `unify.set_linear_tolerance(1e-5)`.
    pub fn set_linear_tolerance(&self, tolerance: f64) {
        self.fields.write().linear_tolerance = tolerance;
    }

    /// `SetAngularTolerance` clamps values below `Precision::Angular()`.
    pub fn set_angular_tolerance(&self, tolerance: f64) {
        self.fields.write().angular_tolerance = tolerance.max(precision::angular());
    }

    /// `AllowInternalEdges` controls whether internal edges may be retained.
    pub fn allow_internal_edges(&self, allow: bool) {
        self.fields.write().allow_internal_edges = allow;
    }

    /// `Build` preserves the input when no enabled same-domain merge is possible.
    pub fn build(&self) -> Result<(), Failure> {
        let fields = self.fields.read().clone();
        let input_face_count = count_shapes(&fields.input, ShapeEnum::Face)?;
        let mut result = fields.input.clone();
        if fields.unify_faces {
            result = merge_coplanar_faces(&result, &fields)?;
        }
        let face_candidate = fields.unify_faces && first_coplanar_pair(&result, &fields)?.is_some();
        let edge_candidate =
            fields.unify_edges && has_collinear_edge_pair(&result, fields.angular_tolerance)?;
        let face_merge_completed = count_shapes(&result, ShapeEnum::Face)? < input_face_count;
        let has_candidate = face_candidate || edge_candidate && !face_merge_completed;
        if has_candidate {
            return Err(Failure::with_type(
                "Standard_NotImplemented",
                "ShapeUpgrade_UnifySameDomain::Build requires face or edge unification",
                "",
            ));
        }
        let mut fields = fields;
        fields.result = result;
        *self.fields.write() = fields;
        Ok(())
    }

    /// `Shape()` returns the current result, initially the input.
    pub fn shape(&self) -> Shape {
        self.fields.read().result.clone()
    }
}

/// Merges planar face pairs until no supported adjacent pair remains.
fn merge_coplanar_faces(shape: &Shape, fields: &UnifyFields) -> Result<Shape, Failure> {
    let mut result = shape.clone();
    let limit = count_shapes(&result, ShapeEnum::Face)?;
    for _ in 0..limit {
        let Some((first, second)) = first_coplanar_pair(&result, fields)? else {
            break;
        };
        result = merge_face_pair(&result, &first, &second, fields.allow_internal_edges)?;
    }
    Ok(result)
}

/// Finds a shared-edge face pair on the same planar support.
fn first_coplanar_pair(
    shape: &Shape,
    fields: &UnifyFields,
) -> Result<Option<(Face, Face)>, Failure> {
    let faces = shape_faces(shape)?;
    for first_index in 0..faces.len() {
        for second_index in first_index + 1..faces.len() {
            if let Some(pair) =
                supported_face_pair(&faces[first_index], &faces[second_index], fields)?
            {
                return Ok(Some(pair));
            }
        }
    }
    Ok(None)
}

/// Collects faces once so coincident boundaries need not share edge identities.
fn shape_faces(shape: &Shape) -> Result<Vec<Face>, Failure> {
    Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?
        .map(|face| topods::face(&face))
        .collect()
}

/// Chooses the supported merge or skips cylindrical pairs whose source p-curve is absent.
fn supported_face_pair(
    first: &Face,
    second: &Face,
    fields: &UnifyFields,
) -> Result<Option<(Face, Face)>, Failure> {
    let shared = shared_outer_edge(first, second)?;
    if shared.is_none() {
        return Ok(None);
    }
    if planes_share_support(first, second, fields)?
        && internal_merge_allowed(first, second, fields)?
    {
        return Ok(Some((first.clone(), second.clone())));
    }
    if skip_missing_cylindrical_pair(first, second)? {
        return Ok(None);
    }
    Ok(None)
}

/// Skips source pairs that cannot transfer an absent cylindrical p-curve.
fn skip_missing_cylindrical_pair(first: &Face, second: &Face) -> Result<bool, Failure> {
    if !cylinders_share_support(first, second)? {
        return Ok(false);
    }
    if pair_has_missing_pcurve(first, second)? {
        return Ok(true);
    }
    Err(Failure::with_type(
        "Standard_NotImplemented",
        "ShapeUpgrade_UnifySameDomain::Build requires cylindrical face merging",
        "",
    ))
}

/// Checks the `AllowInternalEdges` gate used before a face pair can be merged.
fn internal_merge_allowed(
    first: &Face,
    second: &Face,
    fields: &UnifyFields,
) -> Result<bool, Failure> {
    Ok(fields.allow_internal_edges
        || has_no_internal_wires(first)? && has_no_internal_wires(second)?)
}

/// Whether the two faces carry the same located cylindrical support.
fn cylinders_share_support(first: &Face, second: &Face) -> Result<bool, Failure> {
    let first = BRepAdaptorSurface::with_face(first, false)?;
    let second = BRepAdaptorSurface::with_face(second, false)?;
    if first.get_type()? != SurfaceType::Cylinder || second.get_type()? != SurfaceType::Cylinder {
        return Ok(false);
    }
    Ok(first.cylinder()?.coefficients() == second.cylinder()?.coefficients())
}

/// Whether either source face has a boundary edge without a stored p-curve.
fn pair_has_missing_pcurve(first: &Face, second: &Face) -> Result<bool, Failure> {
    Ok(face_has_missing_pcurve(first)? || face_has_missing_pcurve(second)?)
}

/// The native face transform skips an edge when its source representation is absent.
fn face_has_missing_pcurve(face: &Face) -> Result<bool, Failure> {
    let edges = Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )?;
    for edge in edges {
        if !BRepTool::curve_on_face(&topods::edge(&edge)?, face)?.is_stored {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Compares two planar supports; this slice leaves other shared surfaces untouched.
fn planes_share_support(
    first: &topods::Face,
    second: &topods::Face,
    fields: &UnifyFields,
) -> Result<bool, Failure> {
    let first_adaptor = BRepAdaptorSurface::with_face(first, false)?;
    let second_adaptor = BRepAdaptorSurface::with_face(second, false)?;
    if first_adaptor.get_type()? != SurfaceType::Plane
        || second_adaptor.get_type()? != SurfaceType::Plane
    {
        return Ok(false);
    }
    let a = first_adaptor.plane()?.coefficients()?;
    let b = second_adaptor.plane()?.coefficients()?;
    let dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    if (1.0 - dot.abs()).abs() > fields.angular_tolerance {
        return Ok(false);
    }
    let offset = if dot < 0.0 { a[3] + b[3] } else { a[3] - b[3] };
    Ok(offset.abs() <= fields.linear_tolerance)
}

/// Rebuilds two boundary-only coplanar faces as one face and records both replacements.
fn merge_face_pair(
    shape: &Shape,
    first: &Face,
    second: &Face,
    allow_internal: bool,
) -> Result<Shape, Failure> {
    let boundary = merged_boundary(first, second)?;
    let mut merged = make_merged_face(first, &boundary)?;
    if allow_internal {
        internal_wires::add_internal_wires(&mut merged, first, second)?;
    }
    replace_merged_pair(shape, first, second, &merged)
}

/// Makes the single ordered boundary wire from both faces after dropping their shared edge.
fn merged_boundary(first: &Face, second: &Face) -> Result<Wire, Failure> {
    let first_edges = outer_edges(first)?;
    let second_edges = outer_edges(second)?;
    let shared = shared_edge(&first_edges, &second_edges)?;
    let mut wire = BRepBuilderAPIMakeWire::new();
    add_unshared_edges(&mut wire, &first_edges, &shared)?;
    add_unshared_edges(&mut wire, &second_edges, &shared)?;
    wire.wire()
}

/// Finds the shared outer boundary by identity or coincident linear endpoints.
fn shared_outer_edge(first: &Face, second: &Face) -> Result<Option<topods::Edge>, Failure> {
    let first_edges = outer_edges(first)?;
    let second_edges = outer_edges(second)?;
    match_shared_edge(&first_edges, &second_edges)
}

/// Returns the unique topological or geometrically identical boundary edge.
fn shared_edge(first: &[topods::Edge], second: &[topods::Edge]) -> Result<topods::Edge, Failure> {
    match_shared_edge(first, second)?.ok_or_else(|| {
        Failure::with_type(
            "Standard_NotImplemented",
            "ShapeUpgrade_UnifySameDomain::Build requires exactly one shared boundary edge",
            "",
        )
    })
}

/// Returns the unique shared boundary, or `None` when the wires do not touch.
fn match_shared_edge(
    first: &[topods::Edge],
    second: &[topods::Edge],
) -> Result<Option<topods::Edge>, Failure> {
    let mut shared = None;
    for edge in first {
        for other in second {
            if !edges_share_boundary(edge, other)? {
                continue;
            }
            if shared.is_some() {
                return Err(Failure::with_type(
                    "Standard_NotImplemented",
                    "ShapeUpgrade_UnifySameDomain::Build requires exactly one shared boundary edge",
                    "",
                ));
            }
            shared = Some(edge.clone());
        }
    }
    Ok(shared)
}

/// Matches a shared edge identity or a linear segment with the same endpoints.
fn edges_share_boundary(first: &topods::Edge, second: &topods::Edge) -> Result<bool, Failure> {
    if first.is_same(second) {
        return Ok(true);
    }
    let first_curve = BRepAdaptorCurve::with_edge(first)?;
    let second_curve = BRepAdaptorCurve::with_edge(second)?;
    if first_curve.get_type()? != CurveType::Line || second_curve.get_type()? != CurveType::Line {
        return Ok(false);
    }
    coincident_line_endpoints(first, second)
}

/// Compares segment endpoints in either orientation using OCCT confusion tolerance.
fn coincident_line_endpoints(first: &topods::Edge, second: &topods::Edge) -> Result<bool, Failure> {
    let first = edge_endpoints(first)?;
    let second = edge_endpoints(second)?;
    let tolerance = precision::confusion();
    Ok(
        first.0.is_equal(&second.0, tolerance) && first.1.is_equal(&second.1, tolerance)
            || first.0.is_equal(&second.1, tolerance) && first.1.is_equal(&second.0, tolerance),
    )
}

/// Reads an edge's endpoint vertices using cumulative orientation.
fn edge_endpoints(edge: &topods::Edge) -> Result<(Point3, Point3), Failure> {
    let (first, last) = topexp::vertices(edge, true);
    Ok((BRepTool::pnt(&first)?, BRepTool::pnt(&last)?))
}

/// Adds every boundary edge except the shared edge to the new wire.
fn add_unshared_edges(
    wire: &mut BRepBuilderAPIMakeWire,
    edges: &[topods::Edge],
    shared: &topods::Edge,
) -> Result<(), Failure> {
    for edge in edges {
        if !edges_share_boundary(edge, shared)? {
            wire.add_edge(edge)?;
        }
    }
    Ok(())
}

/// Reads the unique non-internal wire's edge occurrences.
fn outer_edges(face: &Face) -> Result<Vec<topods::Edge>, Failure> {
    let outer = face_wires(face)?
        .into_iter()
        .find(|wire| wire.orientation() != rustycad_tkg3d::top_abs::Orientation::Internal)
        .ok_or_else(|| Failure::new("ShapeUpgrade_UnifySameDomain requires an outer wire"))?;
    Explorer::new(&outer.into_shape(), ShapeEnum::Edge, ShapeEnum::Shape)
        .and_then(|edges| edges.map(|edge| topods::edge(&edge)).collect())
}

/// Builds the merged face on the first face's support surface.
fn make_merged_face(first: &Face, boundary: &Wire) -> Result<Shape, Failure> {
    let surface = BRepTool::surface(first)?;
    let mut merged = BRepBuilderAPIMakeFace::from_surface_wire(&surface, boundary, true)?
        .face()?
        .into_shape();
    merged.set_orientation(first.orientation());
    Ok(merged)
}

/// Lists the face's direct wire children in stored order.
pub(super) fn face_wires(face: &Face) -> Result<Vec<Wire>, Failure> {
    ShapeIterator::new(&face.clone().into_shape(), true, true)
        .filter(|child| child.shape_type().ok() == Some(ShapeEnum::Wire))
        .map(|child| topods::wire(&child))
        .collect()
}

/// Applies a two-face replacement using the same recursive reshape path as OCCT.
fn replace_merged_pair(
    shape: &Shape,
    first: &Face,
    second: &Face,
    merged: &Shape,
) -> Result<Shape, Failure> {
    let context = Handle::new(ShapeBuildReShape::new());
    context.set_mode_consider_location(true);
    context.replace(&first.clone().into_shape(), merged)?;
    context.remove(&second.clone().into_shape())?;
    context.apply(shape, ShapeEnum::Shape)
}

/// Counts occurrences of `kind` for the bounded pair-merging loop.
fn count_shapes(shape: &Shape, kind: ShapeEnum) -> Result<usize, Failure> {
    Explorer::new(shape, kind, ShapeEnum::Shape).map(Iterator::count)
}

/// Whether no direct internal wire is present on a face.
fn has_no_internal_wires(face: &Face) -> Result<bool, Failure> {
    Ok(face_wires(face)?
        .iter()
        .all(|wire| wire.orientation() != rustycad_tkg3d::top_abs::Orientation::Internal))
}

/// Returns true when incident edge curves are not safely known to be non-collinear.
fn has_collinear_edge_pair(shape: &Shape, angular_tolerance: f64) -> Result<bool, Failure> {
    let mut vertex_edges = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(shape, ShapeEnum::Vertex, ShapeEnum::Edge, &mut vertex_edges)?;
    for index in 1..=vertex_edges.extent() {
        let edges = vertex_edges
            .find_from_index(index)?
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        for i in 0..edges.len() {
            for j in i + 1..edges.len() {
                if edges[i].is_same(&edges[j]) {
                    continue;
                }
                let first = topods::edge(&edges[i])?;
                let second = topods::edge(&edges[j])?;
                if curves_may_be_collinear(&first, &second, angular_tolerance)? {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

/// Tests whether two incident curves are collinear, conservatively retaining unknown types.
fn curves_may_be_collinear(
    first: &topods::Edge,
    second: &topods::Edge,
    tolerance: f64,
) -> Result<bool, Failure> {
    let first_curve = BRepAdaptorCurve::with_edge(first)?;
    let second_curve = BRepAdaptorCurve::with_edge(second)?;
    if first_curve.get_type()? != CurveType::Line || second_curve.get_type()? != CurveType::Line {
        return Ok(true);
    }
    let first_dir = first_curve.line()?.position().direction();
    let second_dir = second_curve.line()?.position().direction();
    let dot = first_dir.dot(second_dir);
    Ok((1.0 - dot.abs()).abs() <= tolerance)
}
