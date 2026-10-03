use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Face, ShapeIterator, edge, vertex};
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;

/// Spot-face analysis from `ShapeAnalysis_CheckSmallFace`.
#[derive(Default)]
pub struct ShapeAnalysisCheckSmallFace {
    overall_status: Option<i32>,
    spot_status: i32,
    pub(super) strip_status: i32,
    pub(super) pin_status: i32,
    pub(super) split_vertex_status: i32,
    pub(super) tolerance: Option<f64>,
}

impl ShapeAnalysisCheckSmallFace {
    /// Constructs the analyzer; `let analyzer = ShapeAnalysisCheckSmallFace::new();`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Calls native `CheckSpotFace` and records DONE1 or DONE2 without clearing earlier status.
    /// `analyzer.check_spot_face(&face, -1.0)?`.
    pub fn check_spot_face(&mut self, face: &Face, tolerance: f64) -> Result<bool, Failure> {
        let mut spot = Point3::default();
        let mut spot_tolerance = 0.0;
        let result =
            self.is_spot_face_with_tolerance(face, &mut spot, &mut spot_tolerance, tolerance)?;
        let status = spot_status(result);
        if let Some(status) = status {
            self.spot_status = ShapeExtend::encode_status(status);
            return Ok(true);
        }
        Ok(false)
    }

    /// Decodes the native overall status field when known; OCCT leaves that field
    /// uninitialized in its constructor, so a fresh analyzer returns `None`.
    pub fn status(&self, status: ShapeExtendStatus) -> Option<bool> {
        self.overall_status
            .map(|value| ShapeExtend::decode_status(value, status))
    }

    /// Decodes the last `CheckSpotFace` result; `analyzer.status_spot(ShapeExtendStatus::Done1)`.
    pub fn status_spot(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.spot_status, status)
    }

    /// Decodes the last `CheckPin` result; `analyzer.status_pin(ShapeExtendStatus::Done2)`.
    pub fn status_pin(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.pin_status, status)
    }

    /// Decodes the last `CheckSplittingVertices` scan; `analyzer.status_split_vert(ShapeExtendStatus::Done)`.
    pub fn status_split_vert(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.split_vertex_status, status)
    }

    /// Sets the separate checker precision field; `analyzer.set_tolerance(0.01)`.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.tolerance = Some(tolerance);
    }

    /// Reads the checker precision after `SetTolerance`; `analyzer.tolerance() == Some(0.01)`.
    /// OCCT leaves this field uninitialized until its setter has been called.
    pub fn tolerance(&self) -> Option<f64> {
        self.tolerance
    }

    /// Uses the native default local-tolerance check; outputs change only on a spot.
    /// `analyzer.is_spot_face(&face, &mut location, &mut size)?`.
    pub fn is_spot_face(
        &self,
        face: &Face,
        spot: &mut Point3,
        spot_tolerance: &mut f64,
    ) -> Result<i32, Failure> {
        self.is_spot_face_with_tolerance(face, spot, spot_tolerance, -1.0)
    }

    /// Matches `IsSpotFace(F, spot, spotol, tol)`, returning 0 (no), 1 (spot), or 2
    /// (one shared vertex); `analyzer.is_spot_face_with_tolerance(&face, &mut point, &mut size, 1e-3)?`.
    pub fn is_spot_face_with_tolerance(
        &self,
        face: &Face,
        spot: &mut Point3,
        spot_tolerance: &mut f64,
        tolerance: f64,
    ) -> Result<i32, Failure> {
        if !has_wire(face)? {
            return Ok(1);
        }
        let bounds = vertex_bounds(face, tolerance)?;
        if !bounds.is_small(bounds.tolerance) || !edge_samples_are_small(face, bounds.tolerance)? {
            return Ok(0);
        }
        assign_spot_outputs(&bounds, spot, spot_tolerance);
        Ok(if bounds.same_vertex { 2 } else { 1 })
    }
}

fn spot_status(result: i32) -> Option<ShapeExtendStatus> {
    match result {
        1 => Some(ShapeExtendStatus::Done1),
        2 => Some(ShapeExtendStatus::Done2),
        _ => None,
    }
}

#[derive(Clone)]
struct VertexBounds {
    minimum: Point3,
    maximum: Point3,
    tolerance: f64,
    has_vertices: bool,
    same_vertex: bool,
    first_vertex: Option<rustycad_tkbrep::topods::Shape>,
}

impl VertexBounds {
    fn initial(tolerance: f64) -> Self {
        Self {
            minimum: Point3::default(),
            maximum: Point3::new(
                precision::infinite(),
                precision::infinite(),
                precision::infinite(),
            ),
            tolerance,
            has_vertices: false,
            same_vertex: true,
            first_vertex: None,
        }
    }

    fn is_small(&self, tolerance: f64) -> bool {
        axis_is_small(self.maximum.x() - self.minimum.x(), tolerance)
            && axis_is_small(self.maximum.y() - self.minimum.y(), tolerance)
            && axis_is_small(self.maximum.z() - self.minimum.z(), tolerance)
    }

    fn include(
        &mut self,
        vertex: &rustycad_tkbrep::topods::Vertex,
        point: Point3,
    ) -> Result<(), Failure> {
        let mut minimum = self.minimum;
        let mut maximum = self.maximum;
        update_bounds(&mut minimum, &mut maximum, point, self.has_vertices);
        self.minimum = minimum;
        self.maximum = maximum;
        if self.tolerance < 0.0 {
            let vertex_tolerance = BRepTool::vertex_tolerance(vertex)?;
            if vertex_tolerance > self.tolerance {
                self.tolerance = vertex_tolerance;
            }
        }
        Ok(())
    }
}

fn has_wire(face: &Face) -> Result<bool, Failure> {
    let shape = face.clone().into_shape();
    let mut children = ShapeIterator::new(&shape, false, true);
    while children.more() {
        if children.value()?.shape_type()? == ShapeEnum::Wire {
            return Ok(true);
        }
        children.next();
    }
    Ok(false)
}

fn vertex_bounds(face: &Face, tolerance: f64) -> Result<VertexBounds, Failure> {
    let mut bounds = VertexBounds::initial(tolerance);
    let mut vertices = Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Vertex,
        ShapeEnum::Shape,
    )?;
    while vertices.more() {
        include_vertex(vertices.current(), &mut bounds)?;
        vertices.next()?;
    }
    Ok(bounds)
}

fn include_vertex(
    shape: &rustycad_tkbrep::topods::Shape,
    bounds: &mut VertexBounds,
) -> Result<(), Failure> {
    let current = vertex(shape)?;
    if bounds.first_vertex.is_none() {
        bounds.first_vertex = Some(shape.clone());
    } else if bounds.same_vertex
        && !bounds
            .first_vertex
            .as_ref()
            .is_some_and(|first| first.is_same(shape))
    {
        bounds.same_vertex = false;
    }
    bounds.include(&current, BRepTool::pnt(&current)?)?;
    bounds.has_vertices = true;
    Ok(())
}

pub(super) fn update_bounds(
    minimum: &mut Point3,
    maximum: &mut Point3,
    point: Point3,
    has_points: bool,
) {
    let (min_x, max_x) = next_bounds(minimum.x(), maximum.x(), point.x(), has_points);
    let (min_y, max_y) = next_bounds(minimum.y(), maximum.y(), point.y(), has_points);
    let (min_z, max_z) = next_bounds(minimum.z(), maximum.z(), point.z(), has_points);
    minimum.set_coordinates(min_x, min_y, min_z);
    maximum.set_coordinates(max_x, max_y, max_z);
}

fn next_bounds(minimum: f64, maximum: f64, point: f64, has_points: bool) -> (f64, f64) {
    if !has_points {
        return (point, point);
    }
    let next_min = if minimum > point { point } else { minimum };
    let next_max = if maximum < point { point } else { maximum };
    (next_min, next_max)
}

pub(super) fn axis_is_small(extent: f64, tolerance: f64) -> bool {
    extent <= tolerance || precision::is_infinite(extent)
}

fn edge_samples_are_small(face: &Face, tolerance: f64) -> Result<bool, Failure> {
    let mut edges = Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )?;
    while edges.more() {
        if !edge_samples_are_small_one(edges.current(), tolerance)? {
            return Ok(false);
        }
        edges.next()?;
    }
    Ok(true)
}

fn edge_samples_are_small_one(
    shape: &rustycad_tkbrep::topods::Shape,
    tolerance: f64,
) -> Result<bool, Failure> {
    let edge_shape = edge(shape)?;
    let (curve, first, last) = BRepTool::curve(&edge_shape)?;
    let Some(curve) = curve.get() else {
        return Ok(true);
    };
    let start = curve.value(first)?;
    let middle = curve.value((first + last) / 2.0)?;
    Ok(start
        .square_distance(&middle)
        .partial_cmp(&(tolerance * tolerance))
        != Some(std::cmp::Ordering::Greater))
}

fn assign_spot_outputs(bounds: &VertexBounds, spot: &mut Point3, spot_tolerance: &mut f64) {
    *spot = Point3::new(
        (bounds.minimum.x() + bounds.maximum.x()) / 2.0,
        (bounds.minimum.y() + bounds.maximum.y()) / 2.0,
        (bounds.minimum.z() + bounds.maximum.z()) / 2.0,
    );
    *spot_tolerance = max_native(
        bounds.maximum.x() - bounds.minimum.x(),
        bounds.maximum.y() - bounds.minimum.y(),
        bounds.maximum.z() - bounds.minimum.z(),
    ) / 2.0;
}

fn max_native(first: f64, second: f64, third: f64) -> f64 {
    let first_two = if second > first { second } else { first };
    if third > first_two { third } else { first_two }
}
