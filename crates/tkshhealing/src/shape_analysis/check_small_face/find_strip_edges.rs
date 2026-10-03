use super::spot::ShapeAnalysisCheckSmallFace;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{Explorer, vertices};
use rustycad_tkbrep::topods::{Edge, Face, edge};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;

impl ShapeAnalysisCheckSmallFace {
    /// Finds the only two non-null-length edges and checks that their curves form a strip.
    /// `analyzer.find_strip_edges(&face, &mut first, &mut second, 1e-3, &mut maximum)?`.
    pub fn find_strip_edges(
        &mut self,
        face: &Face,
        first_edge: &mut Edge,
        second_edge: &mut Edge,
        tolerance: f64,
        maximum_distance: &mut f64,
    ) -> Result<bool, Failure> {
        *first_edge = Edge::default();
        *second_edge = Edge::default();
        let found = collect_strip_edges(face, tolerance, first_edge, second_edge)?;
        if found != 2
            || !self.check_strip_edges(first_edge, second_edge, tolerance, maximum_distance)?
        {
            return Ok(false);
        }
        self.strip_status = ShapeExtend::encode_status(ShapeExtendStatus::Done3);
        Ok(true)
    }
}

fn collect_strip_edges(
    face: &Face,
    tolerance: f64,
    first_edge: &mut Edge,
    second_edge: &mut Edge,
) -> Result<i32, Failure> {
    let mut found = 0;
    let mut edges = Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )?;
    while edges.more() {
        let current = edge(edges.current())?;
        if inspect_candidate(&mut found, &current, tolerance, first_edge, second_edge)? {
            return Ok(found);
        }
        edges.next()?;
    }
    Ok(found)
}

fn inspect_candidate(
    found: &mut i32,
    current: &Edge,
    tolerance: f64,
    first_edge: &mut Edge,
    second_edge: &mut Edge,
) -> Result<bool, Failure> {
    if is_repeated_seam(*found, current, first_edge) || is_null_length_edge(current, tolerance)? {
        return Ok(false);
    }
    *found += 1;
    assign_candidate(*found, current, first_edge, second_edge);
    Ok(*found > 2)
}

fn is_repeated_seam(found: i32, current: &Edge, first_edge: &Edge) -> bool {
    found == 1 && current.is_same(first_edge)
}

fn is_null_length_edge(edge: &Edge, tolerance: f64) -> Result<bool, Failure> {
    let (start, end) = vertices(edge, true);
    let start_point = BRepTool::pnt(&start)?;
    let end_point = BRepTool::pnt(&end)?;
    let local_tolerance = if tolerance <= 0.0 {
        (BRepTool::vertex_tolerance(&start)? + BRepTool::vertex_tolerance(&end)?) / 2.0
    } else {
        tolerance
    };
    let endpoint_distance = start_point.distance(&end_point);
    let Some(curve) = edge_curve(edge)? else {
        return Ok(endpoint_distance <= local_tolerance);
    };
    midpoint_is_collapsed(&curve, start_point, end_point, local_tolerance)
}

fn midpoint_is_collapsed(
    curve: &rustycad_tkernel::standard::Handle<dyn GeomCurve>,
    start: rustycad_tkmath::gp::Point3,
    end: rustycad_tkmath::gp::Point3,
    tolerance: f64,
) -> Result<bool, Failure> {
    let midpoint = curve.value((curve.first_parameter() + curve.last_parameter()) / 2.0)?;
    if midpoint.distance(&start) < tolerance && midpoint.distance(&end) < tolerance {
        return Ok(true);
    }
    Ok(false)
}

fn edge_curve(
    edge: &Edge,
) -> Result<Option<rustycad_tkernel::standard::Handle<dyn GeomCurve>>, Failure> {
    let (curve, _, _) = BRepTool::curve(edge)?;
    if curve.get().is_some() {
        return Ok(Some(curve));
    }
    Ok(None)
}

fn assign_candidate(count: i32, current: &Edge, first_edge: &mut Edge, second_edge: &mut Edge) {
    match count {
        1 => *first_edge = current.clone(),
        2 => *second_edge = current.clone(),
        _ => {}
    }
}
