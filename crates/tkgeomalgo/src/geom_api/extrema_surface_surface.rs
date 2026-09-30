// Rust translation of GeomAPI_ExtremaSurfaceSurface.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomAPI_ExtremaSurfaceSurface`: the extrema between two surfaces.
use super::{check_done, check_index};
use crate::geom2d_api::extrema_results::ExtremaResults;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomSurface, SurfaceBounds};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeombase::extrema::{ExtSS, POnSurf};
use rustycad_tkmath::gp::Point3;

/// `GeomAPI_ExtremaSurfaceSurface`: the extrema of Extrema_ExtSS between two
/// surfaces, with the index of the nearest one. It fails (0 extrema) when Extrema
/// fails or finds none.
///
/// Natively each Init copies a new Extrema_ExtSS built on local adaptors into its
/// member, so only its results are read afterwards. The object keeps these results;
/// it keeps the adaptors and bounds too, and [`ExtremaSurfaceSurface::extrema`]
/// rebuilds the same Extrema_ExtSS on them.
///
/// ```
/// use rustycad_tkgeomalgo::geom_api::ExtremaSurfaceSurface;
/// use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Axis3, Direction3, Point3};
/// let plane = |z: f64| -> Handle<dyn GeomSurface> {
///     let frame = Axis3::from_normal(Point3::new(0.0, 0.0, z), Direction3::new(0.0, 0.0, 1.0).unwrap()).unwrap();
///     Handle::new(GeomPlane::new(frame)).cast()
/// };
/// // Parallel planes: Extrema_ExtSS answers parallel, with their distance.
/// let e = ExtremaSurfaceSurface::with_surfaces(&plane(0.0), &plane(4.0)).unwrap();
/// assert!((e.lower_distance().unwrap() - 4.0).abs() < 1e-9);
/// ```
#[derive(Default)]
pub struct ExtremaSurfaceSurface {
    my_is_done: bool,
    my_index: i32,
    my_ext_ss: Option<ExtremaResults<(POnSurf, POnSurf)>>,
    is_parallel: Option<Result<bool, Failure>>,
    the_surface1: GeomAdaptorSurface,
    the_surface2: GeomAdaptorSurface,
    bounds: Option<Option<(SurfaceBounds, SurfaceBounds)>>,
}

impl ExtremaSurfaceSurface {
    /// `GeomAPI_ExtremaSurfaceSurface()`: no extrema. `ExtremaSurfaceSurface::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GeomAPI_ExtremaSurfaceSurface(S1, S2)`: on the whole surfaces.
    /// `ExtremaSurfaceSurface::with_surfaces(&s1, &s2)?`.
    pub fn with_surfaces(
        s1: &Handle<dyn GeomSurface>,
        s2: &Handle<dyn GeomSurface>,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.init(s1, s2)?;
        Ok(extrema)
    }
    /// `GeomAPI_ExtremaSurfaceSurface(S1, S2, U1min, U1max, V1min, V1max, U2min,
    /// U2max, V2min, V2max)`. `ExtremaSurfaceSurface::with_bounds(&s1, &s2, (0., 1.,
    /// 0., 1.), (0., 1., 0., 1.))?`.
    pub fn with_bounds(
        s1: &Handle<dyn GeomSurface>,
        s2: &Handle<dyn GeomSurface>,
        bounds1: SurfaceBounds,
        bounds2: SurfaceBounds,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.init_bounds(s1, s2, bounds1, bounds2)?;
        Ok(extrema)
    }
    /// `Init(S1, S2)`: Extrema_ExtSS on the whole surfaces with the tolerance
    /// Precision::PConfusion(). `extrema.init(&s1, &s2)?`.
    pub fn init(
        &mut self,
        s1: &Handle<dyn GeomSurface>,
        s2: &Handle<dyn GeomSurface>,
    ) -> Result<(), Failure> {
        let the_surface1 = GeomAdaptorSurface::with_surface(s1)?;
        let the_surface2 = GeomAdaptorSurface::with_surface(s2)?;
        self.store(the_surface1, the_surface2, None)
    }
    /// `Init(S1, S2, U1min, U1max, V1min, V1max, U2min, U2max, V2min, V2max)`: the
    /// adaptors and Extrema_ExtSS on the bounds. `extrema.init_bounds(&s1, &s2, (0.,
    /// 1., 0., 1.), (0., 1., 0., 1.))?`.
    pub fn init_bounds(
        &mut self,
        s1: &Handle<dyn GeomSurface>,
        s2: &Handle<dyn GeomSurface>,
        bounds1: SurfaceBounds,
        bounds2: SurfaceBounds,
    ) -> Result<(), Failure> {
        let the_surface1 = GeomAdaptorSurface::with_bounds(s1, bounds1)?;
        let the_surface2 = GeomAdaptorSurface::with_bounds(s2, bounds2)?;
        self.store(the_surface1, the_surface2, Some((bounds1, bounds2)))
    }

    // Extrema_ExtSS on the new adaptors, then the nearest extremum.
    fn store(
        &mut self,
        the_surface1: GeomAdaptorSurface,
        the_surface2: GeomAdaptorSurface,
        bounds: Option<(SurfaceBounds, SurfaceBounds)>,
    ) -> Result<(), Failure> {
        let (results, is_parallel) = {
            let ext_ss: ExtSS<'_> = perform(&the_surface1, &the_surface2, bounds)?;
            (snapshot(&ext_ss), ext_ss.is_parallel())
        };
        (self.the_surface1, self.the_surface2) = (the_surface1, the_surface2);
        self.bounds = Some(bounds);
        self.my_is_done = results.done && results.nb_ext()? > 0;
        self.my_ext_ss = Some(results);
        self.is_parallel = Some(is_parallel);
        if self.my_is_done {
            self.my_index = self.results()?.nearest()?;
        }
        Ok(())
    }
}

impl ExtremaSurfaceSurface {
    /// `NbExtrema()`: 0 when the computation failed. `extrema.nb_extrema()`.
    pub fn nb_extrema(&self) -> i32 {
        match (&self.my_ext_ss, self.my_is_done) {
            (Some(ext), true) => ext.nb,
            _ => 0,
        }
    }
    /// `operator int()`: NbExtrema(). `extrema.as_int()`.
    pub fn as_int(&self) -> i32 {
        self.nb_extrema()
    }
    /// `Points(Index, P1, P2)`: Standard_OutOfRange outside `1..=NbExtrema()`.
    /// `let (p1, p2) = extrema.points(1)?;`
    pub fn points(&self, index: i32) -> Result<(Point3, Point3), Failure> {
        let (ps1, ps2) = self.pair(index, "GeomAPI_ExtremaSurfaceSurface::Points")?;
        Ok((*ps1.value(), *ps2.value()))
    }
    /// `Parameters(Index, U1, V1, U2, V2)`: Standard_OutOfRange outside
    /// `1..=NbExtrema()`. `let (u1, v1, u2, v2) = extrema.parameters(1)?;`
    pub fn parameters(&self, index: i32) -> Result<(f64, f64, f64, f64), Failure> {
        let (ps1, ps2) = self.pair(index, "GeomAPI_ExtremaSurfaceSurface::Parameters")?;
        let ((u1, v1), (u2, v2)) = (ps1.parameter(), ps2.parameter());
        Ok((u1, v1, u2, v2))
    }
    /// `Distance(Index)`: Standard_OutOfRange outside `1..=NbExtrema()`.
    /// `extrema.distance(1)?`.
    pub fn distance(&self, index: i32) -> Result<f64, Failure> {
        check_index(
            index,
            self.nb_extrema(),
            "GeomAPI_ExtremaSurfaceSurface::Distance",
        )?;
        Ok(self.results()?.square_distance(index)?.sqrt())
    }
    /// `IsParallel()`: Extrema_ExtSS::IsParallel (StdFail_NotDone when Extrema failed
    /// or before any Init). `extrema.is_parallel()?`.
    pub fn is_parallel(&self) -> Result<bool, Failure> {
        match &self.is_parallel {
            Some(result) => result.clone(),
            None => Err(super::not_done("Extrema_ExtSS::IsParallel()")),
        }
    }
    /// `NearestPoints(P1, P2)`: StdFail_NotDone when the computation failed.
    /// `let (p1, p2) = extrema.nearest_points()?;`
    pub fn nearest_points(&self) -> Result<(Point3, Point3), Failure> {
        check_done(
            self.my_is_done,
            "GeomAPI_ExtremaSurfaceSurface::NearestPoints",
        )?;
        self.points(self.my_index)
    }
    /// `LowerDistanceParameters(U1, V1, U2, V2)`: StdFail_NotDone when the computation
    /// failed. `let (u1, v1, u2, v2) = extrema.lower_distance_parameters()?;`
    pub fn lower_distance_parameters(&self) -> Result<(f64, f64, f64, f64), Failure> {
        check_done(
            self.my_is_done,
            "GeomAPI_ExtremaSurfaceSurface::LowerDistanceParameters",
        )?;
        self.parameters(self.my_index)
    }
    /// `LowerDistance()`: StdFail_NotDone when the computation failed.
    /// `extrema.lower_distance()?`.
    pub fn lower_distance(&self) -> Result<f64, Failure> {
        check_done(
            self.my_is_done,
            "GeomAPI_ExtremaSurfaceSurface::LowerDistance",
        )?;
        Ok(self.results()?.square_distance(self.my_index)?.sqrt())
    }
    /// `operator double()`: LowerDistance(). `extrema.as_real()?`.
    pub fn as_real(&self) -> Result<f64, Failure> {
        self.lower_distance()
    }
    /// `Extrema()`: the Extrema_ExtSS of the last Init, rebuilt on its adaptors (a
    /// default one before any Init). `extrema.extrema()?.is_done()`.
    pub fn extrema(&self) -> Result<ExtSS<'_>, Failure> {
        match self.bounds {
            None => Ok(ExtSS::new()),
            Some(bounds) => perform(&self.the_surface1, &self.the_surface2, bounds),
        }
    }

    fn results(&self) -> Result<&ExtremaResults<(POnSurf, POnSurf)>, Failure> {
        self.my_ext_ss
            .as_ref()
            .ok_or_else(|| super::not_done("Extrema_ExtSS::Points"))
    }
    fn pair(&self, index: i32, member: &str) -> Result<(POnSurf, POnSurf), Failure> {
        check_index(index, self.nb_extrema(), member)?;
        self.results()?.item(index)
    }
}

// `Extrema_ExtSS theExtSS(TheSurface1, TheSurface2, [bounds,] Tol, Tol)`.
fn perform<'s>(
    the_surface1: &GeomAdaptorSurface,
    the_surface2: &'s GeomAdaptorSurface,
    bounds: Option<(SurfaceBounds, SurfaceBounds)>,
) -> Result<ExtSS<'s>, Failure> {
    let tol: f64 = precision::p_confusion();
    let Some((b1, b2)) = bounds else {
        return ExtSS::with_surfaces(the_surface1, the_surface2, tol, tol);
    };
    let bounds1: [f64; 4] = [b1.0, b1.1, b1.2, b1.3];
    let bounds2: [f64; 4] = [b2.0, b2.1, b2.2, b2.3];
    ExtSS::with_ranges(the_surface1, the_surface2, bounds1, bounds2, (tol, tol))
}

// The extrema of an Extrema_ExtSS, with the exceptions of its accessors.
fn snapshot(ext: &ExtSS<'_>) -> ExtremaResults<(POnSurf, POnSurf)> {
    let Ok(nb) = ext.nb_ext() else {
        return ExtremaResults::not_done("Extrema_ExtSS");
    };
    let mut results = ExtremaResults::done("Extrema_ExtSS", nb);
    for n in 1..=nb {
        results.push(ext.points(n), ext.square_distance(n));
    }
    results
}
