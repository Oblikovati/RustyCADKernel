//! CSLib_Class2d, low-level 2D point-in-polygon classification.
mod grid;
mod normalize;
mod ray;

use crate::gp::Point2;
use grid::GridCache;
use normalize::{
    is_representable_value, normalize_tolerance, sanitize_tolerance, transform_to_normalized,
};
use ray::NormalizedPolygon;
use rustycad_tkernel::ncollection::{Array1, DynamicArray, Sequence};

/// Classification of a point against the polygon (native `CSLib_Class2d::Result`).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum Class2dResult {
    /// Strictly inside the polygon.
    Inside = 1,
    /// Strictly outside the polygon.
    Outside = -1,
    /// On the boundary, near it within tolerance, or undecidable.
    Uncertain = 0,
}

/// Ray-casting classifier over a polygon normalized to `[0,1] x [0,1]`.
///
/// After 64 queries on a polygon of at least 24 vertices a 32x32 cell cache is
/// built once, thread-safely, and answers queries far from the boundary.
/// `Class2d::from_array(&points, 1e-10, 1e-10, 0.0, 0.0, 1.0, 1.0).si_dans(Point2::new(0.5, 0.5))`.
#[derive(Debug, Default)]
pub struct Class2d {
    polygon: NormalizedPolygon,
    tol_u: f64,
    tol_v: f64,
    original_tol_u: f64,
    original_tol_v: f64,
    u_min: f64,
    v_min: f64,
    u_max: f64,
    v_max: f64,
    grid: GridCache,
}

/// The native constructor arguments after the point container.
#[derive(Clone, Copy)]
struct Setup {
    tol_u: f64,
    tol_v: f64,
    u_min: f64,
    v_min: f64,
    u_max: f64,
    v_max: f64,
}

impl Class2d {
    /// Classifier over an NCollection_Array1 of vertices (closed implicitly).
    ///
    /// Invalid bounds (non-representable, `max <= min`) or fewer than three
    /// vertices produce a classifier that answers [`Class2dResult::Uncertain`].
    /// `Class2d::from_array(&points, 1e-10, 1e-10, 0.0, 0.0, 1.0, 1.0)`.
    pub fn from_array(
        points: &Array1<'_, Point2>,
        tol_u: f64,
        tol_v: f64,
        u_min: f64,
        v_min: f64,
        u_max: f64,
        v_max: f64,
    ) -> Self {
        let setup: Setup = Setup {
            tol_u,
            tol_v,
            u_min,
            v_min,
            u_max,
            v_max,
        };
        Self::initialized(points.iter(), setup)
    }

    /// Classifier over an NCollection_Sequence of vertices; see [`Self::from_array`].
    /// `Class2d::from_sequence(&points, 1e-10, 1e-10, 0.0, 0.0, 1.0, 1.0)`.
    pub fn from_sequence(
        points: &Sequence<Point2>,
        tol_u: f64,
        tol_v: f64,
        u_min: f64,
        v_min: f64,
        u_max: f64,
        v_max: f64,
    ) -> Self {
        let setup: Setup = Setup {
            tol_u,
            tol_v,
            u_min,
            v_min,
            u_max,
            v_max,
        };
        Self::initialized(points.iter(), setup)
    }

    /// Classifier over an NCollection_DynamicArray of vertices; see [`Self::from_array`].
    /// `Class2d::from_dynamic_array(&points, 1e-10, 1e-10, 0.0, 0.0, 1.0, 1.0)`.
    pub fn from_dynamic_array(
        points: &DynamicArray<Point2>,
        tol_u: f64,
        tol_v: f64,
        u_min: f64,
        v_min: f64,
        u_max: f64,
        v_max: f64,
    ) -> Self {
        let setup: Setup = Setup {
            tol_u,
            tol_v,
            u_min,
            v_min,
            u_max,
            v_max,
        };
        Self::initialized(points.iter(), setup)
    }

    // CSLib_Class2d::init: bounds are stored even when the input is rejected.
    fn initialized<'p>(points: impl ExactSizeIterator<Item = &'p Point2>, setup: Setup) -> Self {
        let mut classifier: Self = Self {
            u_min: setup.u_min,
            v_min: setup.v_min,
            u_max: setup.u_max,
            v_max: setup.v_max,
            ..Self::default()
        };
        if !has_valid_domain(&setup) || points.len() < 3 {
            return classifier;
        }
        classifier.original_tol_u = sanitize_tolerance(setup.tol_u);
        classifier.original_tol_v = sanitize_tolerance(setup.tol_v);
        classifier.tol_u = normalize_tolerance(setup.tol_u, setup.u_min, setup.u_max);
        classifier.tol_v = normalize_tolerance(setup.tol_v, setup.v_min, setup.v_max);
        classifier.polygon = normalized_polygon(points, &setup);
        classifier
    }

    /// Move-constructor semantics: returns the classifier and leaves `source`
    /// empty and grid-disabled, so it answers [`Class2dResult::Uncertain`].
    /// `let moved = Class2d::moved_from(&mut classifier);`.
    pub fn moved_from(source: &mut Self) -> Self {
        let grid: GridCache = source.grid.taken();
        Self {
            polygon: std::mem::take(&mut source.polygon),
            grid,
            ..source.scalars()
        }
    }

    /// Move-assignment semantics; see [`Self::moved_from`]. `target.move_from(&mut source)`.
    pub fn move_from(&mut self, source: &mut Self) {
        *self = Self::moved_from(source);
    }

    /// Move-assigns `source` and returns the mutated destination.
    ///
    /// The returned reference mirrors the native assignment expression for
    /// distinct objects. Use [`Self::assign_self`] for the native self-move case.
    /// `let destination = target.move_from_returning(&mut source);`.
    pub fn move_from_returning(&mut self, source: &mut Self) -> &mut Self {
        self.move_from(source);
        self
    }

    /// Deep-copy assigns `source` and returns the mutated destination.
    ///
    /// A completed grid cache is copied with the polygon. The native self-copy
    /// case is represented separately by [`Self::assign_self`].
    /// `let destination = target.assign_returning(&source);`.
    pub fn assign_returning(&mut self, source: &Self) -> &mut Self {
        Clone::clone_from(self, source);
        self
    }

    /// Models both native self-copy and self-move assignment, retaining every field.
    /// `classifier.assign_self().si_dans(point)` keeps the existing polygon and grid.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }

    // Every non-owning field, with an empty polygon and a fresh grid.
    fn scalars(&self) -> Self {
        Self {
            polygon: NormalizedPolygon::default(),
            tol_u: self.tol_u,
            tol_v: self.tol_v,
            original_tol_u: self.original_tol_u,
            original_tol_v: self.original_tol_v,
            u_min: self.u_min,
            v_min: self.v_min,
            u_max: self.u_max,
            v_max: self.v_max,
            grid: GridCache::default(),
        }
    }

    /// Classifies a point with the construction tolerances (native `SiDans`).
    /// `classifier.si_dans(Point2::new(0.5, 0.5)) == Class2dResult::Inside`.
    pub fn si_dans(&self, point: Point2) -> Class2dResult {
        if self.polygon.count() == 0 {
            return Class2dResult::Uncertain;
        }
        let (tol_u, tol_v) = (self.original_tol_u, self.original_tol_v);
        if self.is_outside_box(point, tol_u, tol_v) {
            return Class2dResult::Outside;
        }
        let x: f64 = transform_to_normalized(point.x(), self.u_min, self.u_max);
        let y: f64 = transform_to_normalized(point.y(), self.v_min, self.v_max);
        let tol: [f64; 2] = [self.tol_u, self.tol_v];
        if let Some(cached) = self.grid.lookup(&self.polygon, tol, x, y) {
            return cached;
        }
        let result: Class2dResult = self.polygon.classify_on(x, y, self.tol_u, self.tol_v);
        if result == Class2dResult::Uncertain {
            return Class2dResult::Uncertain; // ON boundary
        }
        self.confirmed_by_corners(result, x, y, tol)
    }

    /// Classifies a point with an explicit original-space ON tolerance that
    /// replaces the construction tolerances (native `SiDans_OnMode`).
    /// `classifier.si_dans_on_mode(Point2::new(0.5, 0.5), 0.01)`.
    pub fn si_dans_on_mode(&self, point: Point2, tol: f64) -> Class2dResult {
        if self.polygon.count() == 0 {
            return Class2dResult::Uncertain;
        }
        let tolerance: f64 = sanitize_tolerance(tol);
        let tol_u: f64 = normalize_tolerance(tolerance, self.u_min, self.u_max);
        let tol_v: f64 = normalize_tolerance(tolerance, self.v_min, self.v_max);
        if self.is_outside_box(point, tolerance, tolerance) {
            return Class2dResult::Outside;
        }
        let x: f64 = transform_to_normalized(point.x(), self.u_min, self.u_max);
        let y: f64 = transform_to_normalized(point.y(), self.v_min, self.v_max);
        // Unlike si_dans, an ON result still goes through the corner test (native).
        let result: Class2dResult = self.polygon.classify_on(x, y, tol_u, tol_v);
        self.confirmed_by_corners(result, x, y, [tol_u, tol_v])
    }

    fn is_outside_box(&self, point: Point2, tol_u: f64, tol_v: f64) -> bool {
        point.x() < (self.u_min - tol_u)
            || point.x() > (self.u_max + tol_u)
            || point.y() < (self.v_min - tol_v)
            || point.y() > (self.v_max + tol_v)
    }

    // A tolerance box whose corners do not all agree with `result` is near the boundary.
    fn confirmed_by_corners(
        &self,
        result: Class2dResult,
        x: f64,
        y: f64,
        tol: [f64; 2],
    ) -> Class2dResult {
        let [tol_u, tol_v] = tol;
        let has_tolerance: bool = tol_u > 0.0 || tol_v > 0.0;
        if !has_tolerance {
            return result;
        }
        let inside: bool = result == Class2dResult::Inside;
        let corners: [(f64, f64); 4] = [
            (x - tol_u, y - tol_v),
            (x + tol_u, y - tol_v),
            (x - tol_u, y + tol_v),
            (x + tol_u, y + tol_v),
        ];
        if corners
            .iter()
            .any(|&(cx, cy)| inside != self.polygon.contains(cx, cy))
        {
            return Class2dResult::Uncertain;
        }
        result
    }
}

impl Clone for Class2d {
    /// Deep copy; a completed grid is copied, a grid under construction is not.
    fn clone(&self) -> Self {
        Self {
            polygon: self.polygon.clone(),
            grid: self.grid.copied(),
            ..self.scalars()
        }
    }
}

fn has_valid_domain(setup: &Setup) -> bool {
    let bounds: [f64; 4] = [setup.u_min, setup.v_min, setup.u_max, setup.v_max];
    bounds.iter().all(|&bound| is_representable_value(bound))
        && setup.u_max > setup.u_min
        && setup.v_max > setup.v_min
}

// Normalized coordinates with the first vertex repeated to close the polygon.
fn normalized_polygon<'p>(
    points: impl ExactSizeIterator<Item = &'p Point2>,
    setup: &Setup,
) -> NormalizedPolygon {
    let mut polygon: NormalizedPolygon = NormalizedPolygon {
        xs: Vec::with_capacity(points.len() + 1),
        ys: Vec::with_capacity(points.len() + 1),
    };
    for point in points {
        polygon
            .xs
            .push(transform_to_normalized(point.x(), setup.u_min, setup.u_max));
        polygon
            .ys
            .push(transform_to_normalized(point.y(), setup.v_min, setup.v_max));
    }
    polygon.xs.push(polygon.xs[0]);
    polygon.ys.push(polygon.ys[0]);
    polygon
}
