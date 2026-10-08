//! `Select3D_Pnt` and `Select3D_PointData`: the single-precision points of the sensitive
//! polygons (a point is rounded to `float` when stored, as natively).
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::exceptions::{construction_error, out_of_range};
use rustycad_tkernel::standard::real::real_to_short_real;
use rustycad_tkmath::gp::{Point3, Xyz};

/// `Select3D_Pnt`: a point in single precision.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Select3DPnt {
    /// `x`.
    pub x: f32,
    /// `y`.
    pub y: f32,
    /// `z`.
    pub z: f32,
}

impl Select3DPnt {
    /// `operator=(gp_Pnt)`: the coordinates rounded with `RealToShortReal`.
    /// Example: `Select3DPnt::from_point(&Point3::new(1.0, 2.0, 3.0))`.
    pub fn from_point(point: &Point3) -> Self {
        Self {
            x: real_to_short_real(point.x()),
            y: real_to_short_real(point.y()),
            z: real_to_short_real(point.z()),
        }
    }
    /// `operator gp_Pnt`; `pnt.to_point()`.
    pub fn to_point(self) -> Point3 {
        Point3::new(f64::from(self.x), f64::from(self.y), f64::from(self.z))
    }
    /// `operator gp_XYZ`; `pnt.to_xyz()`.
    pub fn to_xyz(self) -> Xyz {
        Xyz::new(f64::from(self.x), f64::from(self.y), f64::from(self.z))
    }
}

/// `Select3D_PointData`: a fixed number of single-precision points.
///
/// Example: `let mut data = Select3DPointData::new(3)?; data.set_pnt(0, &p)?;`.
#[derive(Clone, Debug)]
pub struct Select3DPointData {
    points: Vec<Select3DPnt>,
}

impl Select3DPointData {
    /// `nb_points` zero points; a count <= 0 raises `Standard_ConstructionError`.
    /// Example: `Select3DPointData::new(4)?`.
    pub fn new(nb_points: i32) -> Result<Self, Failure> {
        if nb_points <= 0 {
            return Err(construction_error(&format!(
                "Select3D_PointData: {nb_points} points; expected > 0"
            )));
        }
        Ok(Self {
            points: vec![Select3DPnt::default(); nb_points as usize],
        })
    }
    /// The checked offset of an index.
    fn offset(&self, index: i32, member: &str) -> Result<usize, Failure> {
        usize::try_from(index)
            .ok()
            .filter(|&i| i < self.points.len())
            .ok_or_else(|| {
                out_of_range(&format!(
                    "Select3D_PointData::{member}: index {index}; expected 0..{}",
                    self.points.len()
                ))
            })
    }
    /// `SetPnt(index, Select3D_Pnt)`; an index out of range raises `Standard_OutOfRange`.
    /// Example: `data.set_select3d_pnt(0, pnt)?`.
    pub fn set_select3d_pnt(&mut self, index: i32, value: Select3DPnt) -> Result<(), Failure> {
        let offset = self.offset(index, "SetPnt")?;
        self.points[offset] = value;
        Ok(())
    }
    /// `SetPnt(index, gp_Pnt)`: the point rounded to single precision; `data.set_pnt(0, &p)?`.
    pub fn set_pnt(&mut self, index: i32, value: &Point3) -> Result<(), Failure> {
        self.set_select3d_pnt(index, Select3DPnt::from_point(value))
    }
    /// `Pnt`; an index out of range raises `Standard_OutOfRange`. Example: `data.pnt(0)?`.
    pub fn pnt(&self, index: i32) -> Result<Select3DPnt, Failure> {
        Ok(self.points[self.offset(index, "Pnt")?])
    }
    /// `Pnt3d`: the point in double precision; `data.pnt3d(0)?`.
    pub fn pnt3d(&self, index: i32) -> Result<Point3, Failure> {
        self.pnt(index).map(Select3DPnt::to_point)
    }
    /// `Size`; `data.size()`.
    pub fn size(&self) -> i32 {
        self.points.len() as i32
    }
    /// The points in order (for the dumps and the polygon tests).
    pub fn points(&self) -> &[Select3DPnt] {
        &self.points
    }
}
