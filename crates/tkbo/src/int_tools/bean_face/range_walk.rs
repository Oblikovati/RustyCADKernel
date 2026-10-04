//! Native adaptive stepping preserves flagged-neighbour and stagnation stopping rules.
use super::{BeanFaceIntersector, Failure, Range};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::extrema::GenLocateExtPS;

struct RangeWalk {
    increasing: bool,
    current: f64,
    previous: f64,
    step: f64,
    minimum: f64,
    limit: f64,
    range: Range,
    index: i32,
    boundary: bool,
    end_index: bool,
    valid_index: bool,
    loops: i32,
    found: bool,
    uv: (f64, f64),
}

impl BeanFaceIntersector {
    /// Extends a seed through unclassified ranges; `solver.compute_range_from_start_point(true,t,u,v)?`.
    pub fn compute_range_from_start_point(
        &mut self,
        increasing: bool,
        parameter: f64,
        u: f64,
        v: f64,
    ) -> Result<(), Failure> {
        let index = self
            .range_manager
            .get_index_with_side(parameter, increasing)?;
        if index == 0 {
            return Ok(());
        }
        self.compute_range_from_index(increasing, parameter, u, v, index)
    }

    /// Extends a seed from an explicit interval; `solver.compute_range_from_index(true,t,u,v,index)?`.
    pub fn compute_range_from_index(
        &mut self,
        increasing: bool,
        parameter: f64,
        u: f64,
        v: f64,
        index: i32,
    ) -> Result<(), Failure> {
        if self.range_manager.flag(index)? > 0 {
            return Ok(());
        }
        let mut walk = self.start_range_walk(increasing, parameter, (u, v), index)?;
        while walk.step >= walk.minimum && walk.loops <= 10 {
            let found = self.locate_walk_point(&mut walk)?;
            if walk.accept(found) || !walk.advance(found) {
                break;
            }
            self.walk_boundary(&mut walk, found)?;
        }
        if walk.found {
            let (first, last) = if increasing {
                (parameter, walk.previous)
            } else {
                (walk.previous, parameter)
            };
            self.range_manager.insert_range(first, last, 2)?;
        }
        Ok(())
    }

    fn start_range_walk(
        &self,
        increasing: bool,
        parameter: f64,
        uv: (f64, f64),
        index: i32,
    ) -> Result<RangeWalk, Failure> {
        let limit = 0.1 * (self.range.last() - self.range.first());
        let mut minimum = self.curve_resolution * 0.5;
        if minimum > limit {
            minimum = limit * 0.5;
        }
        let mut walk = RangeWalk {
            increasing,
            current: parameter,
            previous: parameter,
            step: self.curve_resolution,
            minimum,
            limit,
            range: self.range_manager.range(index)?,
            index,
            boundary: false,
            end_index: false,
            valid_index: true,
            loops: 0,
            found: false,
            uv,
        };
        walk.current = walk.shifted();
        if walk.outside() {
            walk.current = walk.endpoint();
        }
        Ok(walk)
    }

    fn locate_walk_point(&self, walk: &mut RangeWalk) -> Result<bool, Failure> {
        let point = self.curve.get().expect("bean curve").value(walk.current)?;
        let mut extrema =
            GenLocateExtPS::new(self.surface.get().expect("bean surface"), 1e-10, 1e-10);
        extrema.perform(point, walk.uv.0, walk.uv.1, false)?;
        if !extrema.is_done() {
            return Ok(self.distance(walk.current)? < self.criteria);
        }
        if extrema.square_distance()? < self.criteria * self.criteria {
            walk.uv = extrema.point()?.parameter();
            return Ok(true);
        }
        Ok(false)
    }

    fn walk_boundary(&self, walk: &mut RangeWalk, found: bool) -> Result<(), Failure> {
        walk.boundary = walk.outside();
        walk.end_index = false;
        walk.valid_index = true;
        if !walk.boundary {
            return Ok(());
        }
        walk.end_index = (!walk.increasing && walk.index == 1)
            || (walk.increasing && walk.index == self.range_manager.length());
        if walk.end_index {
            walk.current = walk.endpoint();
        } else if found {
            self.walk_neighbour(walk)?;
        }
        walk.loops = if walk.step < walk.minimum * 10. {
            walk.loops + 1
        } else {
            0
        };
        Ok(())
    }

    fn walk_neighbour(&self, walk: &mut RangeWalk) -> Result<(), Failure> {
        let next = walk.index + if walk.increasing { 1 } else { -1 };
        if self.range_manager.flag(next)? != 0 {
            walk.valid_index = false;
            walk.current = walk.endpoint();
            return Ok(());
        }
        walk.index = next;
        walk.range = self.range_manager.range(next)?;
        if walk.outside() {
            walk.current = (walk.range.first() + walk.range.last()) * 0.5;
            walk.step *= 0.5;
        }
        Ok(())
    }
}

impl RangeWalk {
    fn shifted(&self) -> f64 {
        if self.increasing {
            return self.previous + self.step;
        }
        self.previous - self.step
    }

    fn outside(&self) -> bool {
        if self.increasing {
            return self.current > self.range.last();
        }
        self.current < self.range.first()
    }

    fn endpoint(&self) -> f64 {
        if self.increasing {
            return self.range.last();
        }
        self.range.first()
    }

    fn accept(&mut self, found: bool) -> bool {
        if !found {
            self.limit = self.step;
            return false;
        }
        self.previous = self.current;
        self.found = true;
        self.boundary && (self.end_index || !self.valid_index)
    }

    fn advance(&mut self, found: bool) -> bool {
        self.step = if found {
            self.step * 2.
        } else {
            self.step * 0.5
        };
        self.step = if self.step < self.limit {
            self.step
        } else {
            self.limit
        };
        self.current = self.shifted();
        self.current != self.previous
    }
}
