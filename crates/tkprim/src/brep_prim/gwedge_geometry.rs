// Rust translation of BRepPrim_GWedge.cxx (Plane, Line, Point) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The geometry of the faces, edges and vertices of `BRepPrim_GWedge`.
use super::BRepPrimDirection;
use super::gwedge::{BRepPrimGWedge, num_dir1, num_dir2, num_dir3, wedge_domain_error};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis1, Direction3, Line3, Plane3, Point3, Vector3};

impl BRepPrimGWedge {
    /// The frame axes as vectors (`VX`, `VY`, `VZ` of the native bodies).
    fn frame_vectors(&self) -> [Vector3; 3] {
        [
            Vector3::from(self.my_axes.x_direction()),
            Vector3::from(self.my_axes.y_direction()),
            Vector3::from(self.my_axes.direction()),
        ]
    }

    /// The point `(X, Y, Z)` of the frame: the location translated along each axis in
    /// turn, as the native bodies do.
    fn frame_point(&self, x: f64, y: f64, z: f64) -> Point3 {
        let [vx, vy, vz] = self.frame_vectors();
        let mut point = self.my_axes.location();
        point.translate(x * vx);
        point.translate(y * vy);
        point.translate(z * vz);
        point
    }

    /// `Plane(d1)`: the plane of the face in direction `d1`, through its first corner;
    /// the XMin, XMax, ZMin and ZMax planes lean when the YMax face is narrower;
    /// Standard_ConstructionError (from gp_Dir) for a null normal;
    /// `wedge.plane(BRepPrimDirection::ZMin)?`.
    pub fn plane(&self, d1: BRepPrimDirection) -> Result<Plane3, Failure> {
        let i = num_dir1(d1);
        let [vx, vy, vz] = self.frame_vectors();
        let mut normal = match i / 2 {
            0 => self.my_axes.x_direction(),
            1 => self.my_axes.y_direction(),
            _ => self.my_axes.direction(),
        };
        let (x, y, z) = match i {
            0 => {
                if self.x2min != self.xmin {
                    normal = Direction3::from_vector(
                        (self.ymax - self.ymin) * vx + (self.xmin - self.x2min) * vy,
                    )?;
                }
                (self.xmin, self.ymin, self.zmin)
            }
            1 => {
                if self.x2max != self.xmax {
                    normal = Direction3::from_vector(
                        (self.ymax - self.ymin) * vx + (self.xmax - self.x2max) * vy,
                    )?;
                }
                (self.xmax, self.ymin, self.zmin)
            }
            2 => (self.xmin, self.ymin, self.zmin),
            3 => (self.xmin, self.ymax, self.zmin),
            4 => {
                if self.z2min != self.zmin {
                    normal = Direction3::from_vector(
                        (self.ymax - self.ymin) * vz + (self.zmin - self.z2min) * vy,
                    )?;
                }
                (self.xmin, self.ymin, self.zmin)
            }
            _ => {
                if self.z2max != self.zmax {
                    normal = Direction3::from_vector(
                        (self.ymax - self.ymin) * vz + (self.zmax - self.z2max) * vy,
                    )?;
                }
                (self.xmin, self.ymin, self.zmax)
            }
        };
        Plane3::new(self.frame_point(x, y, z), normal)
    }

    /// `Line(d1, d2)`: the line of the edge between the faces `d1` and `d2`, from its
    /// first vertex; the edges between the X and Z faces lean when the YMax face is
    /// shifted; Standard_DomainError when the edge does not exist;
    /// `wedge.line(BRepPrimDirection::XMin, BRepPrimDirection::YMin)?`.
    pub fn line(&self, d1: BRepPrimDirection, d2: BRepPrimDirection) -> Result<Line3, Failure> {
        if !self.has_edge(d1, d2)? {
            return Err(missing_edge("Line", d1, d2));
        }
        let i = num_dir2(d1, d2)?;
        let direction = match i / 4 {
            0 => self.my_axes.direction(),
            1 => self.my_axes.x_direction(),
            _ => self.my_axes.y_direction(),
        };
        let (point, direction) = self.edge_origin(i, direction)?;
        Ok(Line3::from_axis(Axis1::new(point, direction)))
    }

    /// The first vertex and the direction of the edge of index `i`; the four edges
    /// between the X and Z faces (8 to 11) take the direction to the YMax face when it
    /// is shifted.
    fn edge_origin(&self, i: usize, axis: Direction3) -> Result<(Point3, Direction3), Failure> {
        let (x, y, z) = match i {
            0 | 4 | 8 => (self.xmin, self.ymin, self.zmin),
            1 | 6 => (self.x2min, self.ymax, self.z2min),
            2 | 10 => (self.xmax, self.ymin, self.zmin),
            3 => (self.x2max, self.ymax, self.z2min),
            5 | 9 => (self.xmin, self.ymin, self.zmax),
            7 => (self.x2min, self.ymax, self.z2max),
            _ => (self.xmax, self.ymin, self.zmax),
        };
        let direction = match i {
            8 => self.leaning_direction(axis, self.xmin, self.x2min, self.zmin, self.z2min)?,
            9 => self.leaning_direction(axis, self.xmin, self.x2min, self.zmax, self.z2max)?,
            10 => self.leaning_direction(axis, self.xmax, self.x2max, self.zmin, self.z2min)?,
            11 => self.leaning_direction(axis, self.xmax, self.x2max, self.zmax, self.z2max)?,
            _ => axis,
        };
        Ok((self.frame_point(x, y, z), direction))
    }

    /// The direction of an edge from the YMin face (`x`, `z`) to the YMax face (`x2`,
    /// `z2`): `axis` unless the YMax face is shifted.
    fn leaning_direction(
        &self,
        axis: Direction3,
        x: f64,
        x2: f64,
        z: f64,
        z2: f64,
    ) -> Result<Direction3, Failure> {
        if x == x2 && z == z2 {
            return Ok(axis);
        }
        let [vx, vy, vz] = self.frame_vectors();
        Direction3::from_vector((x2 - x) * vx + (self.ymax - self.ymin) * vy + (z2 - z) * vz)
    }

    /// `Point(d1, d2, d3)`: the point of the vertex of the faces `d1`, `d2`, `d3`;
    /// Standard_DomainError when the vertex does not exist or two directions share an
    /// axis; `wedge.point(XMin, YMin, ZMin)?`.
    pub fn point(
        &self,
        d1: BRepPrimDirection,
        d2: BRepPrimDirection,
        d3: BRepPrimDirection,
    ) -> Result<Point3, Failure> {
        if !self.has_vertex(d1, d2, d3) {
            return Err(missing_vertex("Point", d1, d2, d3));
        }
        let (x, y, z) = match num_dir3(d1, d2, d3)? {
            0 => (self.xmin, self.ymin, self.zmin),
            1 => (self.xmin, self.ymin, self.zmax),
            2 => (self.x2min, self.ymax, self.z2min),
            3 => (self.x2min, self.ymax, self.z2max),
            4 => (self.xmax, self.ymin, self.zmin),
            5 => (self.xmax, self.ymin, self.zmax),
            6 => (self.x2max, self.ymax, self.z2min),
            _ => (self.x2max, self.ymax, self.z2max),
        };
        Ok(self.frame_point(x, y, z))
    }
}

/// The Standard_DomainError of a member on an edge that does not exist.
pub(crate) fn missing_edge(member: &str, d1: BRepPrimDirection, d2: BRepPrimDirection) -> Failure {
    wedge_domain_error(
        member,
        &format!(
            "no edge between {d1:?} and {d2:?} (an open direction or a null YMax extent); expected an existing edge"
        ),
    )
}

/// The Standard_DomainError of a member on a vertex that does not exist.
pub(crate) fn missing_vertex(
    member: &str,
    d1: BRepPrimDirection,
    d2: BRepPrimDirection,
    d3: BRepPrimDirection,
) -> Failure {
    wedge_domain_error(
        member,
        &format!(
            "no vertex at {d1:?} {d2:?} {d3:?} (an open direction); expected an existing vertex"
        ),
    )
}
