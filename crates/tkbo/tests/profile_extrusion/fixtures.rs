// SPDX-License-Identifier: GPL-2.0-only
//! Named solved-sketch fixtures for the public profile extrusion seam.
use rustycad_tkbo::profile_extrusion::*;

/// A strict resolution: refuse any stored tolerance above 1e-6.
pub fn resolution() -> Resolution {
    let display = DisplayTolerance {
        chordal: 1e-3,
        angular: 0.1,
    };
    Resolution::new(1e-6, 1e-7, 1e-9, display, ToleranceCap::Refuse(1e-6))
}

/// Incremental builder of a solved sketch with caller-chosen ids.
pub struct SketchFixture {
    pub profile: SketchProfile,
}

impl SketchFixture {
    pub fn new() -> Self {
        Self {
            profile: SketchProfile {
                plane: ProfilePlane::world_xy(),
                vertices: Vec::new(),
                curves: Vec::new(),
            },
        }
    }

    pub fn vertex(&mut self, id: u64, point: [f64; 2]) -> &mut Self {
        let id = ProfileVertexId(id);
        self.profile.vertices.push(ProfileVertex { id, point });
        self
    }

    pub fn line(&mut self, id: u64, start: u64, end: u64) -> &mut Self {
        self.profile.curves.push(ProfileCurve::Line {
            id: ProfileCurveId(id),
            start: ProfileVertexId(start),
            end: ProfileVertexId(end),
        });
        self
    }

    pub fn arc(&mut self, id: u64, ends: [u64; 2], center: [f64; 2], sense: ArcSense) -> &mut Self {
        self.profile.curves.push(ProfileCurve::Arc {
            id: ProfileCurveId(id),
            start: ProfileVertexId(ends[0]),
            end: ProfileVertexId(ends[1]),
            center,
            sense,
        });
        self
    }

    pub fn circle(&mut self, id: u64, center: [f64; 2], radius: f64) -> &mut Self {
        self.profile.curves.push(ProfileCurve::Circle {
            id: ProfileCurveId(id),
            center,
            radius,
        });
        self
    }

    pub fn ellipse(
        &mut self,
        id: u64,
        center: [f64; 2],
        major_axis: [f64; 2],
        minor: f64,
    ) -> &mut Self {
        self.profile.curves.push(ProfileCurve::Ellipse {
            id: ProfileCurveId(id),
            center,
            major_axis,
            minor_radius: minor,
        });
        self
    }

    /// Elliptic arc of the ellipse `(center, major_axis, minor)` between vertices `ends`.
    pub fn elliptic_arc(
        &mut self,
        id: u64,
        ends: [u64; 2],
        ellipse: ([f64; 2], [f64; 2], f64),
        sense: ArcSense,
    ) -> &mut Self {
        let (center, major_axis, minor_radius) = ellipse;
        self.profile.curves.push(ProfileCurve::EllipticArc {
            id: ProfileCurveId(id),
            start: ProfileVertexId(ends[0]),
            end: ProfileVertexId(ends[1]),
            center,
            major_axis,
            minor_radius,
            sense,
        });
        self
    }

    /// Clamped spline between vertices `ends`.
    pub fn spline(
        &mut self,
        id: u64,
        ends: [u64; 2],
        degree: u32,
        poles: &[[f64; 2]],
        weights: Option<&[f64]>,
        knots: &[f64],
    ) -> &mut Self {
        self.profile.curves.push(ProfileCurve::Spline {
            id: ProfileCurveId(id),
            start: ProfileVertexId(ends[0]),
            end: ProfileVertexId(ends[1]),
            degree,
            poles: poles.to_vec(),
            weights: weights.map(<[f64]>::to_vec),
            knots: knots.to_vec(),
        });
        self
    }

    /// Closed axis-aligned rectangle: vertices `base..base+4`, lines `base+10..base+14`.
    pub fn rectangle(&mut self, base: u64, min: [f64; 2], max: [f64; 2]) -> &mut Self {
        let corners = [min, [max[0], min[1]], max, [min[0], max[1]]];
        for (offset, corner) in corners.iter().enumerate() {
            self.vertex(base + offset as u64, *corner);
        }
        for offset in 0..4 {
            self.line(base + 10 + offset, base + offset, base + (offset + 1) % 4);
        }
        self
    }

    pub fn build(&self) -> SketchProfile {
        self.profile.clone()
    }
}

/// A 2 x 3 rectangle on the world XY plane.
pub fn rectangle() -> SketchProfile {
    SketchFixture::new()
        .rectangle(0, [0., 0.], [2., 3.])
        .build()
}

/// One circle of radius 1.5 centred at (1, -2).
pub fn circle() -> SketchProfile {
    SketchFixture::new().circle(7, [1., -2.], 1.5).build()
}

/// Concentric circles of radius 2 (curve 1) and 1 (curve 2).
pub fn annulus() -> SketchProfile {
    SketchFixture::new()
        .circle(1, [0., 0.], 2.)
        .circle(2, [0., 0.], 1.)
        .build()
}

/// A 4 x 2 slot: two lines and two tangent half-circle arcs of radius 1; area 8 + pi.
pub fn slot() -> SketchProfile {
    SketchFixture::new()
        .vertex(0, [0., 0.])
        .vertex(1, [4., 0.])
        .vertex(2, [4., 2.])
        .vertex(3, [0., 2.])
        .line(10, 0, 1)
        .arc(11, [1, 2], [4., 1.], ArcSense::CounterClockwise)
        .line(12, 2, 3)
        .arc(13, [3, 0], [0., 1.], ArcSense::CounterClockwise)
        .build()
}

/// Two disjoint rectangles: 1 x 1 at the origin and 2 x 1 at x = 3.
pub fn two_rectangles() -> SketchProfile {
    SketchFixture::new()
        .rectangle(0, [0., 0.], [1., 1.])
        .rectangle(100, [3., 0.], [5., 1.])
        .build()
}

/// An ellipse centred at (1, -2) with major semi-axis (1.2, 1.6) (radius 2) and minor
/// radius 0.5: area pi.
pub fn ellipse() -> SketchProfile {
    SketchFixture::new()
        .ellipse(3, [1., -2.], [1.2, 1.6], 0.5)
        .build()
}

/// The parabola y = x (4 - x) / 2 from (0, 0) to (4, 0) as a two-span quadratic B-spline,
/// closed by the base line: area 16 / 3.
pub fn parabola_segment() -> SketchProfile {
    let poles = [[0., 0.], [1., 2.], [3., 2.], [4., 0.]];
    SketchFixture::new()
        .vertex(0, [0., 0.])
        .vertex(1, [4., 0.])
        .spline(1, [0, 1], 2, &poles, None, &[0., 0., 0., 0.5, 1., 1., 1.])
        .line(2, 1, 0)
        .build()
}

/// The upper unit half circle as a rational quadratic B-spline of two spans, closed by the
/// diameter: area pi / 2.
pub fn rational_half_disc() -> SketchProfile {
    let h = std::f64::consts::FRAC_1_SQRT_2;
    let poles = [[1., 0.], [1., 1.], [0., 1.], [-1., 1.], [-1., 0.]];
    let weights = [1., h, 1., h, 1.];
    SketchFixture::new()
        .vertex(0, [1., 0.])
        .vertex(1, [-1., 0.])
        .spline(
            1,
            [0, 1],
            2,
            &poles,
            Some(&weights),
            &[0., 0., 0., 0.5, 0.5, 1., 1., 1.],
        )
        .line(2, 1, 0)
        .build()
}
