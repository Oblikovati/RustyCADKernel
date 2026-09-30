//! Behavior tests of the BRepLib package functions and of BRepLib_CheckCurveOnSurface
//! and BRepLib_ToolTriangulatedShape (docs/tktopalgo-progress.md). The numeric oracle
//! compares the same functions with OCCT bit for bit (`blp_*` requests).
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{BRepToolsReShape, ReShape};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Face, Shape, Solid, Vertex};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomCylindricalSurface, GeomLine, GeomPlane, GeomSphericalSurface,
    GeomSurface,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Axis3, Direction3, Point2, Point3};
use rustycad_tkmath::poly::{Triangle, Triangulation, mesh_purpose};
use rustycad_tktopalgo::brep_lib::{
    self, BRepLibCheckCurveOnSurface, BRepLibMakeEdge, BRepLibMakeShell,
    BRepLibToolTriangulatedShape, RangeEnd,
};

fn shell_of(
    surface: Handle<dyn GeomSurface>,
    bounds: (f64, f64, f64, f64),
) -> Result<Shape, Failure> {
    Ok(BRepLibMakeShell::with_bounds(&surface, bounds, false)?
        .shell()?
        .into_shape())
}

fn plane_shell(size: f64) -> Result<Shape, Failure> {
    shell_of(
        Handle::new(GeomPlane::new(Axis3::default())).cast(),
        (0., size, 0., size),
    )
}

fn first_of<T>(shape: &Shape, kind: ShapeEnum, cast: fn(Shape) -> T) -> Result<T, Failure> {
    let x = Explorer::new(shape, kind, ShapeEnum::Shape)?;
    Ok(cast(x.current().clone()))
}

/// A bicubic B-spline surface with a C1 knot in U: two C2 patches.
fn two_patch_surface() -> Result<Handle<dyn GeomSurface>, Failure> {
    let mut poles = Array2::<Point3>::with_bounds(1, 6, 1, 4).unwrap();
    for i in 1..=6 {
        for j in 1..=4 {
            poles
                .set_value(
                    i,
                    j,
                    Point3::new(f64::from(i), f64::from(j), 0.1 * f64::from(i * j)),
                )
                .unwrap();
        }
    }
    let mut uk = Array1::<f64>::with_bounds(1, 3).unwrap();
    let mut um = Array1::<i32>::with_bounds(1, 3).unwrap();
    (uk[1], uk[2], uk[3], um[1], um[2], um[3]) = (0., 1., 2., 4, 2, 4);
    let mut vk = Array1::<f64>::with_bounds(1, 2).unwrap();
    let mut vm = Array1::<i32>::with_bounds(1, 2).unwrap();
    (vk[1], vk[2], vm[1], vm[2]) = (0., 1., 4, 4);
    let s = GeomBSplineSurface::new(&poles, (&uk, &um), (&vk, &vm), (3, 3), (false, false))?;
    Ok(Handle::new(s).cast())
}

#[test]
fn make_shell_encodes_the_regularity_between_its_patches() -> Result<(), Failure> {
    let shell = shell_of(two_patch_surface()?, (0., 2., 0., 1.))?;
    let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(&shell, ShapeEnum::Edge, ShapeEnum::Face, &mut ancestors)?;
    let shared = (1..=ancestors.extent())
        .find(|&i| {
            ancestors
                .find_from_index(i)
                .map(|l| l.extent() == 2)
                .unwrap_or(false)
        })
        .expect("the patches share an edge");
    let edge = Edge::unchecked(ancestors.find_key(shared)?.clone());
    let faces: Vec<Face> = ancestors
        .find_from_index(shared)?
        .iter()
        .map(|f| Face::unchecked(f.clone()))
        .collect();
    let continuity = BRepTool::continuity_on_faces(&edge, &faces[0], &faces[1])?;
    assert_eq!(
        continuity,
        brep_lib::continuity_of_faces(&edge, &faces[0], &faces[1], 1e-10)?
    );
    assert!(continuity >= Continuity::G1);
    Ok(())
}

#[test]
fn a_seam_of_an_elementary_surface_is_cn() -> Result<(), Failure> {
    let cylinder: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(Axis3::default(), 1.)?).cast();
    let shell = shell_of(cylinder, (0., 2. * std::f64::consts::PI, 0., 1.))?;
    let face: Face = first_of(&shell, ShapeEnum::Face, Face::unchecked)?;
    let mut x = Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while x.more() {
        let edge = Edge::unchecked(x.current().clone());
        if BRepTool::is_closed_on_face(&edge, &face)? {
            assert_eq!(
                brep_lib::continuity_of_faces(&edge, &face, &face, 1e-10)?,
                Continuity::CN
            );
            return Ok(());
        }
        x.next()?;
    }
    panic!("the cylinder face has a seam")
}

#[test]
fn build_curves3d_gives_every_edge_a_3d_curve() -> Result<(), Failure> {
    let shell = plane_shell(2.)?;
    assert!(brep_lib::build_curves3d_default(&shell)?);
    let edge: Edge = first_of(&shell, ShapeEnum::Edge, Edge::unchecked)?;
    assert!(BRepTool::curve(&edge)?.0.is_some());
    assert!(brep_lib::check_same_range(&edge, 1e-12)?);
    brep_lib::same_range(&edge, 1e-5)?;
    assert!(BRepTool::same_range(&edge)?);
    Ok(())
}

#[test]
fn update_edge_tolerance_measures_the_curves_on_surfaces() -> Result<(), Failure> {
    let shell = plane_shell(2.)?;
    assert!(brep_lib::update_edge_tolerance(&shell, 1e-5, 1.)?);
    let edge: Edge = first_of(&shell, ShapeEnum::Edge, Edge::unchecked)?;
    // The curves coincide: the tolerance drops to their (null) distance.
    assert_eq!(
        BRepTool::edge_tolerance(&edge)?,
        rustycad_tkernel::precision::confusion()
    );
    assert!(!brep_lib::update_edge_tol(&edge, 1e-5, 1e-9)?);
    brep_lib::update_inner_tolerances(&shell)?;
    Ok(())
}

#[test]
fn sort_faces_orders_by_surface_kind() -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let mut compound = rustycad_tkbrep::topods::Compound::default();
    builder.make_compound(&mut compound);
    let sphere: Handle<dyn GeomSurface> =
        Handle::new(GeomSphericalSurface::new(Axis3::default(), 1.)?).cast();
    builder.add(&mut compound, &shell_of(sphere, (0., 1., -0.5, 0.5))?)?;
    builder.add(&mut compound, &plane_shell(1.)?)?;
    let sorted = brep_lib::sort_faces(&compound)?;
    let reversed = brep_lib::reverse_sort_faces(&compound)?;
    let kind = |f: &Shape| {
        BRepTool::surface(&Face::unchecked(f.clone())).map(|s| s.cast::<GeomPlane>().is_some())
    };
    assert!(kind(sorted.first().unwrap())?);
    assert!(kind(reversed.last().unwrap())?);
    Ok(())
}

fn vertex(point: Point3, tolerance: f64) -> Result<Vertex, Failure> {
    let mut v = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut v, point, tolerance)?;
    Ok(v)
}

#[test]
fn bounding_vertex_covers_every_sphere() -> Result<(), Failure> {
    let mut list: List<Shape> = List::default();
    assert!(brep_lib::bounding_vertex(&list)?.is_none());
    list.append(vertex(Point3::new(0., 0., 0.), 0.5)?.into_shape());
    list.append(vertex(Point3::new(0.1, 0., 0.), 0.1)?.into_shape());
    // The larger sphere holds the other one.
    assert_eq!(
        brep_lib::bounding_vertex(&list)?,
        Some((Point3::new(0., 0., 0.), 0.5))
    );
    list.append(vertex(Point3::new(3., 0., 0.), 0.1)?.into_shape());
    let (center, radius) = brep_lib::bounding_vertex(&list)?.unwrap();
    assert!(center.x() > 1. && radius > 1.9);
    Ok(())
}

#[test]
fn find_valid_range_leaves_the_vertex_spheres() -> Result<(), Failure> {
    let line: Handle<dyn rustycad_tkg3d::geom::GeomCurve> = Handle::new(GeomLine::from_point(
        Point3::new(0., 0., 0.),
        Direction3::new(1., 0., 0.)?,
    ))
    .cast();
    let curve = GeomAdaptorCurve::with_range(&line, 0., 10.)?;
    let end = |parameter: f64, tolerance: f64| RangeEnd {
        parameter,
        point: Point3::new(parameter, 0., 0.),
        tolerance,
    };
    let found = brep_lib::find_valid_range(&curve, 1e-7, end(0., 1.), end(10., 1.))?;
    assert!(found.is_valid);
    assert!((found.first.unwrap() - 1.).abs() < 1e-6 && (found.last.unwrap() - 9.).abs() < 1e-6);
    let covered = brep_lib::find_valid_range(&curve, 1e-7, end(0., 6.), end(10., 6.))?;
    assert!(!covered.is_valid);
    // The native call wrote both outputs before finding them overlapping.
    assert!(covered.first.is_some() && covered.last.is_some());
    let edge =
        BRepLibMakeEdge::from_points(Point3::new(0., 0., 0.), Point3::new(1., 0., 0.))?.edge()?;
    assert!(brep_lib::find_valid_range_of_edge(&edge)?.is_valid);
    Ok(())
}

#[test]
fn pcurve_on_plane_is_computed_for_an_edge_without_one() -> Result<(), Failure> {
    let face: Face = first_of(&plane_shell(2.)?, ShapeEnum::Face, Face::unchecked)?;
    let edge =
        BRepLibMakeEdge::from_points(Point3::new(0.5, 0.5, 0.), Point3::new(1., 1., 0.))?.edge()?;
    let (pcurve, to_update) = brep_lib::pcurve_for_edge_on_plane(&edge, &face)?;
    assert!(to_update && pcurve.is_some());
    brep_lib::build_pcurve_for_edge_on_plane(&edge, &face)?;
    assert!(!brep_lib::pcurve_for_edge_on_plane(&edge, &face)?.1);
    Ok(())
}

#[test]
fn check_curve_on_surface_measures_the_edges_of_a_shell() -> Result<(), Failure> {
    let shell = plane_shell(2.)?;
    let face: Face = first_of(&shell, ShapeEnum::Face, Face::unchecked)?;
    let edge: Edge = first_of(&shell, ShapeEnum::Edge, Edge::unchecked)?;
    let mut check = BRepLibCheckCurveOnSurface::with_edge_face(&edge, &face)?;
    check.set_parallel(true);
    assert!(check.is_parallel());
    check.perform();
    assert!(check.is_done());
    assert!(check.max_distance() < 1e-12);
    let mut empty = BRepLibCheckCurveOnSurface::new();
    empty.init(&Edge::default(), &face)?;
    empty.perform();
    assert_eq!(empty.error_status(), 1);
    Ok(())
}

/// Two triangles on the square [0, 1]^2 of the face, with UV nodes.
fn square_mesh(face: &Face) -> Result<Handle<Triangulation>, Failure> {
    let t = Triangulation::with_sizes(4, 2, true, false)?;
    for (k, (u, v)) in [(0., 0.), (1., 0.), (1., 1.), (0., 1.)]
        .into_iter()
        .enumerate()
    {
        t.set_node(k as i32 + 1, Point3::new(u, v, 0.))?;
        t.set_uv_node(k as i32 + 1, Point2::new(u, v))?;
    }
    t.set_triangle(1, Triangle::new(1, 2, 3))?;
    t.set_triangle(2, Triangle::new(1, 3, 4))?;
    let t = Handle::new(t);
    BRepBuilder::default().update_face_triangulation(face, &t, true)?;
    Ok(t)
}

#[test]
fn triangulation_normals_and_deflection() -> Result<(), Failure> {
    let shell = plane_shell(1.)?;
    let face: Face = first_of(&shell, ShapeEnum::Face, Face::unchecked)?;
    let t = square_mesh(&face)?;
    BRepLibToolTriangulatedShape::compute_normals(&face, &t)?;
    let mesh = t.get().unwrap();
    assert!(mesh.has_normals());
    assert_eq!(mesh.normal(1)?.z(), 1.);
    // Normals exist: nothing to compute and no edge between two faces.
    assert!(!brep_lib::ensure_normal_consistency(&shell, 0.001, false)?);
    assert!(brep_lib::ensure_normal_consistency(&shell, 0.001, true)?);
    brep_lib::update_deflection(&shell)?;
    let (again, _) = BRepTool::triangulation(&face, mesh_purpose::NONE)?;
    assert_eq!(again.get().unwrap().deflection(), 0.);
    Ok(())
}

/// The largest tolerance of the sub-shapes of a kind.
fn max_tolerance(shape: &Shape, kind: ShapeEnum) -> Result<f64, Failure> {
    let mut max: f64 = 0.;
    for sub in Explorer::new(shape, kind, ShapeEnum::Shape)? {
        let tol = match kind {
            ShapeEnum::Vertex => BRepTool::vertex_tolerance(&Vertex::unchecked(sub))?,
            ShapeEnum::Edge => BRepTool::edge_tolerance(&Edge::unchecked(sub))?,
            _ => BRepTool::face_tolerance(&Face::unchecked(sub))?,
        };
        max = max.max(tol);
    }
    Ok(max)
}

#[test]
fn update_tolerances_orders_vertex_edge_and_face() -> Result<(), Failure> {
    let shell = plane_shell(2.)?;
    let face: Face = first_of(&shell, ShapeEnum::Face, Face::unchecked)?;
    BRepBuilder::default().update_face_tolerance(&face, 1e-3)?;
    // With a re-shaper the input keeps its tolerances; the copies carry the new ones.
    let reshaper = BRepToolsReShape::new();
    brep_lib::update_tolerances_with_reshape(&shell, &reshaper, false)?;
    assert!(max_tolerance(&shell, ShapeEnum::Edge)? < 1e-3);
    let updated = reshaper.apply(&shell, ShapeEnum::Shape)?;
    assert!(max_tolerance(&updated, ShapeEnum::Edge)? >= 1e-3);
    // In place: every edge and vertex covers the face tolerance.
    brep_lib::update_tolerances(&shell, false)?;
    assert!(max_tolerance(&shell, ShapeEnum::Edge)? >= 1e-3);
    assert!(max_tolerance(&shell, ShapeEnum::Vertex)? >= 1e-3);
    // The verification resets the face tolerance of a plane to Confusion x the size of
    // its box, which the 1e-3 edge tolerances enlarge on both sides (2.002).
    brep_lib::update_tolerances(&shell, true)?;
    let face_tol = max_tolerance(&shell, ShapeEnum::Face)?;
    assert!(
        (face_tol - 2.002e-7).abs() < 1e-15,
        "face tolerance {face_tol}"
    );
    Ok(())
}

#[test]
fn orient_closed_solid_sets_the_material_inside() -> Result<(), Failure> {
    let sphere: Handle<dyn GeomSurface> =
        Handle::new(GeomSphericalSurface::new(Axis3::default(), 2.)?).cast();
    let shell = BRepLibMakeShell::with_surface(&sphere, false)?
        .shell()?
        .into_shape();
    let builder = BRepBuilder::default();
    for (reversed, expected) in [(false, Orientation::Forward), (true, Orientation::Reversed)] {
        let mut solid = Solid::default();
        builder.make_solid(&mut solid);
        builder.add(
            &mut solid,
            &if reversed {
                shell.reversed()
            } else {
                shell.clone()
            },
        )?;
        assert!(brep_lib::orient_closed_solid(&mut solid)?);
        assert_eq!(solid.orientation(), expected);
    }
    Ok(())
}
