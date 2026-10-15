// SPDX-License-Identifier: GPL-2.0-only
//! Closed-form distances and outward normals of the fixture surfaces. They use no
//! kernel geometry, so they are an independent oracle for the presentation mesh.

/// An analytic surface centred on the origin / Z axis.
#[derive(Clone, Copy, Debug)]
pub enum Surface {
    /// The plane `coordinate[axis] = offset`; `sign` orients the outward normal.
    Plane { axis: usize, offset: f64, sign: f64 },
    /// A cylinder around Z; `sign = -1` when the material is outside it.
    Cylinder { radius: f64, sign: f64 },
    /// A sphere centred on the origin.
    Sphere { radius: f64 },
    /// A torus around Z centred on the origin.
    Torus { major: f64, minor: f64 },
}

impl Surface {
    /// An axis-aligned plane.
    pub fn axis_plane(axis: usize, offset: f64, sign: f64) -> Self {
        Self::Plane { axis, offset, sign }
    }

    /// A cylinder around Z.
    pub fn cylinder(radius: f64, sign: f64) -> Self {
        Self::Cylinder { radius, sign }
    }

    /// The unsigned distance from `p` to the surface.
    pub fn distance(&self, p: [f64; 3]) -> f64 {
        match *self {
            Self::Plane { axis, offset, .. } => (p[axis] - offset).abs(),
            Self::Cylinder { radius, .. } => (p[0].hypot(p[1]) - radius).abs(),
            Self::Sphere { radius } => (norm(p) - radius).abs(),
            Self::Torus { major, minor } => ((p[0].hypot(p[1]) - major).hypot(p[2]) - minor).abs(),
        }
    }

    /// The outward unit normal at (or near) `p`.
    pub fn outward_normal(&self, p: [f64; 3]) -> [f64; 3] {
        match *self {
            Self::Plane { axis, sign, .. } => {
                let mut normal = [0.0; 3];
                normal[axis] = sign;
                normal
            }
            Self::Cylinder { sign, .. } => unit([sign * p[0], sign * p[1], 0.0]),
            Self::Sphere { .. } => unit(p),
            Self::Torus { major, .. } => {
                let rho = p[0].hypot(p[1]);
                let scale = (rho - major) / rho;
                unit([p[0] * scale, p[1] * scale, p[2]])
            }
        }
    }
}

pub fn norm(v: [f64; 3]) -> f64 {
    dot(v, v).sqrt()
}

pub fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub fn unit(v: [f64; 3]) -> [f64; 3] {
    let length = norm(v);
    [v[0] / length, v[1] / length, v[2] / length]
}
