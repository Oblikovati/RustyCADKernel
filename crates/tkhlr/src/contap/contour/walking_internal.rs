//! Native walking-line internal vertices and index shifts after inserted points.
use super::walking_root::{Uv, WalkingRoot, view_cross};
use crate::contap::{ContapIType, ContapLine, ContapPoint, ContapSurfFunction};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::int_surf::IntSurfPntOn2S;
use rustycad_tkmath::gp::{self, Vector2, Vector3};
struct WalkingInternal<'a> {
    line: &'a ContapLine,
    function: &'a mut ContapSurfFunction,
    root: WalkingRoot,
    diagnostics: &'a mut dyn std::io::Write,
    lower_index: i32,
    upper_index: i32,
    count: i32,
    parameter: f64,
}
pub(super) fn compute_walking_internal(
    line: &ContapLine,
    function: &mut ContapSurfFunction,
    u_tolerance: f64,
    v_tolerance: f64,
    diagnostics: &mut dyn std::io::Write,
) -> Result<(), Failure> {
    if line.type_contour() != ContapIType::Walking {
        return Ok(());
    }
    let root = WalkingRoot::new(function, u_tolerance, v_tolerance)?;
    let mut search = WalkingInternal {
        line,
        function,
        root,
        diagnostics,
        lower_index: 1,
        upper_index: 2,
        count: line.nb_pnts()?,
        parameter: 0.,
    };
    search.perform()
}
impl WalkingInternal<'_> {
    fn uv(&self, index: i32) -> Result<Uv, Failure> {
        let (u, v) = self.line.point(index)?.parameters_on_s2();
        Ok([u, v])
    }
    fn sample(&mut self, index: i32) -> Result<(Uv, Vector3), Failure> {
        let uv = self.uv(index)?;
        self.root.evaluate(self.function, uv)?;
        Ok((
            uv,
            view_cross(self.function, self.line.point(index)?.value())?,
        ))
    }
    fn initial_reference(&mut self) -> Result<Option<(Uv, Vector3)>, Failure> {
        loop {
            let (uv, cross) = self.sample(self.lower_index)?;
            let negligible = cross.magnitude() <= gp::resolution();
            if !negligible {
                return Ok(Some((uv, cross)));
            }
            self.lower_index += 1;
            if self.lower_index > self.count {
                return Ok(None);
            }
        }
    }
    fn perform(&mut self) -> Result<(), Failure> {
        let Some((mut lower, mut reference)) = self.initial_reference()? else {
            return Ok(());
        };
        self.upper_index = self.lower_index + 1;
        while self.upper_index <= self.count {
            let (mut upper, current) = self.sample(self.upper_index)?;
            if current.magnitude() <= gp::resolution() {
                self.upper_index += 1;
                continue;
            }
            if current.dot(reference) < 0. {
                self.search_interval(lower, upper, reference)?;
                upper = self.uv(self.upper_index)?;
            }
            reference = current;
            self.lower_index = self.upper_index;
            self.upper_index += 1;
            lower = upper;
        }
        Ok(())
    }
    fn search_interval(&mut self, lower: Uv, upper: Uv, reference: Vector3) -> Result<(), Failure> {
        let Some(uv) = self
            .root
            .solve(self.function, lower, upper, reference, self.diagnostics)?
        else {
            return Ok(());
        };
        let inserted = self.locate(uv)?;
        let create = if inserted {
            true
        } else {
            !self.mark_existing()?
        };
        if create && self.parameter > 1. && self.parameter < f64::from(self.count) {
            let mut point = ContapPoint::with_value(self.function.point(), uv[0], uv[1]);
            point.set_parameter(self.parameter);
            point.set_internal();
            self.line.add_vertex(&point);
        }
        Ok(())
    }
    fn locate(&mut self, uv: Uv) -> Result<bool, Failure> {
        let initial = self.uv(self.lower_index)?;
        if self.root.near(uv, initial) {
            self.parameter = f64::from(self.lower_index);
            return Ok(false);
        }
        let mut previous = Vector2::new(uv[0] - initial[0], uv[1] - initial[1]);
        for index in self.lower_index + 1..=self.upper_index {
            let current = self.uv(index)?;
            if self.root.near(uv, current) {
                self.parameter = f64::from(index);
                return Ok(false);
            }
            let next = Vector2::new(uv[0] - current[0], uv[1] - current[1]);
            if previous.dot(next) < 0. {
                self.insert(index, uv)?;
                return Ok(true);
            }
            previous = next;
        }
        Ok(false)
    }
    fn insert(&mut self, index: i32, uv: Uv) -> Result<(), Failure> {
        self.parameter = f64::from(index);
        let mut point = IntSurfPntOn2S::new();
        point.set_value_on(self.function.point(), false, uv[0], uv[1]);
        self.line.line_on_2s().insert_before(index, &point)?;
        for rank in 1..=self.line.nb_vertex() {
            self.line.change_vertex(rank, |vertex| {
                let parameter = vertex.parameter_on_line();
                if parameter >= f64::from(index) {
                    vertex.set_parameter(parameter + 1.);
                }
            })?;
        }
        self.count += 1;
        self.upper_index += 1;
        Ok(())
    }
    fn mark_existing(&self) -> Result<bool, Failure> {
        let mut found = false;
        for index in 1..=self.line.nb_vertex() {
            if self.line.vertex(index)?.parameter_on_line() == self.parameter {
                self.line.change_vertex(index, ContapPoint::set_internal)?;
                found = true;
            }
        }
        Ok(found)
    }
}
