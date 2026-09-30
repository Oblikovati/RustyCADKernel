// Rust translation of BRepBndLib::AddOBB and its file-static helpers
// (BRepBndLib_1.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBndLib::AddOBB`: the oriented box of a shape, by DiTO over its vertices and mesh
//! nodes when the shape is polyhedral enough (`CheckPoints`), else by the axis-aligned
//! box of the shape moved to its axes of inertia (`ComputePCA`).
use super::BRepBndLib;
use super::add::located;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::{BoundingBox3, OrientedBoundingBox};
use rustycad_tkmath::geom_abs::{CurveType, Shape as Continuity, SurfaceType};
use rustycad_tkmath::gp::{Axis3, Direction3, Point3, Transform3, TransformForm, Xyz};
use rustycad_tkmath::poly::mesh_purpose;
use rustycad_tkmath::toploc::Location;

impl BRepBndLib {
    /// `AddOBB(S, OBB, isTriangulationUsed, isOptimal, isShapeToleranceUsed)`: extends
    /// `obb` to hold `shape`. When every vertex, the nodes of the meshes of the curved
    /// faces and the 3d polygons of the curved free edges can stand for the shape (the
    /// meshes only with `use_triangulation`), DiTO rebuilds the box over them and the
    /// eight corners of a non-void `obb` (enlarged by the vertex tolerances and mesh
    /// deflections with `use_shape_tolerance`, `optimal` selecting the optimal DiTO);
    /// otherwise the box is the axis-aligned box (`AddOptimal` with `optimal`, else
    /// `Add`) of the shape expressed in its principal axes of inertia. Exceptions of the
    /// adaptors, of BRepGProp and of the frame construction are returned as errors;
    /// `BRepBndLib::add_obb(&shape, &mut obb, true, false, true)?`.
    pub fn add_obb(
        shape: &Shape,
        obb: &mut OrientedBoundingBox,
        use_triangulation: bool,
        optimal: bool,
        use_shape_tolerance: bool,
    ) -> Result<(), Failure> {
        let flags = ObbFlags {
            use_triangulation,
            optimal,
            use_shape_tolerance,
        };
        if check_points(shape, flags, obb)? {
            return Ok(());
        }
        compute_pca(shape, obb, flags)
    }
}

/// The three flags of `AddOBB`, passed together to its helpers.
#[derive(Clone, Copy)]
struct ObbFlags {
    use_triangulation: bool,
    optimal: bool,
    use_shape_tolerance: bool,
}

/// `IsLinear`: a line, a B-spline or Bezier curve of degree 1 that is not C0, or an
/// offset of such a curve.
fn is_linear(curve: &dyn Adaptor3dCurve) -> Result<bool, Failure> {
    let kind: CurveType = curve.get_type()?;
    if kind == CurveType::OffsetCurve {
        let offset = curve.offset_curve()?;
        let basis = offset.get().map(|o| o.basis_curve()).unwrap_or_default();
        return is_linear(&GeomAdaptorCurve::with_curve(&basis)?);
    }
    if kind == CurveType::BSplineCurve || kind == CurveType::BezierCurve {
        // Curves with C0-continuity and degree 1 could be sets of points (native note).
        return Ok(curve.degree()? == 1 && curve.continuity()? != Continuity::C0);
    }
    Ok(kind == CurveType::Line)
}

/// `IsPlanar`: a plane, an extrusion of a linear curve, a B-spline or Bezier surface of
/// degrees 1 that is not C0 in either direction, or an offset of such a surface.
fn is_planar(surface: &dyn Adaptor3dSurface) -> Result<bool, Failure> {
    let kind: SurfaceType = surface.get_type()?;
    match kind {
        SurfaceType::OffsetSurface => is_planar(basis_of(&surface.basis_surface()?)?),
        SurfaceType::SurfaceOfExtrusion => is_linear(basis_of(&surface.basis_curve()?)?),
        SurfaceType::BSplineSurface | SurfaceType::BezierSurface => is_bilinear_patch(surface),
        _ => Ok(kind == SurfaceType::Plane),
    }
}

/// The adaptor a basis handle holds; the native code dereferences it unchecked, so a
/// null handle is Standard_NullObject here.
fn basis_of<T: ?Sized + RuntimeTyped>(handle: &Handle<T>) -> Result<&T, Failure> {
    handle.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "BRepBndLib::AddOBB: null basis adaptor; expected the basis of an offset or extrusion",
            "",
        )
    })
}

/// A B-spline or Bezier surface of degrees (1, 1), not C0 in U nor in V.
fn is_bilinear_patch(surface: &dyn Adaptor3dSurface) -> Result<bool, Failure> {
    if surface.u_degree()? != 1 || surface.v_degree()? != 1 {
        return Ok(false);
    }
    // Surfaces with C0-continuity and degree 1 could be sets of points (native note).
    Ok(surface.u_continuity()? != Continuity::C0 && surface.v_continuity()? != Continuity::C0)
}

/// The points `PointsForOBB` gathers, with the tolerance of each (the vertex
/// tolerance or the deflection of the mesh or polygon the node comes from).
#[derive(Default)]
struct ObbPoints {
    points: Vec<Point3>,
    tolerances: Vec<f64>,
}

impl ObbPoints {
    fn push(&mut self, point: Point3, tolerance: f64) {
        self.points.push(point);
        self.tolerances.push(tolerance);
    }
}

/// `PointsForOBB`: the vertices of `shape`, then the mesh nodes of each face that is
/// not planar with linear edges, then the 3d polygon nodes of each free edge that is
/// not linear; `None` (the native 0) when the shape has no vertex, or when a face or
/// edge needs a mesh or polygon that is not allowed or missing. The native function
/// runs twice (count, then fill) over the same exploration; one pass gives the same
/// points. Infinite faces and edges are not handled (native note).
fn points_for_obb(shape: &Shape, use_triangulation: bool) -> Result<Option<ObbPoints>, Failure> {
    let mut gathered = ObbPoints::default();
    for vertex in Explorer::new(shape, ShapeEnum::Vertex, ShapeEnum::Shape)? {
        let vertex = topods::vertex(&vertex)?;
        gathered.push(
            BRepTool::pnt(&vertex)?,
            BRepTool::vertex_tolerance(&vertex)?,
        );
    }
    if gathered.points.is_empty() {
        return Ok(None);
    }
    for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        if !add_face_nodes(&topods::face(&face)?, use_triangulation, &mut gathered)? {
            return Ok(None);
        }
    }
    for edge in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Face)? {
        if !add_free_edge_nodes(&topods::edge(&edge)?, use_triangulation, &mut gathered)? {
            return Ok(None);
        }
    }
    Ok(Some(gathered))
}

/// One face of `PointsForOBB`: nothing for a planar face whose geometric edges are
/// linear (its vertices stand for it), else the nodes of its mesh; false (the native
/// `return 0`) when the mesh is needed but not allowed or absent.
fn add_face_nodes(
    face: &Face,
    use_triangulation: bool,
    gathered: &mut ObbPoints,
) -> Result<bool, Failure> {
    let surface = BRepAdaptorSurface::with_face(face, false)?;
    if is_planar(&surface)? {
        match first_curved_edge(face)? {
            None => return Ok(true),
            Some(()) if !use_triangulation => return Ok(false),
            Some(()) => {}
        }
    } else if !use_triangulation {
        return Ok(false);
    }
    let (mesh, location) = BRepTool::triangulation(face, mesh_purpose::NONE)?;
    let Some(mesh) = mesh.get() else {
        return Ok(false);
    };
    let transform: Transform3 = location.transformation();
    for i in 1..=mesh.nb_nodes() {
        let node = mesh.node(i)?;
        let point = match transform.form() == TransformForm::Identity {
            true => node,
            false => node.transformed(transform),
        };
        gathered.push(point, mesh.deflection());
    }
    Ok(true)
}

/// The first geometric edge of a planar face whose curve is not linear (the native
/// edge loop stops there), or `None` when every geometric edge is linear.
fn first_curved_edge(face: &Face) -> Result<Option<()>, Failure> {
    for edge in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edge = topods::edge(&edge)?;
        if BRepTool::is_geometric_edge(&edge)? && !is_linear(&BRepAdaptorCurve::with_edge(&edge)?)?
        {
            return Ok(Some(()));
        }
    }
    Ok(None)
}

/// One edge outside a face of `PointsForOBB`: nothing for a linear geometric edge,
/// else the nodes of its 3d polygon; false when the polygon is not allowed or absent.
fn add_free_edge_nodes(
    edge: &Edge,
    use_triangulation: bool,
    gathered: &mut ObbPoints,
) -> Result<bool, Failure> {
    if BRepTool::is_geometric_edge(edge)? && is_linear(&BRepAdaptorCurve::with_edge(edge)?)? {
        return Ok(true);
    }
    if !use_triangulation {
        return Ok(false);
    }
    let (polygon, location) = BRepTool::polygon3d(edge)?;
    let Some(polygon) = polygon.get() else {
        return Ok(false);
    };
    let nodes = polygon.nodes();
    for i in 1..=polygon.nb_nodes() {
        gathered.push(located(nodes[i], &location), polygon.deflection());
    }
    Ok(true)
}

/// `CheckPoints`: DiTO over the points of `PointsForOBB` and the corners of a non-void
/// `obb` (zero tolerance); false when the shape gives no points or the box stays void.
fn check_points(
    shape: &Shape,
    flags: ObbFlags,
    obb: &mut OrientedBoundingBox,
) -> Result<bool, Failure> {
    let Some(mut gathered) = points_for_obb(shape, flags.use_triangulation)? else {
        return Ok(false);
    };
    if let Some(corners) = obb.vertices() {
        // All points of the old OBB have zero tolerance (native note).
        corners
            .into_iter()
            .for_each(|corner| gathered.push(corner, 0.0));
    }
    let points = Array1::from_shared(&gathered.points, 0);
    let tolerances = Array1::from_shared(&gathered.tolerances, 0);
    let tolerances = flags.use_shape_tolerance.then_some(&tolerances);
    obb.rebuild(&points, tolerances, flags.optimal)?;
    Ok(!obb.is_void())
}

/// `ComputeProperties`: the volume properties of the solids, the surface properties of
/// the faces outside a solid, the linear properties of the edges outside a face and the
/// points of the vertices outside an edge, each added. The native calls pass one
/// `true` positional flag: for `VolumeProperties` it is `OnlyClosed` (so a solid adds
/// only its closed shells, shared faces counted twice), for the other two `SkipShared`.
fn compute_properties(shape: &Shape) -> Result<rustycad_tkgeombase::gprop::GProps, Failure> {
    use crate::brep_gprop::BRepGProp;
    use rustycad_tkgeombase::gprop::GProps;
    let mut common = GProps::new();
    for solid in Explorer::new(shape, ShapeEnum::Solid, ShapeEnum::Shape)? {
        let mut props = GProps::new();
        BRepGProp::volume_properties(&solid, &mut props, true, false, false)?;
        common.add(&props, 1.0)?;
    }
    for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Solid)? {
        let mut props = GProps::new();
        BRepGProp::surface_properties(&face, &mut props, true, false)?;
        common.add(&props, 1.0)?;
    }
    for edge in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Face)? {
        let mut props = GProps::new();
        BRepGProp::linear_properties(&edge, &mut props, true, false)?;
        common.add(&props, 1.0)?;
    }
    for vertex in Explorer::new(shape, ShapeEnum::Vertex, ShapeEnum::Edge)? {
        let props = GProps::with_location(BRepTool::pnt(&topods::vertex(&vertex)?)?);
        common.add(&props, 1.0)?;
    }
    Ok(common)
}

/// `IsWCS`: 3, 2 or 1 when `direction` is the Z, Y or X axis of the world (up to
/// the square of the angular precision, tested in that order), else 0.
fn is_wcs(direction: Direction3) -> i32 {
    // tol: the native `Precision::Angular() * Precision::Angular()`.
    let tolerance: f64 = precision::angular() * precision::angular();
    let (x, y, z) = (direction.x(), direction.y(), direction.z());
    let (vx, vy, vz) = (y * y + z * z, x * x + z * z, x * x + y * y);
    if vz < tolerance {
        return 3;
    }
    if vy < tolerance {
        return 2;
    }
    if vx < tolerance {
        return 1;
    }
    0
}

/// The transformation into the principal frame of inertia of `shape` (centre of mass,
/// third axis as normal, first axis as X direction), or the identity when the first
/// two axes are both world axes.
fn principal_frame(shape: &Shape) -> Result<Transform3, Failure> {
    let common = compute_properties(shape)?;
    let principal = common.principal_properties()?;
    let first = Direction3::from_vector(principal.first_axis_of_inertia())?;
    let second = Direction3::from_vector(principal.second_axis_of_inertia())?;
    let mut transform = Transform3::default();
    if is_wcs(first) == 0 || is_wcs(second) == 0 {
        let third = Direction3::from_vector(principal.third_axis_of_inertia())?;
        let frame = Axis3::new(common.centre_of_mass(), third, first)?;
        transform.set_transformation_to(frame);
    }
    Ok(transform)
}

/// The axis-aligned box of `ComputePCA` (`AddOptimal` or `Add`).
fn shape_box(shape: &Shape, flags: ObbFlags) -> Result<BoundingBox3, Failure> {
    let mut bounds = BoundingBox3::default();
    if flags.optimal {
        BRepBndLib::add_optimal(
            shape,
            &mut bounds,
            flags.use_triangulation,
            flags.use_shape_tolerance,
        )?;
    } else {
        BRepBndLib::add(shape, &mut bounds, true)?;
    }
    Ok(bounds)
}

/// The box `ComputePCA` builds: centre, the three axes (columns of the inverse
/// transformation) and the half sizes of the axis-aligned box.
struct PcaBox {
    center: Xyz,
    axes: [Xyz; 3],
    half_sizes: Xyz,
}

/// `ComputePCA`: the axis-aligned box of `shape` moved to its principal frame, set on a
/// void `obb` or merged with its corners by `ReBuild`; nothing when that box is void.
fn compute_pca(
    shape: &Shape,
    obb: &mut OrientedBoundingBox,
    flags: ObbFlags,
) -> Result<(), Failure> {
    let transform = principal_frame(shape)?;
    let is_identity = transform.form() == TransformForm::Identity;
    let moved = match is_identity {
        true => shape.clone(),
        false => shape.moved(&Location::new(transform), false)?,
    };
    let bounds = shape_box(&moved, flags)?;
    if bounds.is_void() {
        return Ok(());
    }
    let pca = pca_box(&bounds, transform)?;
    if obb.is_void() {
        return set_pca_box(obb, &pca, is_identity);
    }
    rebuild_with_pca_box(obb, &pca)
}

/// The centre and half diagonal of `bounds` (gap included), moved back to the world by
/// the inverse of `transform`.
fn pca_box(bounds: &BoundingBox3, mut transform: Transform3) -> Result<PcaBox, Failure> {
    let (min, max) = (bounds.corner_min()?.xyz(), bounds.corner_max()?.xyz());
    let mut center: Xyz = (min + max) / 2.;
    let half_sizes: Xyz = (max - min) / 2.;
    let mut axes = [
        Xyz::new(1., 0., 0.),
        Xyz::new(0., 1., 0.),
        Xyz::new(0., 0., 1.),
    ];
    if transform.form() != TransformForm::Identity {
        transform.invert()?;
        center = transform.apply_coordinates(center);
        let matrix = transform.matrix();
        axes = [matrix.column(1)?, matrix.column(2)?, matrix.column(3)?];
    }
    Ok(PcaBox {
        center,
        axes,
        half_sizes,
    })
}

/// A void `obb` takes the PCA box (each axis normalized as the native `gp_Dir`
/// argument); it is axis-aligned when the frame was the identity.
fn set_pca_box(
    obb: &mut OrientedBoundingBox,
    pca: &PcaBox,
    is_identity: bool,
) -> Result<(), Failure> {
    obb.set_center(Point3::from(pca.center));
    obb.set_x_component(Direction3::from_xyz(pca.axes[0])?, pca.half_sizes.x());
    obb.set_y_component(Direction3::from_xyz(pca.axes[1])?, pca.half_sizes.y());
    obb.set_z_component(Direction3::from_xyz(pca.axes[2])?, pca.half_sizes.z());
    obb.set_aa_box(is_identity);
    Ok(())
}

/// A non-void `obb` is rebuilt over its eight corners and the eight corners of the PCA
/// box, in the native order.
fn rebuild_with_pca_box(obb: &mut OrientedBoundingBox, pca: &PcaBox) -> Result<(), Failure> {
    let mut points: Vec<Point3> = obb.vertices().map(Vec::from).unwrap_or_default();
    let [x_ext, y_ext, z_ext] = [
        pca.half_sizes.x() * pca.axes[0],
        pca.half_sizes.y() * pca.axes[1],
        pca.half_sizes.z() * pca.axes[2],
    ];
    let c = pca.center;
    points.extend(
        [
            c - x_ext - y_ext - z_ext,
            c + x_ext - y_ext - z_ext,
            c - x_ext + y_ext - z_ext,
            c + x_ext + y_ext - z_ext,
            c - x_ext - y_ext + z_ext,
            c + x_ext - y_ext + z_ext,
            c - x_ext + y_ext + z_ext,
            c + x_ext + y_ext + z_ext,
        ]
        .map(Point3::from),
    );
    obb.rebuild(&Array1::from_shared(&points, 0), None, false)
}
