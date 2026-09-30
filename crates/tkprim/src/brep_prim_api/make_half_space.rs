// Rust translation of BRepPrimAPI_MakeHalfSpace.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepPrimAPI_MakeHalfSpace`: the maker of half-spaces bounded by a face or a shell.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_lprop::BRepLPropSLProps;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Shell, Solid, Vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{self, Direction3, Point3, Xyz};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase, BRepBuilderAPIMakeVertex};
use rustycad_tktopalgo::brep_extrema::{BRepExtremaDistShapeShape, BRepExtremaSupportType};

/// `getNormalOnFace`: the normal of the face at (u, v) (BRepLProp_SLProps of order 2
/// and resolution gp::Resolution()), reversed for a REVERSED face; LProp_NotDefined
/// when the normal is not defined.
fn normal_on_face(face: &Face, u: f64, v: f64) -> Result<Direction3, Failure> {
    let surface = BRepAdaptorSurface::with_face(face, true)?;
    let mut props = BRepLPropSLProps::with_parameters(&surface, u, v, 2, gp::resolution())?;
    let mut normal = props.normal()?;
    if face.orientation() == Orientation::Reversed {
        normal.reverse();
    }
    Ok(normal)
}

/// The direction of a sum of normals, when it is long enough (`theNormal = aSum`).
fn summed_direction(sum: Xyz) -> Result<Option<Direction3>, Failure> {
    if sum.square_modulus() > gp::resolution() {
        return Ok(Some(Direction3::from_xyz(sum)?));
    }
    Ok(None)
}

/// The faces of `shape` in explorer order, each with every occurrence of a sub-shape
/// of kind `kind` that is the same as `target`.
fn face_occurrences(shape: &Shape, target: &Shape, kind: ShapeEnum) -> Result<Vec<Face>, Failure> {
    let mut found = Vec::new();
    let mut faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?;
    while faces.more() {
        let face = topods::face(faces.current())?;
        let mut subs = Explorer::new(&face, kind, ShapeEnum::Shape)?;
        while subs.more() {
            if subs.current().is_same(target) {
                found.push(face.clone());
            }
            subs.next()?;
        }
        faces.next()?;
    }
    Ok(found)
}

/// `getNormalFromEdge`: the sum of the normals of the faces of `shape` holding `edge`,
/// at the point of parameter `par` of its p-curve on each.
fn normal_from_edge(shape: &Shape, edge: &Edge, par: f64) -> Result<Option<Direction3>, Failure> {
    let mut sum = Xyz::default();
    for face in face_occurrences(shape, edge, ShapeEnum::Edge)? {
        let pcurve = BRepTool::curve_on_face(edge, &face)?.pcurve;
        let Some(curve) = pcurve.get() else {
            // A null p-curve is dereferenced natively (a crash).
            return Err(Failure::with_message(
                "Standard_NullObject",
                "BRepPrimAPI_MakeHalfSpace: edge without p-curve on its face; expected a p-curve",
            ));
        };
        let p2d = curve.value(par)?;
        sum += normal_on_face(&face, p2d.x(), p2d.y())?.xyz();
    }
    summed_direction(sum)
}

/// `getNormalFromVertex`: the sum of the normals of the faces of `shape` holding
/// `vertex`, at its parameters on each.
fn normal_from_vertex(shape: &Shape, vertex: &Vertex) -> Result<Option<Direction3>, Failure> {
    let mut sum = Xyz::default();
    for face in face_occurrences(shape, vertex, ShapeEnum::Vertex)? {
        let p2d = BRepTool::parameters(vertex, &face)?;
        sum += normal_on_face(&face, p2d.x(), p2d.y())?.xyz();
    }
    summed_direction(sum)
}

/// The nearest point of an edge or vertex solution with a defined normal.
fn boundary_extremum(ext: &BRepExtremaDistShapeShape, shape: &Shape, iext: i32) -> Result<Option<(Point3, Direction3)>, Failure> {
    let normal = match ext.support_type_shape2(iext)? {
        BRepExtremaSupportType::IsOnEdge => {
            let par = ext.par_on_edge_s2(iext)?;
            let edge = topods::edge(&ext.support_on_shape2(iext)?)?;
            normal_from_edge(shape, &edge, par)?
        }
        BRepExtremaSupportType::IsVertex => {
            let vertex = topods::vertex(&ext.support_on_shape2(iext)?)?;
            normal_from_vertex(shape, &vertex)?
        }
        BRepExtremaSupportType::IsInFace => return Ok(None),
    };
    let min_pnt = ext.point_on_shape2(iext)?;
    Ok(normal.map(|normal| (min_pnt, normal)))
}

/// `FindExtrema`: the point of `shape` nearest to `pnt` and the normal there, taken
/// from the first solution inside a face, else from the first edge or vertex solution
/// with a defined normal.
fn find_extrema(pnt: Point3, shape: &Shape) -> Result<Option<(Point3, Direction3)>, Failure> {
    let ref_vertex: Shape = BRepBuilderAPIMakeVertex::new(pnt)?.vertex()?.into();
    let ext = BRepExtremaDistShapeShape::with_shapes(&ref_vertex, shape, Default::default())?;
    if !ext.is_done() || ext.nb_solution() == 0 {
        return Ok(None);
    }
    let nbext = ext.nb_solution();
    for iext in 1..=nbext {
        if ext.support_type_shape2(iext)? == BRepExtremaSupportType::IsInFace {
            let face = topods::face(&ext.support_on_shape2(iext)?)?;
            let min_pnt = ext.point_on_shape2(iext)?;
            let (u, v) = ext.par_on_face_s2(iext)?;
            return Ok(Some((min_pnt, normal_on_face(&face, u, v)?)));
        }
    }
    for iext in 1..=nbext {
        if let Some(found) = boundary_extremum(&ext, shape, iext)? {
            return Ok(Some(found));
        }
    }
    Ok(None)
}

/// `isOutside`: the reference point is on the side of the normal at the nearest point;
/// Standard_ConstructionError when the two points coincide.
fn is_outside(pnt: Point3, ponf: Point3, normal: Direction3) -> Result<bool, Failure> {
    let opp_ref = Direction3::from_xyz(pnt.xyz() - ponf.xyz())?;
    Ok(normal.dot(opp_ref) > 0.)
}

/// `BRepPrimAPI_MakeHalfSpace`: the solid bounded by a face or a shell on the side of a
/// reference point: the shell is reversed when the point lies on the side of the normal
/// at the nearest point of the boundary. The maker is not done when no nearest point
/// with a normal is found (docs/prim-design.md section 4).
///
/// ```
/// use rustycad_tkmath::gp::{Direction3, Plane, Point3};
/// use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeHalfSpace;
/// use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape};
/// let plane = Plane::new(Point3::new(0.0, 0.0, 0.0), Direction3::new(0.0, 0.0, 1.0)?);
/// let face = BRepBuilderAPIMakeFace::with_plane_bounds(plane, -1.0, 1.0, -1.0, 1.0)?.face()?;
/// let maker = BRepPrimAPIMakeHalfSpace::with_face(&face, Point3::new(0.0, 0.0, 1.0))?;
/// assert!(maker.is_done());
/// assert!(!maker.solid()?.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct BRepPrimAPIMakeHalfSpace {
    base: BRepBuilderAPIMakeShapeBase,
    my_solid: Solid,
}

impl BRepPrimAPIMakeHalfSpace {
    /// The maker of the half-space of `shell` (holding `boundary`) seen from `ref_pnt`.
    fn bounded(boundary: &Shape, ref_pnt: Point3, make_shell: impl FnOnce() -> Result<Shell, Failure>) -> Result<Self, Failure> {
        let mut maker = Self { base: BRepBuilderAPIMakeShapeBase::new(), my_solid: Solid::default() };
        maker.base.set_not_done();
        let Some((min_pnt, normal)) = find_extrema(ref_pnt, boundary)? else {
            return Ok(maker);
        };
        let to_reverse = is_outside(ref_pnt, min_pnt, normal)?;
        let mut shell = make_shell()?;
        let builder = BRepBuilder::default();
        builder.make_solid(&mut maker.my_solid);
        if to_reverse {
            shell.reverse();
        }
        builder.add(&mut maker.my_solid, &shell)?;
        maker.base.my_shape = maker.my_solid.clone().into();
        maker.base.set_done();
        Ok(maker)
    }

    /// `BRepPrimAPI_MakeHalfSpace(Face, RefPnt)`: the half-space bounded by a new shell
    /// holding `face`; `BRepPrimAPIMakeHalfSpace::with_face(&face, p)?`.
    pub fn with_face(face: &Face, ref_pnt: Point3) -> Result<Self, Failure> {
        Self::bounded(face, ref_pnt, || {
            let builder = BRepBuilder::default();
            let mut shell = Shell::default();
            builder.make_shell(&mut shell);
            builder.add(&mut shell, face)?;
            Ok(shell)
        })
    }

    /// `BRepPrimAPI_MakeHalfSpace(Shell, RefPnt)`: the half-space bounded by `shell`;
    /// `BRepPrimAPIMakeHalfSpace::with_shell(&shell, p)?`.
    pub fn with_shell(shell: &Shell, ref_pnt: Point3) -> Result<Self, Failure> {
        Self::bounded(shell, ref_pnt, || Ok(shell.clone()))
    }

    /// `Solid()` and `operator TopoDS_Solid()`: the half-space; StdFail_NotDone when the
    /// maker is not done; `maker.solid()?`.
    pub fn solid(&self) -> Result<&Solid, Failure> {
        if !self.is_done() {
            return Err(Failure::with_message("StdFail_NotDone", "BRepPrimAPI_MakeHalfSpace::Solid"));
        }
        Ok(&self.my_solid)
    }
}

impl BRepBuilderAPIMakeShape for BRepPrimAPIMakeHalfSpace {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }
}
