//! Rust CAD kernel: a behavioral port of the pinned OCCT source tree.
//!
//! The first TKernel APIs are available through [`foundation`]. Full toolkit
//! acceptance requires all upstream tests and API mappings to pass.
//!
//! [`modeling`] is the feature-evaluator seam (ADR 0068): solved sketch profiles to
//! certified planar regions, and a non-destructive extrusion returning an owned
//! [`modeling::OperationResult`] with complete history.

pub use rustycad_tkernel as foundation;

/// Planar regions from solved sketches and extrusion into solids with complete history.
///
/// ```
/// use rustycad_kernel::modeling::*;
/// let corners = [[0., 0.], [3., 0.], [3., 2.], [0., 2.]];
/// let vertices = corners.iter().enumerate()
///     .map(|(i, p)| ProfileVertex { id: ProfileVertexId(i as u64), point: *p }).collect();
/// let curves = (0..4u64).map(|i| ProfileCurve::Line { id: ProfileCurveId(i),
///     start: ProfileVertexId(i), end: ProfileVertexId((i + 1) % 4) }).collect();
/// let profile = SketchProfile { plane: ProfilePlane::world_xy(), vertices, curves };
/// let display = DisplayTolerance { chordal: 1e-3, angular: 0.1 };
/// let resolution = Resolution::new(1e-6, 1e-7, 1e-9, display, ToleranceCap::Refuse(1e-6));
/// let regions = build_planar_regions(&profile, resolution)?;
/// let cube = extrude(&regions.regions()[0], [0., 0., 1.], &ExtrudeOptions { resolution })?;
/// assert!(cube.volume().encloses(6.0));
/// assert_eq!(cube.faces().len(), 6);
/// let sides = cube.history().of(HistorySource::Curve(ProfileCurveId(0)));
/// assert!(sides.iter().any(|record| record.role == HistoryRole::SideFace));
/// # Ok::<(), OperationDecline>(())
/// ```
pub mod modeling {
    pub use rustycad_tkbo::profile_extrusion::*;
}
