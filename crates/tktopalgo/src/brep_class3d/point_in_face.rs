// Rust translation of the PointInTheFace members of BRepClass3d_SolidExplorer.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `PointInTheFace` overloads of the solid explorer: a point of a face found on
//! UV grids, numbered so that a search can resume after the last point found.
use super::{BRepClass3dSolidExplorer, FacePointSample};
use crate::int_curves_face::IntCurvesFaceIntersector;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{self, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkmath::gp::{Point2, Point3};

/// The UV rectangle `(U1, V1, U2, V2)` of the search, in the native argument order.
pub type UvRectangle = [f64; 4];

/// One direction of a grid scan: start, signed step and the bound it must not reach.
#[derive(Clone, Copy)]
struct Sweep {
    start: f64,
    step: f64,
    bound: f64,
}

impl Sweep {
    /// `for (x = start; x < bound; x += step)` for a positive step, `x > bound` and
    /// `x -= |step|` for a negative one.
    fn values(self) -> impl Iterator<Item = f64> {
        std::iter::successors(Some(self.start), move |x| Some(x + self.step)).take_while(move |x| {
            if self.step > 0.0 {
                *x < self.bound
            } else {
                *x > self.bound
            }
        })
    }
}

/// The grid search state: the intersector of the face, its surface and the number of
/// points tried (`NbPntCalc`).
struct GridSearch<'s> {
    explorer: &'s BRepClass3dSolidExplorer,
    intersector: &'s IntCurvesFaceIntersector,
    surface: &'s BRepAdaptorSurface,
    count: i32,
}

impl GridSearch<'_> {
    /// Tries the points of one grid (`u` outer, `v` inner) from the `index`-th point
    /// counted since the start; a point IN the face is written into `sample`.
    fn scan(
        &mut self,
        us: Sweep,
        vs: Sweep,
        index: &mut i32,
        sample: &mut FacePointSample,
    ) -> Result<bool, Failure> {
        for u in us.values() {
            for v in vs.values() {
                if self.try_point(u, v, index, sample)? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// `if (++NbPntCalc >= IndexPoint)`: classifies (u, v); IN writes the point, its
    /// derivatives and its index.
    fn try_point(
        &mut self,
        u: f64,
        v: f64,
        index: &mut i32,
        sample: &mut FacePointSample,
    ) -> Result<bool, Failure> {
        self.count += 1;
        if self.count < *index {
            return Ok(false);
        }
        let state =
            self.explorer
                .classify_uv_point(self.intersector, self.surface, Point2::new(u, v))?;
        if state != State::In {
            return Ok(false);
        }
        let d1 = self.surface.d1(u, v)?;
        *sample = FacePointSample {
            point: d1.point,
            u,
            v,
            d1u: d1.d1u,
            d1v: d1.d1v,
        };
        *index = self.count;
        Ok(true)
    }
}

impl BRepClass3dSolidExplorer {
    /// `PointInTheFace(F, P, u, v, Param, Index)`: the search of
    /// [`BRepClass3dSolidExplorer::point_in_the_face_on_surface`] on the restricted
    /// surface of the FORWARD face, infinite bounds limited to ±1e10;
    /// `explorer.point_in_the_face(&face, &mut p, &mut u, &mut v, &mut param, &mut index)?`.
    pub fn point_in_the_face(
        &self,
        face_in: &Face,
        point: &mut Point3,
        u: &mut f64,
        v: &mut f64,
        param: &mut f64,
        index: &mut i32,
    ) -> Result<bool, Failure> {
        let face = topods::face(&face_in.oriented(Orientation::Forward))?;
        let surface = BRepAdaptorSurface::with_face(&face, true)?;
        let rectangle = limit_infinite_uv([
            surface.first_u_parameter()?,
            surface.first_v_parameter()?,
            surface.last_u_parameter()?,
            surface.last_v_parameter()?,
        ]);
        let mut sample = FacePointSample {
            point: *point,
            u: *u,
            v: *v,
            ..FacePointSample::default()
        };
        let found = self.point_in_the_face_on_surface(
            &face,
            &mut sample,
            param,
            index,
            &surface,
            rectangle,
        )?;
        (*point, *u, *v) = (sample.point, sample.u, sample.v);
        Ok(found)
    }

    /// `PointInTheFace(F, P, u, v, Param, Index, surf, U1, V1, U2, V2)`: as
    /// [`BRepClass3dSolidExplorer::point_in_the_face_on_surface_d1`], writing only the
    /// point and its parameters of `sample`;
    /// `explorer.point_in_the_face_on_surface(&face, &mut s, &mut param, &mut index, &surf, [0., 0., 1., 1.])?`.
    pub fn point_in_the_face_on_surface(
        &self,
        face: &Face,
        sample: &mut FacePointSample,
        param: &mut f64,
        index: &mut i32,
        surface: &BRepAdaptorSurface,
        rectangle: UvRectangle,
    ) -> Result<bool, Failure> {
        let mut full = *sample;
        let found = self
            .point_in_the_face_on_surface_d1(face, &mut full, param, index, surface, rectangle)?;
        (sample.point, sample.u, sample.v) = (full.point, full.u, full.v);
        Ok(found)
    }

    /// `PointInTheFace(F, P, u, v, Param, Index, surf, U1, V1, U2, V2, theVecD1U,
    /// theVecD1V)`: for a face of the shape, first the given (u, v) when it lies in the
    /// rectangle (on a non-periodic direction), is IN and its surface point is within
    /// Precision::Confusion() of `P`; then, from the `index`-th point on, the points of
    /// four quarter grids of step 1/6 of the rectangle from its centre, of a grid of
    /// step 1/37 and the centre, IN the face (ON near an edge or vertex of the tree);
    /// `index` becomes the rank of the point found (or the number of points tried).
    /// Otherwise (or for a face of another shape) `FindAPointInTheFace` with `param`;
    /// `explorer.point_in_the_face_on_surface_d1(&face, &mut s, &mut param, &mut index, &surf, [0., 0., 1., 1.])?`.
    pub fn point_in_the_face_on_surface_d1(
        &self,
        face: &Face,
        sample: &mut FacePointSample,
        param: &mut f64,
        index: &mut i32,
        surface: &BRepAdaptorSurface,
        rectangle: UvRectangle,
    ) -> Result<bool, Failure> {
        if let Some(intersector) = self.my_map_of_inter.seek(face) {
            let mut search = GridSearch {
                explorer: self,
                intersector,
                surface,
                count: 0,
            };
            if keeps_given_point(&search, sample, rectangle)?
                || grid_point(&mut search, rectangle, index, sample)?
            {
                return Ok(true);
            }
            *index = search.count;
        }
        Self::find_a_point_in_the_face_d1(face, sample, param)
    }
}

/// `LimitInfiniteUV`: infinite bounds become ±1e10.
fn limit_infinite_uv([u1, v1, u2, v2]: UvRectangle) -> UvRectangle {
    let low = |x: f64| {
        if precision::is_negative_infinite(x) {
            -1e10
        } else {
            x
        }
    };
    let high = |x: f64| {
        if precision::is_positive_infinite(x) {
            1e10
        } else {
            x
        }
    };
    [low(u1), low(v1), high(u2), high(v2)]
}

/// The given (u, v) when it lies in the rectangle (checked on the non-periodic
/// directions), is IN, and its surface point is the given point; the derivatives are
/// written in any case once the point is IN.
fn keeps_given_point(
    search: &GridSearch<'_>,
    sample: &mut FacePointSample,
    [u1, v1, u2, v2]: UvRectangle,
) -> Result<bool, Failure> {
    let surface = search.surface;
    let mut is_inside = true;
    if !surface.is_u_periodic()? {
        is_inside = sample.u >= u1 && sample.u <= u2;
    }
    if !surface.is_v_periodic()? {
        is_inside &= sample.v >= v1 && sample.v <= v2;
    }
    // Check if the point is already in the face
    if !is_inside
        || search.explorer.classify_uv_point(
            search.intersector,
            surface,
            Point2::new(sample.u, sample.v),
        )? != State::In
    {
        return Ok(false);
    }
    let d1 = surface.d1(sample.u, sample.v)?;
    (sample.d1u, sample.d1v) = (d1.d1u, d1.d1v);
    let confusion: f64 = precision::confusion();
    Ok(d1.point.square_distance(&sample.point) < confusion * confusion)
}

/// Floors a grid step at 1e-12.
fn floored_step(step: f64) -> f64 {
    if step < 1e-12 { 1e-12 } else { step }
}

/// The quarter grids (steps of a sixth, from the centre), the 1/37 grid and the centre.
fn grid_point(
    search: &mut GridSearch<'_>,
    [u1, v1, u2, v2]: UvRectangle,
    index: &mut i32,
    sample: &mut FacePointSample,
) -> Result<bool, Failure> {
    let (du, dv) = (floored_step((u2 - u1) / 6.0), floored_step((v2 - v1) / 6.0));
    let (um, vm) = ((u1 + u2) * 0.5, (v1 + v2) * 0.5);
    let up = Sweep {
        start: du + um,
        step: du,
        bound: u2,
    };
    let down = Sweep {
        start: -du + um,
        step: -du,
        bound: u1,
    };
    let vup = Sweep {
        start: dv + vm,
        step: dv,
        bound: v2,
    };
    let vdown = Sweep {
        start: -dv + vm,
        step: -dv,
        bound: v1,
    };
    //-- Take 4 points in each Quarter of surface, then a matrix of points on a tight grid
    for (us, vs) in [(up, vup), (down, vdown), (down, vup), (up, vdown)] {
        if search.scan(us, vs, index, sample)? {
            return Ok(true);
        }
    }
    //-- the remainder
    let (du, dv) = (
        floored_step((u2 - u1) / 37.0),
        floored_step((v2 - v1) / 37.0),
    );
    let us = Sweep {
        start: du + u1,
        step: du,
        bound: u2,
    };
    let vs = Sweep {
        start: dv + v1,
        step: dv,
        bound: v2,
    };
    if search.scan(us, vs, index, sample)? {
        return Ok(true);
    }
    search.try_point(um, vm, index, sample)
}
