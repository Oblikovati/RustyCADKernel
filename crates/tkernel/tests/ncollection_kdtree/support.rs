//! Legacy points and deterministic random generator, independent of production query algorithms.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::KdPoint;
#[derive(Clone, Copy, Debug, Default)]
pub struct TestPoint2D {
    pub X: f64,
    pub Y: f64,
}
impl TestPoint2D {
    pub fn new(x: impl Into<f64>, y: impl Into<f64>) -> Self {
        Self {
            X: x.into(),
            Y: y.into(),
        }
    }
}
impl KdPoint for TestPoint2D {
    fn coord(&self, index: usize) -> f64 {
        if index == 1 { self.X } else { self.Y }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct TestPoint3D {
    pub X: f64,
    pub Y: f64,
    pub Z: f64,
}
impl TestPoint3D {
    pub fn new(x: impl Into<f64>, y: impl Into<f64>, z: impl Into<f64>) -> Self {
        Self {
            X: x.into(),
            Y: y.into(),
            Z: z.into(),
        }
    }
}
impl KdPoint for TestPoint3D {
    fn coord(&self, index: usize) -> f64 {
        match index {
            1 => self.X,
            2 => self.Y,
            _ => self.Z,
        }
    }
}
pub fn sqDist2D(a: TestPoint2D, b: TestPoint2D) -> f64 {
    let dx = a.X - b.X;
    let dy = a.Y - b.Y;
    dx * dx + dy * dy
}
pub fn sqDist3D(a: TestPoint3D, b: TestPoint3D) -> f64 {
    let dx = a.X - b.X;
    let dy = a.Y - b.Y;
    let dz = a.Z - b.Z;
    dx * dx + dy * dy + dz * dz
}
pub struct TestRandom {
    seed: u32,
}
impl TestRandom {
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }
    pub fn next_double(&mut self, min: impl Into<f64>, max: impl Into<f64>) -> f64 {
        self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
        let norm = f64::from((self.seed >> 16) & 0x7fff) / 32767.0;
        let min = min.into();
        min + norm * (max.into() - min)
    }
}
#[derive(Debug, Default)]
pub struct TestSet<T> {
    values: Vec<T>,
}
impl<T: PartialEq> TestSet<T> {
    pub fn insert(&mut self, value: T) {
        if !self.values.contains(&value) {
            self.values.push(value);
        }
    }
    pub fn count(&self, value: T) -> usize {
        usize::from(self.values.contains(&value))
    }
    pub fn len(&self) -> usize {
        self.values.len()
    }
}
impl<T: PartialEq> PartialEq for TestSet<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.values.iter().all(|v| other.values.contains(v))
    }
}

pub const EMPTY_POINTS: [TestPoint2D; 0] = [];
pub const EMPTY_RADII: [f64; 0] = [];
