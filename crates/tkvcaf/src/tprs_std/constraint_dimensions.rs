// Rust port of ApplicationFramework/TKVCAF/TPrsStd/TPrsStd_ConstraintTools.cxx (the
// distance, radius, diameter, placement and round presentations) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The dimension presentations of the constraints: distance (`PrsDim_LengthDimension`),
//! radius, minor and major radius, diameter, placement (`PrsDim_OffsetDimension`) and round.
use super::constraint_tools::{
    ConstraintPresentation, TPrsStdConstraintTools, constraint_geometry, constraint_plane,
    first_edge, first_face, good_shape, is_face, live_constraint, no_presentation, one_shape,
    two_shapes,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkcaf::tdata_xtd::TDataXtdConstraint;
use rustycad_tkernel::standard::{Failure, Handle, exceptions};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gc::{MakePlane, Root};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkv3d::prs_dim::{
    PrsDimDiameterDimension, PrsDimDimensionSource, PrsDimLengthDimension,
    PrsDimMaxRadiusDimension, PrsDimMinRadiusDimension, PrsDimOffsetDimension,
    PrsDimRadiusDimension, PrsDimRelationMembers,
};

/// The shapes of a distance: one edge, or two shapes; `None` when the native code nullifies
/// the object.
struct DistanceShapes {
    first: Shape,
    second: Shape,
    count: i32,
}

/// The shapes of a distance constraint (`None` for a wrong count or a null shape).
fn distance_shapes(
    constraint: &Handle<TDataXtdConstraint>,
) -> Result<Option<DistanceShapes>, Failure> {
    let count = live_constraint(constraint)?.nb_geometries();
    if !(1..=2).contains(&count) {
        return Ok(None);
    }
    let (first, second) = if count == 1 {
        (one_shape(constraint)?, Shape::default())
    } else {
        two_shapes(constraint)?
    };
    if first.is_null() || (count == 2 && second.is_null()) {
        return Ok(None);
    }
    Ok(Some(DistanceShapes {
        first,
        second,
        count,
    }))
}

/// The kinds of shape pair of a two-shape distance.
#[derive(Default)]
struct PairKind {
    faces: bool,
    edges: bool,
    edge_face: bool,
    vertices: bool,
}

impl PairKind {
    fn of(first: &Shape, second: &Shape) -> Result<Self, Failure> {
        let (a, b) = (first.shape_type()?, second.shape_type()?);
        Ok(Self {
            faces: is_face(first)? && is_face(second)?,
            edges: a == ShapeEnum::Edge && b == ShapeEnum::Edge,
            edge_face: (a == ShapeEnum::Face && b == ShapeEnum::Edge)
                || (a == ShapeEnum::Edge && b == ShapeEnum::Face),
            vertices: a == ShapeEnum::Vertex && b == ShapeEnum::Vertex,
        })
    }
}

/// The non-planar search of `computeDistanceImpl` for two shapes of one other type: the
/// first wire vertex and the nearer end of the second wire, with the plane through the
/// first vertex whose normal is the X direction of the frame along the segment. Natively
/// two non-wire shapes leave the vertices null and `BRep_Tool::Pnt` fails.
fn nearest_wire_ends(shapes: &mut DistanceShapes) -> Result<Handle<GeomPlane>, Failure> {
    let (mut v1, mut v3, mut v4) = Default::default();
    if shapes.first.shape_type()? == ShapeEnum::Wire {
        (v1, _) = topexp::wire_vertices(&topods::wire(&shapes.first)?);
        (v3, v4) = topexp::wire_vertices(&topods::wire(&shapes.second)?);
    }
    shapes.first = Shape::from(v1.clone());
    let (p1, p2, p3) = (
        BRepTool::pnt(&v1)?,
        BRepTool::pnt(&v3)?,
        BRepTool::pnt(&v4)?,
    );
    let (vertex, toward) = if p1.distance(&p2) < p1.distance(&p3) {
        (v3, p2)
    } else {
        (v4, p3)
    };
    shapes.second = Shape::from(vertex);
    let frame = Axis2::new(p1, Direction3::from_xyz(toward.xyz() - p1.xyz())?)?;
    Ok(Handle::new(GeomPlane::from_point(p1, frame.x_direction())?))
}

/// The plane through three points of two vertices (the third point is the native
/// `(P1.Y() - 1, P2.X() + 1, 0)`) or of two edges (both ends of the first, the first end of
/// the second); null when `GC_MakePlane` fails.
fn plane_through_shapes(
    shapes: &DistanceShapes,
    kind: &PairKind,
) -> Result<Handle<GeomPlane>, Failure> {
    let (p1, p2, p3) = if kind.vertices {
        let p1 = BRepTool::pnt(&topods::vertex(&shapes.first)?)?;
        let p2 = BRepTool::pnt(&topods::vertex(&shapes.second)?)?;
        (p1, p2, Point3::new(p1.y() - 1.0, p2.x() + 1.0, 0.0))
    } else {
        let (a, b) = topexp::vertices(&topods::edge(&shapes.first)?, false);
        let (c, _) = topexp::vertices(&topods::edge(&shapes.second)?, false);
        (BRepTool::pnt(&a)?, BRepTool::pnt(&b)?, BRepTool::pnt(&c)?)
    };
    let maker = MakePlane::through_points(p1, p2, p3)?;
    if !maker.is_done() {
        return Ok(Handle::default());
    }
    maker.value()
}

impl TPrsStdConstraintTools {
    /// `ComputeDistance`: a `PrsDim_LengthDimension` of one edge, of two faces or of two
    /// shapes, with the constraint's (or a computed) plane as custom plane.
    pub fn compute_distance(constraint: &Handle<TDataXtdConstraint>) -> ConstraintPresentation {
        let Some(mut shapes) = distance_shapes(constraint)? else {
            return no_presentation();
        };
        let planar = live_constraint(constraint)?.is_planar();
        let mut plane = Handle::default();
        if planar {
            let geometry = constraint_geometry(constraint)?;
            good_shape(&mut shapes.first)?;
            if shapes.count == 2 {
                good_shape(&mut shapes.second)?;
            }
            plane = geometry.cast::<GeomPlane>();
        }
        let (mut value, mut text) = (0.0, ExtendedString::default());
        Self::compute_text_and_value(constraint, &mut value, &mut text, false)?;
        let mut kind = PairKind::default();
        if shapes.count == 1 {
            if shapes.first.shape_type()? != ShapeEnum::Edge {
                return no_presentation();
            }
        } else {
            kind = PairKind::of(&shapes.first, &shapes.second)?;
            if !planar && !kind.faces && !kind.edges && !kind.vertices {
                if shapes.first.shape_type()? == shapes.second.shape_type()? {
                    plane = nearest_wire_ends(&mut shapes)?;
                } else if !kind.edge_face {
                    return no_presentation();
                }
            }
        }
        if (kind.vertices || kind.edges) && !planar {
            let through = plane_through_shapes(&shapes, &kind)?;
            if !through.is_null() {
                plane = through;
            }
        }
        if (!kind.faces || planar) && plane.is_null() {
            return no_presentation();
        }
        length_dimension(&shapes, &kind, &plane)
    }
}

/// The new `PrsDim_LengthDimension` of `computeDistanceImpl`, with the plane as custom
/// plane when there is one.
fn length_dimension(
    shapes: &DistanceShapes,
    kind: &PairKind,
    plane: &Handle<GeomPlane>,
) -> ConstraintPresentation {
    let pln = plane.get().map(GeomPlane::pln);
    let dimension = if shapes.count == 1 {
        let pln = pln.ok_or_else(|| null_plane("ComputeDistance"))?;
        PrsDimLengthDimension::from_edge(&first_edge(&shapes.first)?, &pln)?
    } else if kind.faces {
        PrsDimLengthDimension::from_faces(
            &first_face(&shapes.first)?,
            &first_face(&shapes.second)?,
        )?
    } else {
        let pln = pln.ok_or_else(|| null_plane("ComputeDistance"))?;
        PrsDimLengthDimension::from_shapes(&shapes.first, &shapes.second, &pln)?
    };
    if let Some(pln) = pln {
        dimension.set_custom_plane(&pln);
    }
    Ok(dimension.cast())
}

/// `aPlane->Pln()` on a null plane (a null dereference natively).
fn null_plane(member: &str) -> Failure {
    exceptions::null_object(&format!(
        "TPrsStd_ConstraintTools::{member}: null plane; expected a Geom_Plane"
    ))
}

/// The single shape of a radius-like constraint: not a compound, compsolid, solid or shell;
/// its first face when it has one, its edge or vertex when planar. `None` when the native
/// code nullifies the object.
fn radius_shape(constraint: &Handle<TDataXtdConstraint>) -> Result<Option<Shape>, Failure> {
    if live_constraint(constraint)?.nb_geometries() < 1 {
        return Ok(None);
    }
    let mut shape = one_shape(constraint)?;
    if shape.is_null() {
        return Ok(None);
    }
    let solid_like = [
        ShapeEnum::Compound,
        ShapeEnum::CompSolid,
        ShapeEnum::Solid,
        ShapeEnum::Shell,
    ];
    if solid_like.contains(&shape.shape_type()?) {
        return Ok(None);
    }
    if is_face(&shape)? {
        shape = first_face(&shape)?.into_shape();
    }
    Ok(Some(shape))
}

/// The radius kinds that share `computeRadiusImpl`'s structure.
enum RadiusKind {
    Radius,
    Minor,
    Major,
}

impl TPrsStdConstraintTools {
    /// `ComputeRadius`: a `PrsDim_RadiusDimension` (custom plane of a planar constraint).
    pub fn compute_radius(constraint: &Handle<TDataXtdConstraint>) -> ConstraintPresentation {
        radius_presentation(constraint, RadiusKind::Radius)
    }
    /// `ComputeMinRadius`: a `PrsDim_MinRadiusDimension` with the value and text.
    pub fn compute_min_radius(constraint: &Handle<TDataXtdConstraint>) -> ConstraintPresentation {
        radius_presentation(constraint, RadiusKind::Minor)
    }
    /// `ComputeMaxRadius`: a `PrsDim_MaxRadiusDimension` with the value and text.
    pub fn compute_max_radius(constraint: &Handle<TDataXtdConstraint>) -> ConstraintPresentation {
        radius_presentation(constraint, RadiusKind::Major)
    }
}

/// `computeRadiusImpl`, `computeMinRadiusImpl` and `computeMaxRadiusImpl`.
fn radius_presentation(
    constraint: &Handle<TDataXtdConstraint>,
    kind: RadiusKind,
) -> ConstraintPresentation {
    let Some(mut shape) = radius_shape(constraint)? else {
        return no_presentation();
    };
    let (mut value, mut text) = (0.0, ExtendedString::default());
    TPrsStdConstraintTools::compute_text_and_value(constraint, &mut value, &mut text, false)?;
    let planar = live_constraint(constraint)?.is_planar();
    if planar {
        good_shape(&mut shape)?;
    }
    let object: Handle<dyn rustycad_tkv3d::ais::AisInteractiveObject> = match kind {
        RadiusKind::Radius => PrsDimRadiusDimension::from_shape(&shape)?.cast(),
        RadiusKind::Minor => PrsDimMinRadiusDimension::new(&shape, value, &text).cast(),
        RadiusKind::Major => PrsDimMaxRadiusDimension::new(&shape, value, &text).cast(),
    };
    if !planar {
        return Ok(object);
    }
    let plane = constraint_plane(constraint)?;
    let Some(plane_ref) = plane.get() else {
        return no_presentation();
    };
    match kind {
        RadiusKind::Radius => object
            .cast::<PrsDimRadiusDimension>()
            .get()
            .map(|d| d.set_custom_plane(&plane_ref.pln())),
        RadiusKind::Minor => object
            .cast::<PrsDimMinRadiusDimension>()
            .get()
            .map(|r| r.set_plane(&plane)),
        RadiusKind::Major => object
            .cast::<PrsDimMaxRadiusDimension>()
            .get()
            .map(|r| r.set_plane(&plane)),
    };
    Ok(object)
}

impl TPrsStdConstraintTools {
    /// `ComputeDiameter`: a `PrsDim_DiameterDimension` (a planar constraint needs a plane,
    /// which is not used: the native `SetCustomPlane` call is commented out).
    pub fn compute_diameter(constraint: &Handle<TDataXtdConstraint>) -> ConstraintPresentation {
        if live_constraint(constraint)?.nb_geometries() < 1 {
            return no_presentation();
        }
        let mut shape = one_shape(constraint)?;
        if shape.is_null() {
            return no_presentation();
        }
        let (mut value, mut text) = (0.0, ExtendedString::default());
        Self::compute_text_and_value(constraint, &mut value, &mut text, false)?;
        let planar = live_constraint(constraint)?.is_planar();
        if planar {
            good_shape(&mut shape)?;
        }
        let dimension = PrsDimDiameterDimension::from_shape(&shape)?;
        if planar && constraint_plane(constraint)?.is_null() {
            return no_presentation();
        }
        Ok(dimension.cast())
    }
    /// `ComputePlacement`: a `PrsDim_OffsetDimension` of the first faces of both shapes
    /// (arrow size value / 20), null when a shape has no face; `Standard_ProgramError`
    /// with fewer than two geometries.
    pub fn compute_placement(constraint: &Handle<TDataXtdConstraint>) -> ConstraintPresentation {
        let value_ref = live_constraint(constraint)?;
        if value_ref.nb_geometries() < 2 {
            return Err(exceptions::program_error(
                "TPrsStd_ConstraintTools::ComputePlacement: at leat two constraints are needed",
            ));
        }
        let (first, second) = two_shapes(constraint)?;
        if first.is_null() || second.is_null() {
            return no_presentation();
        }
        let (mut value, mut text) = (0.0, ExtendedString::from_utf8(" "));
        if value_ref.is_dimension() {
            Self::compute_text_and_value(constraint, &mut value, &mut text, false)?;
        }
        let (face1, face2) = (first_face(&first)?, first_face(&second)?);
        let dimension = PrsDimOffsetDimension::new(
            &face1.clone().into_shape(),
            &face2.clone().into_shape(),
            value,
            &text,
        );
        dimension.set_arrow_size(value / 20.0);
        if face1.is_null() || face2.is_null() {
            return no_presentation();
        }
        Ok(dimension.cast())
    }
    /// `ComputeRound`: a `PrsDim_RadiusDimension` of the first geometry; null when its
    /// construction fails (the native `OCC_CATCH_SIGNALS` block).
    pub fn compute_round(constraint: &Handle<TDataXtdConstraint>) -> ConstraintPresentation {
        if live_constraint(constraint)?.nb_geometries() < 1 {
            return no_presentation();
        }
        let shape = one_shape(constraint)?;
        if shape.is_null() {
            return no_presentation();
        }
        let (mut value, mut text) = (0.0, ExtendedString::default());
        Self::compute_text_and_value(constraint, &mut value, &mut text, false)?;
        match PrsDimRadiusDimension::from_shape(&shape) {
            Ok(dimension) => Ok(dimension.cast()),
            Err(_) => no_presentation(),
        }
    }
}
