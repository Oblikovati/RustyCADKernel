// Translation of BRepOffset::CollapseSingularities for B-spline surfaces at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Face, Shape, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Point3, resolution};

#[derive(Clone, Copy)]
struct DegeneratedVertexPoint {
    point: Point3,
    tolerance: f64,
}

#[derive(Clone, Copy)]
struct SurfaceSidePole {
    u_index: i32,
    v_index: i32,
    point: Point3,
}

/// Collapses singular B-spline boundary pole rows using the face's degenerated vertices.
/// Other surface classes pass through unchanged, as in `BRepOffset::CollapseSingularities`.
///
/// ```
/// use rustycad_tkbrep::topods::Face;
/// use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::Axis3;
/// use rustycad_tkoffset::brep_offset::collapse_singularities;
/// let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
/// let collapsed = collapse_singularities(&plane, &Face::default(), 1.0e-7).unwrap();
/// assert_eq!(collapsed.object_address(), plane.object_address());
/// ```
pub fn collapse_singularities(
    surface: &Handle<dyn GeomSurface>,
    face: &Face,
    precision: f64,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let bspline_handle: Handle<GeomBSplineSurface> = surface.cast();
    let Some(bspline) = bspline_handle.get() else {
        return Ok(surface.clone());
    };
    let mut vertices = degenerated_vertex_points(face)?;
    let collapsed_sides = collapsed_side_centers(bspline, &mut vertices, precision)?;
    copy_with_collapsed_sides(surface, bspline, &collapsed_sides)
}

fn degenerated_vertex_points(face: &Face) -> Result<Vec<DegeneratedVertexPoint>, Failure> {
    let shape: Shape = face.clone().into();
    let mut explorer = Explorer::new(&shape, ShapeEnum::Edge, ShapeEnum::Shape)?;
    let mut points: Vec<DegeneratedVertexPoint> = Vec::new();
    while explorer.more() {
        collect_degenerated_vertex(&Edge::unchecked(explorer.current().clone()), &mut points)?;
        explorer.next()?;
    }
    Ok(points)
}

fn collect_degenerated_vertex(
    edge: &Edge,
    points: &mut Vec<DegeneratedVertexPoint>,
) -> Result<(), Failure> {
    if !BRepTool::degenerated(edge)? {
        return Ok(());
    }
    let (first, last) = topexp::vertices(edge, false);
    if !first.is_same(&last) {
        return Ok(());
    }
    append_vertex_point(&first, points)
}

fn append_vertex_point(
    vertex: &Vertex,
    points: &mut Vec<DegeneratedVertexPoint>,
) -> Result<(), Failure> {
    points.push(DegeneratedVertexPoint {
        point: BRepTool::pnt(vertex)?,
        tolerance: BRepTool::vertex_tolerance(vertex)?,
    });
    Ok(())
}

fn collapsed_side_centers(
    surface: &GeomBSplineSurface,
    vertices: &mut Vec<DegeneratedVertexPoint>,
    precision: f64,
) -> Result<Vec<(usize, Point3)>, Failure> {
    let mut collapsed: Vec<(usize, Point3)> = Vec::new();
    for side in 0..4 {
        let poles = side_poles(surface, side)?;
        let Some(center) = collapse_side_center(&poles, vertices, precision) else {
            continue;
        };
        collapsed.push((side, center));
    }
    Ok(collapsed)
}

fn side_poles(surface: &GeomBSplineSurface, side: usize) -> Result<Vec<SurfaceSidePole>, Failure> {
    let (u_count, v_count) = (surface.nb_u_poles(), surface.nb_v_poles());
    let (u_start, v_start, u_step, v_step, count) = match side {
        0 => (1, 1, 0, 1, v_count),
        1 => (1, 1, 1, 0, u_count),
        2 => (u_count, 1, 0, 1, v_count),
        _ => (1, v_count, 1, 0, u_count),
    };
    let mut poles: Vec<SurfaceSidePole> = Vec::with_capacity(count as usize);
    for index in 0..count {
        let (u_index, v_index) = (u_start + index * u_step, v_start + index * v_step);
        poles.push(SurfaceSidePole {
            u_index,
            v_index,
            point: surface.pole(u_index, v_index)?,
        });
    }
    Ok(poles)
}

fn collapse_side_center(
    poles: &[SurfaceSidePole],
    vertices: &mut Vec<DegeneratedVertexPoint>,
    precision: f64,
) -> Option<Point3> {
    let center = pole_center(poles);
    if poles
        .iter()
        .all(|pole| pole.point.distance(&center) <= resolution())
    {
        return None;
    }
    if poles
        .iter()
        .all(|pole| pole.point.distance(&center) <= precision)
    {
        return Some(center);
    }
    let fitting_count = remove_fitting_vertices(poles, vertices);
    if fitting_count != 1 {
        return None;
    }
    Some(center)
}

fn remove_fitting_vertices(
    poles: &[SurfaceSidePole],
    vertices: &mut Vec<DegeneratedVertexPoint>,
) -> usize {
    let fitting: Vec<bool> = vertices
        .iter()
        .map(|vertex| side_fits_vertex(poles, vertex))
        .collect();
    let count = fitting.iter().filter(|fits| **fits).count();
    *vertices = vertices
        .iter()
        .zip(fitting)
        .filter_map(|(vertex, fits)| (!fits).then_some(*vertex))
        .collect();
    count
}

fn side_fits_vertex(poles: &[SurfaceSidePole], vertex: &DegeneratedVertexPoint) -> bool {
    poles
        .iter()
        .all(|pole| pole.point.distance(&vertex.point) < vertex.tolerance)
}

fn pole_center(poles: &[SurfaceSidePole]) -> Point3 {
    let (x, y, z) = poles.iter().fold((0.0, 0.0, 0.0), |sum, pole| {
        (
            sum.0 + pole.point.x(),
            sum.1 + pole.point.y(),
            sum.2 + pole.point.z(),
        )
    });
    let count = poles.len() as f64;
    Point3::new(x / count, y / count, z / count)
}

fn copy_with_collapsed_sides(
    original: &Handle<dyn GeomSurface>,
    surface: &GeomBSplineSurface,
    collapsed_sides: &[(usize, Point3)],
) -> Result<Handle<dyn GeomSurface>, Failure> {
    if collapsed_sides.is_empty() {
        return Ok(original.clone());
    }
    let copy = GeomBSplineSurface::copy_of(surface)?;
    set_collapsed_side_poles(&copy, collapsed_sides)?;
    Ok(Handle::new(copy).cast())
}

fn set_collapsed_side_poles(
    surface: &GeomBSplineSurface,
    collapsed_sides: &[(usize, Point3)],
) -> Result<(), Failure> {
    for (side, center) in collapsed_sides {
        for pole in side_poles(surface, *side)? {
            surface.set_pole(pole.u_index, pole.v_index, *center)?;
        }
    }
    Ok(())
}
