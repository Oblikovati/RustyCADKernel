//! ComputeTangency in Contap_Contour.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! The reused PathPoint retains flags; SetValue replaces only its UV handle and point.
use crate::contap::{
    ContapHContTool, ContapHCurve2dTool, ContapSurfFunction, ContapSurfProps,
    ContapThePathPointOfTheSearch, ContapTheSearch,
};
use rustycad_tkernel::{ncollection::Array1, precision, standard::Failure};
use rustycad_tkg2d::geom2d::arrays::{at, put};
use rustycad_tkg3d::{
    adaptor3d::h_surface_tool::{self as surface_tool, Adaptor3dSurfaceHandle},
    top_abs::{Orientation, State},
};
use rustycad_tkgeomalgo::{
    int_start::{TheTopolTool, TheVertex},
    int_surf::IntSurfPathPoint,
    toptrans::CurveTransition,
};
use rustycad_tkmath::{
    gp::{Direction2, Direction3, Point2, Vector2, Vector3},
    math::{Matrix, Vector},
};
mod grouping;
mod transition;
const ANGULAR_TOLERANCE: f64 = 5e-6;
struct TangencyWalk<'a, 'b> {
    search: &'a ContapTheSearch,
    domain: &'a (dyn TheTopolTool + 'static),
    function: &'a mut ContapSurfFunction,
    output: &'a mut Vec<IntSurfPathPoint>,
    destination: &'a mut Array1<'b, i32>,
    surface: Adaptor3dSurfaceHandle,
    point: IntSurfPathPoint,
    appended: i32,
    count: i32,
    parameters: Vector<'static>,
    value: Vector<'static>,
    gradient: Matrix<'static>,
}
pub(super) fn compute_tangency(
    search: &ContapTheSearch,
    domain: &(dyn TheTopolTool + 'static),
    function: &mut ContapSurfFunction,
    output: &mut Vec<IntSurfPathPoint>,
    destination: &mut Array1<'_, i32>,
) -> Result<(), Failure> {
    let mut walk = TangencyWalk::new(search, domain, function, output, destination)?;
    for index in 1..=walk.count {
        if at(walk.destination, index)? == 0 {
            walk.start(index)?;
        }
    }
    Ok(())
}
impl<'a, 'b> TangencyWalk<'a, 'b> {
    fn new(
        search: &'a ContapTheSearch,
        domain: &'a (dyn TheTopolTool + 'static),
        function: &'a mut ContapSurfFunction,
        output: &'a mut Vec<IntSurfPathPoint>,
        destination: &'a mut Array1<'b, i32>,
    ) -> Result<Self, Failure> {
        let count = search.nb_points()?;
        Ok(Self {
            search,
            domain,
            surface: function.surface(),
            function,
            output,
            destination,
            point: IntSurfPathPoint::new(),
            appended: 0,
            count,
            parameters: Vector::with_bounds(1, 2)?,
            value: Vector::with_bounds(1, 1)?,
            gradient: Matrix::with_bounds(1, 1, 1, 2).unwrap(),
        })
    }

    fn start(&mut self, index: i32) -> Result<(), Failure> {
        let start = self.search.point(index)?.clone();
        let arc = start.arc();
        let projected = ContapHCurve2dTool::value(&arc, start.parameter())?;
        if self.on_solution(&start, projected)? {
            return Ok(());
        }
        let orientation = self.domain.arc_orientation(&arc)?;
        let passing = unoriented(orientation);
        let (uv, tangent) = ContapHCurve2dTool::d1(&arc, start.parameter())?;
        self.parameters[1] = uv.x();
        self.parameters[2] = uv.y();
        self.point.set_value(start.value(), uv.x(), uv.y());
        self.function
            .values(&self.parameters, &mut self.value, &mut self.gradient)?;
        if self.function.is_tangent()? {
            return self.tangent_point(index, &start, passing);
        }
        self.regular_point(index, &start, orientation, passing, uv, tangent)
    }
    fn on_solution(
        &self,
        start: &ContapThePathPointOfTheSearch,
        uv: Point2,
    ) -> Result<bool, Failure> {
        for index in 1..=self.search.nb_segments()? {
            let arc = self.search.segment(index)?.curve();
            if let Some((_, projected)) = ContapHContTool::project(&arc, uv)? {
                let point = surface_tool::value(&self.surface, projected.x(), projected.y())?;
                if start.value().distance(&point) <= precision::confusion() {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
    fn append(&mut self, passing: bool) {
        self.point.set_passing(passing);
        self.output.push(self.point.clone());
        self.appended += 1;
    }
    fn regular_point(
        &mut self,
        index: i32,
        start: &ContapThePathPointOfTheSearch,
        orientation: Orientation,
        passing: bool,
        uv: Point2,
        tangent: Vector2,
    ) -> Result<(), Failure> {
        let mut group = self.crossing(uv, passing)?;
        let boundary = restriction_tangent(tangent, group.du, group.dv);
        if group.normal.square_magnitude() < f64::EPSILON {
            return Ok(());
        }
        let test = group.direction.dot(group.normal.crossed(boundary));
        if start.is_new() {
            return self.new_point(index, orientation, boundary, test, &mut group);
        }
        self.vertex_point(index, start, orientation, boundary, test, &mut group)
    }
    fn crossing(&mut self, uv: Point2, passing: bool) -> Result<Crossing, Failure> {
        let direction = self.function.direction3d()?;
        let direction2d = self.function.direction2d()?;
        let (_, du, dv, normal) = ContapSurfProps::deriv_and_norm(&self.surface, uv.x(), uv.y())?;
        Ok(Crossing {
            direction,
            direction2d,
            du,
            dv,
            normal,
            passing,
            verify: false,
            transition: CurveTransition::new(),
        })
    }
    fn new_point(
        &mut self,
        index: i32,
        orientation: Orientation,
        boundary: Vector3,
        test: f64,
        group: &mut Crossing,
    ) -> Result<(), Failure> {
        let parallel = group.direction.normalized()?.dot(boundary.normalized()?);
        if parallel.abs() < 1. - ANGULAR_TOLERANCE {
            if (test < 0. && orientation == Orientation::Forward)
                || (test > 0. && orientation == Orientation::Reversed)
            {
                group.reverse();
            }
            self.point
                .set_directions(group.direction, group.direction2d);
        } else {
            self.point.set_tangency(true);
        }
        put(self.destination, index, self.appended + 1)?;
        self.append(group.passing);
        Ok(())
    }
}
fn unoriented(orientation: Orientation) -> bool {
    orientation == Orientation::Internal || orientation == Orientation::External
}
fn restriction_tangent(tangent: Vector2, du: Vector3, dv: Vector3) -> Vector3 {
    du * tangent.x() + dv * tangent.y()
}
struct Crossing {
    direction: Vector3,
    direction2d: Direction2,
    du: Vector3,
    dv: Vector3,
    normal: Vector3,
    passing: bool,
    verify: bool,
    transition: CurveTransition,
}
