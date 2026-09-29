//! Intf interference probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_intf_poly.cxx: InterferencePolygon2d over a probe
//! polygon, and the generic InterferencePolygonPolyhedron over probe tools.
use super::common::{Reader, attempt, end, pnt2d};
use super::intf::interference_print;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::intf::{
    InterferencePolygon2d, InterferencePolygonPolyhedron, Polygon2d, Polygon3dTool, PolyhedronTool,
};
use rustycad_tkmath::bnd::{BoundSortBox, BoundingBox2, BoundingBox3};
use rustycad_tkmath::gp::{Line3, Point2, Point3};

fn out_of_range(member: &str, index: i32) -> Failure {
    Failure::out_of_range(&format!("{member}: index {index} out of range"))
}

/// `GaPolygon2d`: a polyline whose box holds its points, enlarged by the deflection.
struct ProbePolygon2d {
    closed: bool,
    deflection: f64,
    points: Vec<Point2>,
    bounds: BoundingBox2,
}

impl ProbePolygon2d {
    fn read(r: &mut Reader) -> Self {
        let (closed, deflection) = (r.flag(), r.real());
        let points: Vec<Point2> = (0..r.integer()).map(|_| r.pnt2d()).collect();
        let mut bounds = BoundingBox2::default();
        points.iter().for_each(|point| bounds.add_point(*point));
        bounds.enlarge(deflection);
        Self {
            closed,
            deflection,
            points,
            bounds,
        }
    }
}

impl Polygon2d for ProbePolygon2d {
    fn bounding(&self) -> &BoundingBox2 {
        &self.bounds
    }
    fn closed(&self) -> bool {
        self.closed
    }
    fn deflection_over_estimation(&self) -> f64 {
        self.deflection
    }
    fn nb_segments(&self) -> i32 {
        self.points.len() as i32 - 1
    }
    fn segment(&self, index: i32) -> Result<(Point2, Point2), Failure> {
        if index < 1 || index > self.nb_segments() {
            return Err(out_of_range("GaPolygon2d::Segment", index));
        }
        Ok((self.points[index as usize - 1], self.points[index as usize]))
    }
}

fn ipoly2d_print(interference: &InterferencePolygon2d) {
    interference_print(interference);
    for i in 1..=interference.nb_section_points() + 1 {
        attempt(|| interference.pnt2d_value(i).map(pnt2d));
    }
}

/// Performs (0, 1) and constructors (2, 3) of two polygons or one on one interference.
pub fn ipoly2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut interference = InterferencePolygon2d::new();
    ipoly2d_print(&interference);
    for _ in 0..r.integer() {
        let kind = r.integer();
        let first = ProbePolygon2d::read(&mut r);
        if kind == 0 || kind == 2 {
            let second = ProbePolygon2d::read(&mut r);
            if kind == 0 {
                attempt(|| interference.perform(&first, &second));
            } else {
                attempt(|| {
                    interference = InterferencePolygon2d::between(&first, &second)?;
                    Ok(())
                });
            }
        } else if kind == 1 {
            attempt(|| interference.perform_self(&first));
        } else {
            attempt(|| {
                interference = InterferencePolygon2d::of_self(&first)?;
                Ok(())
            });
        }
        ipoly2d_print(&interference);
    }
    end();
}

/// `GaPolygon3d`: a 3d polyline whose box holds its points, enlarged by the deflection.
struct ProbePolygon3d {
    closed: bool,
    deflection: f64,
    points: Vec<Point3>,
    bounds: BoundingBox3,
}

impl ProbePolygon3d {
    fn read(r: &mut Reader) -> Self {
        let (closed, deflection) = (r.flag(), r.real());
        let points: Vec<Point3> = (0..r.integer()).map(|_| r.pnt()).collect();
        let mut bounds = BoundingBox3::default();
        points.iter().for_each(|point| bounds.add_point(*point));
        bounds.enlarge(deflection);
        Self {
            closed,
            deflection,
            points,
            bounds,
        }
    }
    fn point(&self, index: i32) -> Result<Point3, Failure> {
        if index < 1 || index > self.points.len() as i32 {
            return Err(out_of_range("GaPolygon3d::Point", index));
        }
        Ok(self.points[index as usize - 1])
    }
}

/// `GaPolygon3dTool`.
struct ProbePolygon3dTool;

impl Polygon3dTool for ProbePolygon3dTool {
    type Polygon = ProbePolygon3d;
    fn bounding(polygon: &ProbePolygon3d) -> &BoundingBox3 {
        &polygon.bounds
    }
    fn deflection_over_estimation(polygon: &ProbePolygon3d) -> f64 {
        polygon.deflection
    }
    fn closed(polygon: &ProbePolygon3d) -> bool {
        polygon.closed
    }
    fn nb_segments(polygon: &ProbePolygon3d) -> i32 {
        polygon.points.len() as i32 - 1
    }
    fn begin_of_seg(polygon: &ProbePolygon3d, index: i32) -> Result<Point3, Failure> {
        polygon.point(index)
    }
    fn end_of_seg(polygon: &ProbePolygon3d, index: i32) -> Result<Point3, Failure> {
        polygon.point(index + 1)
    }
}

/// `GaPolyhedron`: a triangulation with per-triangle boxes and listed boundary edges.
struct ProbePolyhedron {
    deflection: f64,
    border: f64,
    points: Vec<Point3>,
    triangles: Vec<[i32; 3]>,
    bounds_edges: Vec<(i32, i32)>,
    bounds: BoundingBox3,
    boxes: Array1<'static, BoundingBox3>,
}

impl ProbePolyhedron {
    fn read(r: &mut Reader) -> Self {
        let (deflection, border) = (r.real(), r.real());
        let points: Vec<Point3> = (0..r.integer()).map(|_| r.pnt()).collect();
        let mut bounds = BoundingBox3::default();
        points.iter().for_each(|point| bounds.add_point(*point));
        bounds.enlarge(deflection);
        let triangles: Vec<[i32; 3]> = (0..r.integer())
            .map(|_| [r.integer(), r.integer(), r.integer()])
            .collect();
        let mut boxes: Array1<'static, BoundingBox3> =
            Array1::with_bounds(1, triangles.len() as i32).expect("probe triangle boxes");
        for (slot, triangle) in boxes.iter_mut().zip(&triangles) {
            triangle
                .iter()
                .for_each(|&index| slot.add_point(points[index as usize - 1]));
            slot.enlarge(deflection);
        }
        let bounds_edges: Vec<(i32, i32)> = (0..r.integer())
            .map(|_| (r.integer(), r.integer()))
            .collect();
        Self {
            deflection,
            border,
            points,
            triangles,
            bounds_edges,
            bounds,
            boxes,
        }
    }
}

/// `GaPolyhedronTool`.
struct ProbePolyhedronTool;

impl PolyhedronTool for ProbePolyhedronTool {
    type Polyhedron = ProbePolyhedron;
    fn bounding(polyhedron: &ProbePolyhedron) -> &BoundingBox3 {
        &polyhedron.bounds
    }
    fn components_bounding(polyhedron: &ProbePolyhedron) -> &Array1<'static, BoundingBox3> {
        &polyhedron.boxes
    }
    fn deflection_over_estimation(polyhedron: &ProbePolyhedron) -> f64 {
        polyhedron.deflection
    }
    fn triangle(polyhedron: &ProbePolyhedron, index: i32) -> Result<[i32; 3], Failure> {
        if index < 1 || index > polyhedron.triangles.len() as i32 {
            return Err(out_of_range("GaPolyhedron::Triangle", index));
        }
        Ok(polyhedron.triangles[index as usize - 1])
    }
    fn point(polyhedron: &ProbePolyhedron, index: i32) -> Result<Point3, Failure> {
        if index < 1 || index > polyhedron.points.len() as i32 {
            return Err(out_of_range("GaPolyhedron::Point", index));
        }
        Ok(polyhedron.points[index as usize - 1])
    }
    /// The probe polyhedron has no connectivity: every edge is free.
    fn tri_connex(
        _polyhedron: &ProbePolyhedron,
        _triang: i32,
        _pivot: i32,
        _pedge: i32,
    ) -> Result<(i32, i32, i32), Failure> {
        Ok((0, 0, 0))
    }
    fn is_on_bound(polyhedron: &ProbePolyhedron, a: i32, b: i32) -> Result<bool, Failure> {
        Ok(polyhedron
            .bounds_edges
            .iter()
            .any(|&(e1, e2)| (e1 == a && e2 == b) || (e1 == b && e2 == a)))
    }
    fn get_border_deflection(polyhedron: &ProbePolyhedron) -> f64 {
        polyhedron.border
    }
}

type ProbeInterference = InterferencePolygonPolyhedron<ProbePolygon3dTool, ProbePolyhedronTool>;

fn lines(r: &mut Reader) -> Array1<'static, Line3> {
    let n = r.integer();
    let mut lines: Array1<'static, Line3> = Array1::with_bounds(1, n.max(1)).expect("probe lines");
    for slot in lines.iter_mut().take(n as usize) {
        let p = r.pnt();
        *slot = Line3::new(p, r.dir());
    }
    lines
}

fn grid_of(polyhedron: &ProbePolyhedron) -> BoundSortBox {
    let mut grid = BoundSortBox::default();
    grid.initialize_with_enclosing(&polyhedron.bounds, polyhedron.boxes.clone())
        .expect("probe polyhedron grid");
    grid
}

/// One operation of `ga_ipp_operation`: Perform (0..5), Interference (6, 7) and the
/// constructors (8..13).
fn ipp_operation(
    r: &mut Reader,
    interference: &mut ProbeInterference,
    polyhedron: &ProbePolyhedron,
    kind: i32,
) {
    let mut grid = grid_of(polyhedron);
    let (base, construct) = if kind >= 8 {
        (kind - 8, true)
    } else {
        (kind, false)
    };
    let replace = |interference: &mut ProbeInterference,
                   made: Result<ProbeInterference, Failure>| {
        made.map(|made| *interference = made)
    };
    if base == 0 || base == 3 || kind == 6 || kind == 7 {
        let polygon = ProbePolygon3d::read(r);
        attempt(|| match (kind, base, construct) {
            (6, _, _) => interference.interference(&polygon, polyhedron),
            (7, _, _) => interference.interference_grid(&polygon, polyhedron, &mut grid),
            (_, 0, true) => replace(
                interference,
                ProbeInterference::with_polygon(&polygon, polyhedron),
            ),
            (_, 0, false) => interference.perform_polygon(&polygon, polyhedron),
            (_, _, true) => replace(
                interference,
                ProbeInterference::with_polygon_grid(&polygon, polyhedron, &mut grid),
            ),
            _ => interference.perform_polygon_grid(&polygon, polyhedron, &mut grid),
        });
        return;
    }
    if base == 1 || base == 4 {
        let p = r.pnt();
        let line = Line3::new(p, r.dir());
        attempt(|| match (base, construct) {
            (1, true) => replace(interference, ProbeInterference::with_line(line, polyhedron)),
            (1, false) => interference.perform_line(line, polyhedron),
            (_, true) => replace(
                interference,
                ProbeInterference::with_line_grid(line, polyhedron, &mut grid),
            ),
            _ => interference.perform_line_grid(line, polyhedron, &mut grid),
        });
        return;
    }
    let lines = lines(r);
    attempt(|| match (base, construct) {
        (2, true) => replace(
            interference,
            ProbeInterference::with_lines(&lines, polyhedron),
        ),
        (2, false) => interference.perform_lines(&lines, polyhedron),
        (_, true) => replace(
            interference,
            ProbeInterference::with_lines_grid(&lines, polyhedron, &mut grid),
        ),
        _ => interference.perform_lines_grid(&lines, polyhedron, &mut grid),
    });
}

/// `polyhedron count (kind operand)...`: the operations on one interference.
pub fn ipp_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let polyhedron = ProbePolyhedron::read(&mut r);
    let mut interference = ProbeInterference::new();
    for _ in 0..r.integer() {
        let kind = r.integer();
        ipp_operation(&mut r, &mut interference, &polyhedron, kind);
        interference_print(&interference);
    }
    end();
}
