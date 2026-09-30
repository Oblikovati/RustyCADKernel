// Rust translation of FoundationClasses/TKMath/gp/gp_Ax3.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::{Direction3, Failure, Point3, StandardDirection3, cross};
use rustycad_tkernel::standard::dump::{self, DumpOutput};
use std::io;
/// Origin and three frame directions; `Axis3::new(origin,Direction3::Z,Direction3::X)?`.
#[derive(Clone, Copy, Debug)]
pub struct Axis3 {
    location: Point3,
    direction: Direction3,
    x_direction: Direction3,
    y_direction: Direction3,
}
impl Default for Axis3 {
    fn default() -> Self {
        Self {
            location: Point3::default(),
            direction: Direction3::Z,
            x_direction: Direction3::X,
            y_direction: Direction3::Y,
        }
    }
}
impl Axis3 {
    pub(super) const fn from_stored(
        location: Point3,
        direction: Direction3,
        x_direction: Direction3,
        y_direction: Direction3,
    ) -> Self {
        Self {
            location,
            direction,
            x_direction,
            y_direction,
        }
    }

    /// Orthogonalizes the supplied X direction using native double-cross order; `Axis3::new(origin,normal,x)?`.
    pub fn new(location: Point3, direction: Direction3, x: Direction3) -> Result<Self, Failure> {
        let normal = direction.coordinates;
        let raw_x = cross(normal, cross(x.coordinates, normal));
        let x_direction =
            Direction3::normalize(raw_x, "gp_Dir::CrossCross() - result vector has zero norm")?;
        let y_direction = Direction3::normalize(
            cross(normal, x_direction.coordinates),
            "gp_Dir::Cross() - result vector has zero norm",
        )?;
        Ok(Self {
            location,
            direction,
            x_direction,
            y_direction,
        })
    }
    /// gp_Ax3(gp_Pnt, gp_Dir::D, gp_Dir::D): Y is the table cross product `N x Vx`, and no
    /// orthogonality is checked, so a parallel pair keeps the native fallback Y;
    /// `Axis3::with_standard_frame(origin,StandardDirection3::Z,StandardDirection3::X)`.
    pub const fn with_standard_frame(
        location: Point3,
        normal: StandardDirection3,
        x: StandardDirection3,
    ) -> Self {
        Self::from_stored(
            location,
            Direction3::from_standard(normal),
            Direction3::from_standard(x),
            Direction3::from_standard(normal.cross(x)),
        )
    }
    /// gp_Ax3(gp_Pnt, gp_Dir::D): X and Y are the native perpendicular table entries;
    /// `Axis3::with_standard(origin,StandardDirection3::Y)`.
    pub const fn with_standard(location: Point3, normal: StandardDirection3) -> Self {
        Self::from_stored(
            location,
            Direction3::from_standard(normal),
            Direction3::from_standard(normal.perpendicular_x()),
            Direction3::from_standard(normal.perpendicular_y()),
        )
    }
    /// gp_Ax3(gp_Dir::D) at the origin; `Axis3::from_standard(StandardDirection3::X)`.
    pub const fn from_standard(normal: StandardDirection3) -> Self {
        Self::with_standard(Point3::new(0.0, 0.0, 0.0), normal)
    }
    /// Returns the frame origin; `axis.location()`.
    pub const fn location(&self) -> Point3 {
        self.location
    }
    /// Returns the normal direction; `axis.direction()`.
    pub const fn direction(&self) -> Direction3 {
        self.direction
    }
    /// Writes all four native fields in order; `axis.dump_json(&mut stream)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput) -> io::Result<()> {
        dump::dump_vector(output, "Location", self.location.coordinates())?;
        dump::dump_vector(output, "Direction", self.direction.coordinates())?;
        dump::dump_vector(output, "XDirection", self.x_direction.coordinates())?;
        dump::dump_vector(output, "YDirection", self.y_direction.coordinates())
    }
    /// Parses independently normalized axes, preserving partial-update behavior; `axis.init_from_json(text,&mut position)?`.
    pub fn init_from_json(&mut self, text: &str, position: &mut i32) -> Result<bool, Failure> {
        let mut local = *position;
        let mut origin = [0.0; 3];
        if !dump::init_vector(text, "Location", &mut local, &mut origin)? {
            return Ok(false);
        }
        self.location = Point3 {
            coordinates: origin,
        };
        let mut directions = [[0.0; 3]; 3];
        for (name, coordinates) in ["Direction", "XDirection", "YDirection"]
            .into_iter()
            .zip(&mut directions)
        {
            if !dump::init_vector(text, name, &mut local, coordinates)? {
                return Ok(false);
            }
        }
        self.direction = Direction3::new(directions[0][0], directions[0][1], directions[0][2])?;
        self.x_direction = Direction3::new(directions[1][0], directions[1][1], directions[1][2])?;
        self.y_direction = Direction3::new(directions[2][0], directions[2][1], directions[2][2])?;
        let original_direction =
            Direction3::new(directions[0][0], directions[0][1], directions[0][2])?;
        if !self.direction.is_equal(&original_direction, 1e-12) {
            return Ok(false);
        }
        *position = local;
        Ok(true)
    }
}

mod directions;

mod transformations;

mod placement;
