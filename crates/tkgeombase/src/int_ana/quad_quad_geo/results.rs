// Rust translation of the result accessors and DirToAx2 of IntAna_QuadQuadGeo.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The solutions of an [`QuadQuadGeo`] as gp elements.
use super::{QuadQuadGeo, domain_error};
use crate::int_ana::ResultType;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{
    Axis2, Circle3, Direction3, Ellipse3, Hyperbola3, Line3, Parabola3, Point3, Vector3,
};

impl QuadQuadGeo {
    /// Point(Num): the point solution `n` (of an IntAna_Point or IntAna_PointAndCircle
    /// result); the origin for another kind. StdFail_NotDone when not done,
    /// Standard_DomainError outside `1..=NbSolutions()` (or `n != 1` for
    /// PointAndCircle). `inter.point(1)?`.
    pub fn point(&self, n: i32) -> Result<Point3, Failure> {
        self.check_done("Point")?;
        if n > self.nbint || n < 1 {
            return Err(domain_error(
                "Point",
                &format!("index {n} outside [1, {}]", self.nbint),
            ));
        }
        match self.typeres {
            ResultType::PointAndCircle if n != 1 => Err(domain_error(
                "Point",
                &format!("index {n}; expected 1 for a point and a circle"),
            )),
            ResultType::PointAndCircle => Ok(if self.param1 == 0.0 {
                self.pt1
            } else {
                self.pt2
            }),
            ResultType::Point => Ok(if n == 1 { self.pt1 } else { self.pt2 }),
            _ => Ok(Point3::new(0.0, 0.0, 0.0)),
        }
    }

    /// Line(Num): the line solution `n`; StdFail_NotDone when not done,
    /// Standard_DomainError outside `1..=NbSolutions()` or for another kind.
    /// `inter.line(1)?`.
    pub fn line(&self, n: i32) -> Result<Line3, Failure> {
        self.check_done("Line")?;
        if n > self.nbint || n < 1 || self.typeres != ResultType::Line {
            return Err(self.kind_error("Line", n));
        }
        Ok(if n == 1 {
            Line3::new(self.pt1, self.dir1)
        } else {
            Line3::new(self.pt2, self.dir2)
        })
    }

    /// Circle(Num): the circle solution `n` (of up to four), in the frame that
    /// DirToAx2 builds around its axis; for PointAndCircle the circle whose radius
    /// is not null. StdFail_NotDone, Standard_DomainError outside the solutions.
    /// `inter.circle(1)?`.
    pub fn circle(&self, n: i32) -> Result<Circle3, Failure> {
        self.check_done("Circle")?;
        if self.typeres == ResultType::PointAndCircle {
            if n != 1 {
                return Err(domain_error(
                    "Circle",
                    &format!("index {n}; expected 1 for a point and a circle"),
                ));
            }
            if self.param2 == 0.0 {
                return Circle3::new(dir_to_ax2(self.pt1, self.dir1)?, self.param1);
            }
            return Circle3::new(dir_to_ax2(self.pt2, self.dir2)?, self.param2);
        }
        if n > self.nbint || n < 1 || self.typeres != ResultType::Circle {
            return Err(self.kind_error("Circle", n));
        }
        let (point, direction, radius) = match n {
            1 => (self.pt1, self.dir1, self.param1),
            2 => (self.pt2, self.dir2, self.param2),
            3 => (self.pt3, self.dir3, self.param3),
            _ => (self.pt4, self.dir4, self.param4),
        };
        Circle3::new(dir_to_ax2(point, direction)?, radius)
    }

    /// Ellipse(Num): the ellipse solution `n`, its radii ordered major first; the
    /// second ellipse swaps the frame directions. StdFail_NotDone,
    /// Standard_DomainError outside the solutions. `inter.ellipse(1)?`.
    pub fn ellipse(&self, n: i32) -> Result<Ellipse3, Failure> {
        self.check_done("Ellipse")?;
        if n > self.nbint || n < 1 || self.typeres != ResultType::Ellipse {
            return Err(self.kind_error("Ellipse", n));
        }
        let (point, main, x, r1, r2) = if n == 1 {
            (self.pt1, self.dir1, self.dir2, self.param1, self.param1bis)
        } else {
            (self.pt2, self.dir2, self.dir1, self.param2, self.param2bis)
        };
        let (major, minor) = if r1 < r2 { (r2, r1) } else { (r1, r2) };
        Ellipse3::new(Axis2::with_x_direction(point, main, x)?, major, minor)
    }

    /// Parabola(Num): the parabola solution; StdFail_NotDone, Standard_DomainError
    /// for another kind, Standard_OutOfRange unless `n == 1`. `inter.parabola(1)?`.
    pub fn parabola(&self, n: i32) -> Result<Parabola3, Failure> {
        self.check_done("Parabola")?;
        if self.typeres != ResultType::Parabola {
            return Err(self.kind_error("Parabola", n));
        }
        if n > self.nbint || n != 1 {
            return Err(Failure::out_of_range(&format!(
                "IntAna_QuadQuadGeo::Parabola() - index {n}; expected 1"
            )));
        }
        Parabola3::new(
            Axis2::with_x_direction(self.pt1, self.dir1, self.dir2)?,
            self.param1,
        )
    }

    /// Hyperbola(Num): the hyperbola branch `n`; the second branch reverses the X
    /// direction. StdFail_NotDone, Standard_DomainError outside the solutions.
    /// `inter.hyperbola(1)?`.
    pub fn hyperbola(&self, n: i32) -> Result<Hyperbola3, Failure> {
        self.check_done("Hyperbola")?;
        if n > self.nbint || n < 1 || self.typeres != ResultType::Hyperbola {
            return Err(self.kind_error("Hyperbola", n));
        }
        if n == 1 {
            let frame = Axis2::with_x_direction(self.pt1, self.dir1, self.dir2)?;
            return Hyperbola3::new(frame, self.param1, self.param1bis);
        }
        let frame = Axis2::with_x_direction(self.pt2, self.dir1, self.dir2.reversed())?;
        Hyperbola3::new(frame, self.param2, self.param2bis)
    }

    /// The Standard_DomainError of an index or a kind that has no such solution.
    fn kind_error(&self, member: &str, n: i32) -> Failure {
        domain_error(
            member,
            &format!(
                "index {n} of {} solutions of kind {}",
                self.nbint,
                self.typeres.name()
            ),
        )
    }
}

/// DirToAx2: the frame of main direction `d` at `p`, its X direction normal to `d`
/// in the plane of the two largest coordinates of `d`.
fn dir_to_ax2(p: Point3, d: Direction3) -> Result<Axis2, Failure> {
    let (x, y, z) = (d.x(), d.y(), d.z());
    let (ax, ay, az) = (x.abs(), y.abs(), z.abs());
    let x_vector: Vector3 = if (ax == 0.0) || ((ax < ay) && (ax < az)) {
        Vector3::new(0.0, -z, y)
    } else if (ay == 0.0) || ((ay < ax) && (ay < az)) {
        Vector3::new(-z, 0.0, x)
    } else {
        Vector3::new(-y, x, 0.0)
    };
    Axis2::with_x_direction(p, d, Direction3::from_vector(x_vector)?)
}
