// Rust translation of ModelingData/TKG2d/Geom2dAdaptor/Geom2dAdaptor_Curve.cxx and ModelingData/TKG3d/GeomAdaptor/GeomAdaptor_Curve.cxx (IsBoundary and RebuildCache, which both adaptors repeat) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The boundary-span test and the evaluation caches that Geom2dAdaptor_Curve and
//! GeomAdaptor_Curve build the same way for Bezier and B-spline curves.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib::{Cache, CurvePoint, flat_bezier_knots};

/// Half the parametric confusion: the LocateU tolerance of the boundary spans.
pub const BOUNDARY_LOCATE_TOLERANCE: f64 = precision::p_confusion() / 2.0;

/// `IsBoundary`: the knot span that `u` ends when it equals an adaptor bound, widened
/// to one span inside `1..=nb_knots`; `None` elsewhere. `locate_u(bound)` is the
/// curve's `LocateU(bound, BOUNDARY_LOCATE_TOLERANCE, false)`;
/// `boundary_span(0.0, (0.0, 1.0), 4, |u| curve.locate_u(u, tol, false))?`.
pub fn boundary_span(
    u: f64,
    (first, last): (f64, f64),
    nb_knots: impl FnOnce() -> i32,
    locate_u: impl FnOnce(f64) -> Result<(i32, i32), Failure>,
) -> Result<Option<(i32, i32)>, Failure> {
    if u == first {
        let (mut start, mut finish) = locate_u(first)?;
        start = start.max(1);
        if start >= finish {
            finish = start + 1;
        }
        return Ok(Some((start, finish)));
    }
    if u == last {
        let (mut start, mut finish) = locate_u(last)?;
        finish = finish.min(nb_knots());
        if start >= finish {
            start = finish - 1;
        }
        return Ok(Some((start, finish)));
    }
    Ok(None)
}

/// The poles, weights and flat knots a cache is built from.
pub struct CacheSource<'a, P: CurvePoint> {
    /// The flat knot sequence.
    pub flat: &'a Array1<'a, f64>,
    /// The poles.
    pub poles: &'a Array1<'a, P>,
    /// The weights of a rational curve.
    pub weights: Option<&'a Array1<'a, f64>>,
}

/// `RebuildCache` of a B-spline: create the cache once with `degree()` and
/// `periodic()`, then fill it on the span of `u`;
/// `rebuild_cache(&mut cache, &source, || 3, || false, 0.5)?`.
pub fn rebuild_cache<P: CurvePoint>(
    cache: &mut Option<Cache>,
    source: &CacheSource<'_, P>,
    degree: impl FnOnce() -> i32,
    periodic: impl FnOnce() -> bool,
    u: f64,
) -> Result<(), Failure> {
    if cache.is_none() {
        let periodic: bool = periodic();
        *cache = Some(Cache::new(
            degree(),
            periodic,
            source.flat,
            source.poles,
            source.weights,
        )?);
    }
    let cache: &mut Cache = cache.as_mut().expect("cache created above");
    cache.build_cache(u, source.flat, source.poles, source.weights)
}

/// The cache of a Bezier curve of `degree`, built on its single span, which is valid
/// for every parameter; `bezier_span_cache(3, false, &poles, None, 0.5)?`.
pub fn bezier_span_cache<P: CurvePoint>(
    degree: i32,
    periodic: bool,
    poles: &Array1<'_, P>,
    weights: Option<&Array1<'_, f64>>,
    u: f64,
) -> Result<Cache, Failure> {
    let knots: &'static [f64] = flat_bezier_knots(degree)?;
    let flat = Array1::from_shared(&knots[..2 * (degree as usize + 1)], 1);
    let cache = Cache::new(degree, periodic, &flat, poles, weights)?;
    cache.build_cache(u, &flat, poles, weights)?;
    Ok(cache)
}
