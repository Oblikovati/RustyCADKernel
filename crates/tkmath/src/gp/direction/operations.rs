use super::super::{
    Axis1, Axis2, Direction3, Failure, Transform3, TransformForm, Vector3, Xyz, cross, dot, reflect,
};
use rustycad_tkernel::standard::dump::{self, DumpOutput};
use std::io;
impl Direction3 {
    /// Constructs from coordinate algebra with the native constructor checks; `Direction3::from_xyz(xyz)?`.
    pub fn from_xyz(xyz: Xyz) -> Result<Self, Failure> {
        Self::new(xyz.x(), xyz.y(), xyz.z())
    }
    /// Constructs from vector magnitude; `Direction3::from_vector(vector)?`.
    pub fn from_vector(vector: Vector3) -> Result<Self, Failure> {
        Self::from_xyz(*vector.xyz())
    }
    /// Reads X; `direction.x()`.
    pub const fn x(&self) -> f64 {
        self.coordinates[0]
    }
    /// Reads Y; `direction.y()`.
    pub const fn y(&self) -> f64 {
        self.coordinates[1]
    }
    /// Reads Z; `direction.z()`.
    pub const fn z(&self) -> f64 {
        self.coordinates[2]
    }
    /// Signs the angle by the native scalar triple product; `direction.angle_with_ref(other,reference)`.
    pub fn angle_with_ref(self, other: Self, reference: Self) -> f64 {
        let angle = self.angle(&other);
        let normal = cross(self.coordinates, other.coordinates);
        if dot(normal, reference.coordinates) >= 0.0 {
            angle
        } else {
            -angle
        }
    }
    /// Compares with a right angle; `direction.is_normal(other,1e-12)`.
    pub fn is_normal(self, other: Self, tolerance: f64) -> bool {
        (std::f64::consts::FRAC_PI_2 - self.angle(&other)).abs() <= tolerance
    }
    /// Compares with an opposite direction; `direction.is_opposite(other,1e-12)`.
    pub fn is_opposite(self, other: Self, tolerance: f64) -> bool {
        std::f64::consts::PI - self.angle(&other) <= tolerance
    }
    /// Accepts parallel or antiparallel directions; `direction.is_parallel(other,1e-12)`.
    pub fn is_parallel(self, other: Self, tolerance: f64) -> bool {
        let angle = self.angle(&other);
        angle <= tolerance || std::f64::consts::PI - angle <= tolerance
    }
    /// Computes a normalized cross product; `direction.crossed(other)?`.
    pub fn crossed(mut self, other: Self) -> Result<Self, Failure> {
        self.cross(other)?;
        Ok(self)
    }
    /// Mutates before checking zero norm, preserving native failure state; `direction.cross(other)?`.
    pub fn cross(&mut self, other: Self) -> Result<(), Failure> {
        self.coordinates = cross(self.coordinates, other.coordinates);
        *self = Self::normalize(
            self.coordinates,
            "gp_Dir::Cross() - result vector has zero norm",
        )?;
        Ok(())
    }
    /// Computes a normalized nested vector product; `direction.cross_crossed(first,second)?`.
    pub fn cross_crossed(mut self, first: Self, second: Self) -> Result<Self, Failure> {
        self.cross_cross(first, second)?;
        Ok(self)
    }
    /// Retains the unnormalized product when normalization fails; `direction.cross_cross(first,second)?`.
    pub fn cross_cross(&mut self, first: Self, second: Self) -> Result<(), Failure> {
        self.coordinates = cross(
            self.coordinates,
            cross(first.coordinates, second.coordinates),
        );
        *self = Self::normalize(
            self.coordinates,
            "gp_Dir::CrossCross() - result vector has zero norm",
        )?;
        Ok(())
    }
    /// Computes scalar product; `direction.dot(other)`.
    pub fn dot(self, other: Self) -> f64 {
        dot(self.coordinates, other.coordinates)
    }
    /// Computes scalar triple product without intermediate normalization; `direction.dot_cross(first,second)`.
    pub fn dot_cross(self, first: Self, second: Self) -> f64 {
        dot(
            self.coordinates,
            cross(first.coordinates, second.coordinates),
        )
    }
    /// Reverses every stored component; `direction.reversed()`.
    pub fn reversed(self) -> Self {
        Self {
            coordinates: self.coordinates.map(|v| -v),
        }
    }
    /// Reverses this direction in place; `direction.reverse()`.
    pub fn reverse(&mut self) {
        *self = self.reversed();
    }
    /// Mirrors about a direction without normalization; `direction.mirrored_direction(other)`.
    pub fn mirrored_direction(self, other: Self) -> Self {
        self.mirror_coefficients(other.coordinates)
    }
    /// Mirrors this direction about another; `direction.mirror_direction(other)`.
    pub fn mirror_direction(&mut self, other: Self) {
        *self = self.mirrored_direction(other);
    }
    /// Preserves the pinned gp_Dir::Mirror(gp_Ax1) Y-for-Z behavior; `direction.mirrored_axis(axis)`.
    pub fn mirrored_axis(self, axis: Axis1) -> Self {
        let [x, y, _z] = axis.direction().coordinates;
        // OCCT 3d097a gp_Dir.cxx assigns both B and C from XYZ.Y(); do not silently correct it.
        self.mirror_coefficients([x, y, y])
    }
    /// Mirrors this direction about an axis; `direction.mirror_axis(axis)`.
    pub fn mirror_axis(&mut self, axis: Axis1) {
        *self = self.mirrored_axis(axis);
    }
    /// Mirrors through the frame's XY plane; `direction.mirrored_plane(frame)`.
    pub fn mirrored_plane(self, frame: Axis2) -> Self {
        self.mirrored_direction(frame.direction()).reversed()
    }
    /// Mirrors this direction through a frame's plane; `direction.mirror_plane(frame)`.
    pub fn mirror_plane(&mut self, frame: Axis2) {
        *self = self.mirrored_plane(frame);
    }
    fn mirror_coefficients(self, normal: [f64; 3]) -> Self {
        Self {
            coordinates: reflect(normal, self.coordinates),
        }
    }
    /// Rotates without the extra normalization performed by Transform; `direction.rotated(axis,angle)?`.
    pub fn rotated(self, axis: Axis1, angle: f64) -> Result<Self, Failure> {
        let xyz = Transform3::rotation(axis, angle)?
            .matrix()
            .apply(Xyz::from(self));
        Ok(Self {
            coordinates: *xyz.coordinates(),
        })
    }
    /// Rotates this direction in place; `direction.rotate(axis,angle)?`.
    pub fn rotate(&mut self, axis: Axis1, angle: f64) -> Result<(), Failure> {
        *self = self.rotated(axis, angle)?;
        Ok(())
    }
    /// Applies rigid transforms and native unconditional renormalization; `direction.transformed(transform)`.
    pub fn transformed(self, transform: Transform3) -> Self {
        if matches!(
            transform.form(),
            TransformForm::Identity | TransformForm::Translation
        ) {
            return self;
        }
        if transform.form() == TransformForm::PointMirror {
            return self.reversed();
        }
        if transform.form() == TransformForm::Scale {
            return if transform.scale_factor() < 0.0 {
                self.reversed()
            } else {
                self
            };
        }
        let xyz = transform.matrix().apply(Xyz::from(self));
        let result = Self {
            coordinates: *(xyz / xyz.modulus()).coordinates(),
        };
        if transform.scale_factor() < 0.0 {
            result.reversed()
        } else {
            result
        }
    }

    /// Transforms this direction in place; `direction.transform(transform)`.
    pub fn transform(&mut self, transform: Transform3) {
        *self = self.transformed(transform);
    }
    /// Writes native stored components, including post-failure values; `direction.dump_json(&mut stream)?`.
    pub fn dump_json(&self, stream: &mut impl DumpOutput) -> io::Result<()> {
        dump::dump_vector(stream, "gp_Dir", &self.coordinates)
    }
    /// Parses without normalization, matching native InitFromJson; `direction.init_from_json(text,&mut position)?`.
    pub fn init_from_json(&mut self, text: &str, position: &mut i32) -> Result<bool, Failure> {
        dump::init_vector(text, "gp_Dir", position, &mut self.coordinates)
    }
}
