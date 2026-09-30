//! Complex transition uses first-vertex derivatives for every grouped UV, as natively.
use super::*;
impl Crossing {
    pub(super) fn reset(&mut self) -> Result<(), Failure> {
        self.transition
            .reset(direction(self.direction)?, Direction3::new(1., 1., 1.)?, 0.);
        Ok(())
    }
    pub(super) fn reverse(&mut self) {
        self.direction.reverse();
        self.direction2d.reverse();
    }
    pub(super) fn compare(
        &mut self,
        vertex: Orientation,
        orientation: Orientation,
        mut boundary: Vector3,
        mut test: f64,
    ) -> Result<(), Failure> {
        test /= self.direction.magnitude();
        test /= self.normal.crossed(boundary).magnitude();
        let local = self.local_transition(test, orientation, &mut boundary);
        self.transition.compare(
            ANGULAR_TOLERANCE,
            direction(boundary)?,
            Direction3::new(1., 1., 1.)?,
            0.,
            local,
            vertex,
        );
        Ok(())
    }
    fn local_transition(
        &mut self,
        test: f64,
        orientation: Orientation,
        boundary: &mut Vector3,
    ) -> Orientation {
        if test.abs() <= ANGULAR_TOLERANCE {
            self.verify = true;
            return Orientation::External;
        }
        let local = if (test > 0. && orientation == Orientation::Forward)
            || (test < 0. && orientation == Orientation::Reversed)
        {
            Orientation::Forward
        } else {
            Orientation::Reversed
        };
        if orientation == Orientation::Reversed {
            boundary.reverse();
        }
        local
    }
    pub(super) fn accept(&mut self) -> bool {
        if self.passing {
            return true;
        }
        let before = self.transition.state_before();
        let after = self.transition.state_after();
        if before == State::Unknown || after == State::Unknown {
            return false;
        }
        if before != State::In {
            return after == State::In;
        }
        if after == State::In {
            self.passing = true;
        } else {
            self.reverse();
        }
        true
    }
}
fn direction(vector: Vector3) -> Result<Direction3, Failure> {
    Direction3::new(vector.x(), vector.y(), vector.z())
}
