// Rust translation of ModelingData/TKGeomBase/GProp/GProp_PEquation.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GProp_PEquation`: is a point cloud a point, a line, a plane or a volume?
use super::PGProps;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction3, Line3, Matrix3, Plane3, Point3, Vector3};
use rustycad_tkmath::math::Matrix;
use rustycad_tkmath::math_lin;

/// `GProp_PEquation::Type`: the entity that best fits the cloud.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PEquationType {
    /// Not computed (the eigen decomposition failed).
    #[default]
    None,
    /// All points coincide within the tolerance.
    Point,
    /// The points are collinear within the tolerance.
    Line,
    /// The points are coplanar within the tolerance.
    Plane,
    /// The points span the space.
    Space,
}

/// `GProp_PEquation`: principal-axis analysis of a point cloud; the extents along
/// the axes (from `MathLin::Jacobi` of the inertia matrix, sorted descending)
/// decide the dimension within the tolerance.
///
/// ```
/// use rustycad_tkernel::ncollection::Array1;
/// use rustycad_tkgeombase::gprop::PEquation;
/// use rustycad_tkmath::gp::Point3;
/// let mut points = Array1::with_bounds(1, 3).unwrap();
/// points[1] = Point3::new(0.0, 0.0, 0.0);
/// points[2] = Point3::new(1.0, 0.0, 0.0);
/// points[3] = Point3::new(2.0, 0.0, 0.0);
/// assert!(PEquation::new(&points, 1e-6).is_linear());
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PEquation {
    my_type: PEquationType,
    my_g: Point3,
    my_v1: Vector3,
    my_v2: Vector3,
    my_v3: Vector3,
    my_barycentre: Point3,
    my_axes: [Vector3; 3],
    my_extents: [f64; 3],
}

fn no_such_object(message: &str) -> Failure {
    Failure::with_type("Standard_NoSuchObject", message, "")
}

// The projection bounds of the cloud on each principal axis.
struct Projections {
    min: [f64; 3],
    max: [f64; 3],
}

impl PEquation {
    /// `GProp_PEquation(Pnts, Tol)`: analyze `points` with the tolerance `tol` on
    /// the extents; `PEquation::new(&points, 1e-6)`.
    pub fn new(points: &Array1<'_, Point3>, tol: f64) -> Self {
        let props = PGProps::from_points(points);
        let barycentre: Point3 = props.centre_of_mass();
        let mut result = Self {
            my_type: PEquationType::None,
            my_g: barycentre,
            my_v1: Vector3::default(),
            my_v2: Vector3::default(),
            my_v3: Vector3::default(),
            my_barycentre: barycentre,
            my_axes: [
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
                Vector3::new(0.0, 0.0, 1.0),
            ],
            my_extents: [0.0; 3],
        };
        let Some(axes) = principal_axes(props.matrix_of_inertia()) else {
            return result;
        };
        result.my_axes = axes;
        let projections: Projections = result.project(points);
        for k in 0..3 {
            result.my_extents[k] = projections.max[k] - projections.min[k];
        }
        result.classify(tol, &projections);
        result
    }

    // The projections of every point, relative to the barycentre, on each axis.
    fn project(&self, points: &Array1<'_, Point3>) -> Projections {
        let mut projections = Projections {
            min: [f64::MAX; 3],
            max: [-f64::MAX; 3],
        };
        for point in points.iter() {
            let dx: f64 = point.x() - self.my_g.x();
            let dy: f64 = point.y() - self.my_g.y();
            let dz: f64 = point.z() - self.my_g.z();
            for (k, axis) in self.my_axes.iter().enumerate() {
                let d: f64 = dx * axis.x() + dy * axis.y() + dz * axis.z();
                if d > projections.max[k] {
                    projections.max[k] = d;
                }
                if d < projections.min[k] {
                    projections.min[k] = d;
                }
            }
        }
        projections
    }

    // The native dimension count and code: each collapsed extent removes one
    // dimension; the code tells which axes collapsed.
    fn classify(&mut self, tol: f64, projections: &Projections) {
        let mut dimension: i32 = 3;
        let mut dim_code: i32 = 0;
        if self.my_extents[0].abs() <= tol {
            dimension -= 1;
            dim_code = 1;
        }
        if self.my_extents[1].abs() <= tol {
            dimension -= 1;
            dim_code = 2 * (dim_code + 1);
        }
        if self.my_extents[2].abs() <= tol {
            dimension -= 1;
            dim_code = 3 * (dim_code + 1);
        }
        match dimension {
            0 => self.my_type = PEquationType::Point,
            1 => self.set_line(dim_code),
            2 => self.set_plane(dim_code),
            _ => self.set_space(projections),
        }
    }

    // The line direction: the axis left by the two collapsed ones.
    fn set_line(&mut self, dim_code: i32) {
        self.my_type = PEquationType::Line;
        self.my_v1 = match dim_code {
            4 => self.my_axes[2],
            6 => self.my_axes[1],
            _ => self.my_axes[0],
        };
    }

    // The plane normal: the collapsed axis.
    fn set_plane(&mut self, dim_code: i32) {
        self.my_type = PEquationType::Plane;
        self.my_v1 = match dim_code {
            1 => self.my_axes[0],
            2 => self.my_axes[1],
            _ => self.my_axes[2],
        };
    }

    // The box: corner at the minimum projections, edges along the axes.
    fn set_space(&mut self, projections: &Projections) {
        self.my_type = PEquationType::Space;
        let [a1, a2, a3] = self.my_axes;
        let [min1, min2, min3] = projections.min;
        self.my_g
            .set_xyz(self.my_g.xyz() + min1 * *a1.xyz() + min2 * *a2.xyz() + min3 * *a3.xyz());
        self.my_v1 = self.my_extents[0] * a1;
        self.my_v2 = self.my_extents[1] * a2;
        self.my_v3 = self.my_extents[2] * a3;
    }

    /// `GetType()`: the fitted entity; `equation.get_type()`.
    pub fn get_type(&self) -> PEquationType {
        self.my_type
    }

    /// `IsPlanar()`; `equation.is_planar()`.
    pub fn is_planar(&self) -> bool {
        self.my_type == PEquationType::Plane
    }

    /// `IsLinear()`; `equation.is_linear()`.
    pub fn is_linear(&self) -> bool {
        self.my_type == PEquationType::Line
    }

    /// `IsPoint()`; `equation.is_point()`.
    pub fn is_point(&self) -> bool {
        self.my_type == PEquationType::Point
    }

    /// `IsSpace()`; `equation.is_space()`.
    pub fn is_space(&self) -> bool {
        self.my_type == PEquationType::Space
    }

    /// `Plane()`: the mean plane through the barycentre, Standard_NoSuchObject
    /// unless planar; `equation.plane()?`.
    pub fn plane(&self) -> Result<Plane3, Failure> {
        if !self.is_planar() {
            return Err(no_such_object(
                "GProp_PEquation::Plane: result is not planar",
            ));
        }
        Plane3::new(self.my_g, Direction3::from_vector(self.my_v1)?)
    }

    /// `Line()`: the mean line through the barycentre, Standard_NoSuchObject
    /// unless linear; `equation.line()?`.
    pub fn line(&self) -> Result<Line3, Failure> {
        if !self.is_linear() {
            return Err(no_such_object(
                "GProp_PEquation::Line: result is not linear",
            ));
        }
        Ok(Line3::new(self.my_g, Direction3::from_vector(self.my_v1)?))
    }

    /// `Point()`: the mean point, Standard_NoSuchObject unless a point;
    /// `equation.point()?`.
    pub fn point(&self) -> Result<Point3, Failure> {
        if !self.is_point() {
            return Err(no_such_object(
                "GProp_PEquation::Point: result is not a point",
            ));
        }
        Ok(self.my_g)
    }

    /// `Box(P, V1, V2, V3)`: the corner and the edges of the principal box,
    /// Standard_NoSuchObject unless the cloud spans the space;
    /// `let (p, v1, v2, v3) = equation.box_()?;`.
    pub fn box_(&self) -> Result<(Point3, Vector3, Vector3, Vector3), Failure> {
        if !self.is_space() {
            return Err(no_such_object(
                "GProp_PEquation::Box: result is not a 3D space",
            ));
        }
        Ok((self.my_g, self.my_v1, self.my_v2, self.my_v3))
    }

    /// `Barycentre()`: the centre of mass of the cloud; `equation.barycentre()`.
    pub fn barycentre(&self) -> Point3 {
        self.my_barycentre
    }

    /// `PrincipalAxis(Index)`: the unit axis `index` (1 to 3, by decreasing
    /// eigenvalue); Standard_OutOfRange otherwise (the native array read is
    /// unchecked); `equation.principal_axis(1)?`.
    pub fn principal_axis(&self, index: i32) -> Result<Vector3, Failure> {
        Ok(self.my_axes[axis_index(index, "PrincipalAxis")?])
    }

    /// `Extent(Index)`: the extent along axis `index` (1 to 3);
    /// Standard_OutOfRange otherwise; `equation.extent(1)?`.
    pub fn extent(&self, index: i32) -> Result<f64, Failure> {
        Ok(self.my_extents[axis_index(index, "Extent")?])
    }
}

fn axis_index(index: i32, member: &str) -> Result<usize, Failure> {
    if !(1..=3).contains(&index) {
        return Err(Failure::out_of_range(&format!(
            "GProp_PEquation::{member}: index {index}; expected in [1, 3]"
        )));
    }
    Ok(index as usize - 1)
}

// The eigenvectors of the inertia matrix sorted by decreasing eigenvalue, or None
// when MathLin::Jacobi fails.
fn principal_axes(inertia: Matrix3) -> Option<[Vector3; 3]> {
    let mut matrix: Matrix<'static> = Matrix::with_bounds(1, 3, 1, 3).unwrap();
    for i in 1..=3 {
        for j in 1..=3 {
            *matrix.change_value(i, j).unwrap() = inertia.value(i, j).unwrap();
        }
    }
    let eigen = math_lin::jacobi(&matrix, true);
    if !eigen.is_done() {
        return None;
    }
    let vectors: &Matrix<'static> = eigen.eigen_vectors.as_ref()?;
    let column = |c: usize| {
        let at = |r: usize| *vectors.at(r, c).unwrap();
        Vector3::new(at(0), at(1), at(2))
    };
    Some([column(0), column(1), column(2)])
}
