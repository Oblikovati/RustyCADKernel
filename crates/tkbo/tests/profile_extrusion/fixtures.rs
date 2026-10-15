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
