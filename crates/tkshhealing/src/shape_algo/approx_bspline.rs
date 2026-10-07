// Rust translation of ShapeAlgo_AlgoContainer::ApproxBSplineCurve (both overloads) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ApproxBSplineCurve`: a polyline B-spline cut at its coincident poles, each piece
//! smoothed by GeomInt_WLApprox through its poles.
use super::ShapeAlgoAlgoContainer;
use super::algo_container::null_object;
use rustycad_tkernel::ncollection::{Array1, Sequence};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::{one_based, put};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkgeomalgo::approx_int::ApproxIntRange;
use rustycad_tkgeomalgo::geom_int::GeomIntWLApprox;
use rustycad_tkgeomalgo::int_patch::IntPatchWLine;
use rustycad_tkgeomalgo::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use rustycad_tkgeombase::app_par_curves::{MultiBSpCurve, MultiCurveEval};
use rustycad_tkgeombase::approx::ParametrizationType;
use rustycad_tkmath::gp::{Point2, Point3};

/// One piece of the cut: `jpole` poles from `PoleIndex` and the knots from `I1`.
struct PolePiece {
    first_pole: i32,
    nb_poles: i32,
    first_knot: i32,
    nb_knots: i32,
}

/// The knots, multiplicities and weights of one piece (the end multiplicities set to
/// degree + 1, every weight that of the first pole of the piece, as natively).
struct PieceData {
    knots: Array1<'static, f64>,
    mults: Array1<'static, i32>,
    weights: Array1<'static, f64>,
}

impl ShapeAlgoAlgoContainer {
    /// `ApproxBSplineCurve(bspline, seq)` (3D): a curve of degree other than 1 as is;
    /// else the pieces between coincident poles (Precision::Confusion), each of more than
    /// 2 poles replaced by its GeomInt_WLApprox through the poles (Precision::Approximation,
    /// degrees 4 to 8) when that gives one multi-curve.
    ///
    /// Native quirk (kept): once a piece is cut, the poles after the last coincident pair
    /// are dropped; the knots of a piece are read from the pole index, so a knot index past
    /// the end is Standard_OutOfRange (undefined behaviour natively).
    /// Example: `let pieces = container.approx_bspline_curve(&polyline)?;`.
    pub fn approx_bspline_curve(
        &self,
        bspline: &Handle<GeomBSplineCurve>,
    ) -> Result<Sequence<Handle<dyn GeomCurve>>, Failure> {
        let curve = bspline.get().ok_or_else(|| null_object("bspline"))?;
        let mut seq = Sequence::default();
        if curve.degree() != 1 {
            seq.append(bspline.clone().cast());
            return Ok(seq);
        }
        let tol = precision::confusion();
        let coincident = |i: i32| Ok(curve.pole(i)?.is_equal(&curve.pole(i + 1)?, tol));
        let pieces = cut_at_coincident_poles(curve.nb_poles(), curve.degree(), coincident)?;
        let mut curves = Vec::with_capacity(pieces.len().max(1));
        for piece in &pieces {
            curves.push(Handle::new(piece_curve_3d(curve, piece)?));
        }
        if curves.is_empty() {
            curves.push(bspline.clone());
        }
        for piece in curves {
            seq.append(smoothed_3d(piece)?.cast());
        }
        Ok(seq)
    }

    /// `ApproxBSplineCurve(bspline, seq)` (2D): as the 3D overload with
    /// Precision::PConfusion, pieces of more than 10 poles approximated as (u1, v1)
    /// parameters with Precision::PApproximation.
    /// Example: `let pieces = container.approx_bspline_curve_2d(&polyline)?;`.
    pub fn approx_bspline_curve_2d(
        &self,
        bspline: &Handle<Geom2dBSplineCurve>,
    ) -> Result<Sequence<Handle<dyn Geom2dCurve>>, Failure> {
        let curve = bspline.get().ok_or_else(|| null_object("bspline"))?;
        let mut seq = Sequence::default();
        if curve.degree() != 1 {
            seq.append(bspline.clone().cast());
            return Ok(seq);
        }
        let tol = precision::p_confusion();
        let coincident = |i: i32| Ok(curve.pole(i)?.is_equal(curve.pole(i + 1)?, tol));
        let pieces = cut_at_coincident_poles(curve.nb_poles(), curve.degree(), coincident)?;
        let mut curves = Vec::with_capacity(pieces.len().max(1));
        for piece in &pieces {
            curves.push(Handle::new(piece_curve_2d(curve, piece)?));
        }
        if curves.is_empty() {
            curves.push(bspline.clone());
        }
        for piece in curves {
            seq.append(smoothed_2d(piece)?.cast());
        }
        Ok(seq)
    }
}

/// The native loop over the poles: a coincident pair closes the current piece, or skips
/// a pole when the piece has only one.
fn cut_at_coincident_poles(
    nb_poles: i32,
    degree: i32,
    coincident: impl Fn(i32) -> Result<bool, Failure>,
) -> Result<Vec<PolePiece>, Failure> {
    let (mut jpole, mut pole_index, mut i1) = (1, 1, 1);
    let mut pieces = Vec::new();
    for ipole in 1..nb_poles {
        if !coincident(ipole)? {
            jpole += 1;
            continue;
        }
        if jpole == 1 {
            pole_index += 1;
            continue;
        }
        pieces.push(PolePiece {
            first_pole: pole_index,
            nb_poles: jpole,
            first_knot: i1,
            nb_knots: jpole - degree + 1,
        });
        pole_index += jpole;
        i1 = ipole + 1;
        jpole = 1;
    }
    Ok(pieces)
}

/// The knots, multiplicities and weights of `piece` read through the accessors.
fn piece_data(
    piece: &PolePiece,
    degree: i32,
    knot: impl Fn(i32) -> Result<f64, Failure>,
    mult: impl Fn(i32) -> Result<i32, Failure>,
    weight: impl Fn(i32) -> Result<f64, Failure>,
) -> Result<PieceData, Failure> {
    let mut data = PieceData {
        knots: one_based(piece.nb_knots)?,
        mults: one_based(piece.nb_knots)?,
        weights: one_based(piece.nb_poles)?,
    };
    for j in 1..=piece.nb_knots {
        put(&mut data.knots, j, knot(piece.first_knot + j - 1)?)?;
        put(&mut data.mults, j, mult(piece.first_knot + j - 1)?)?;
    }
    put(&mut data.mults, 1, degree + 1)?;
    put(&mut data.mults, piece.nb_knots, degree + 1)?;
    for j in 1..=piece.nb_poles {
        put(&mut data.weights, j, weight(piece.first_pole + j - 1)?)?;
    }
    Ok(data)
}

fn piece_curve_3d(
    curve: &GeomBSplineCurve,
    piece: &PolePiece,
) -> Result<GeomBSplineCurve, Failure> {
    let degree = curve.degree();
    let data = piece_data(
        piece,
        degree,
        |i| curve.knot(i),
        |i| curve.multiplicity(i),
        |i| curve.weight(i),
    )?;
    let mut poles = one_based(piece.nb_poles)?;
    for j in 1..=piece.nb_poles {
        put(&mut poles, j, curve.pole(piece.first_pole + j - 1)?)?;
    }
    GeomBSplineCurve::with_weights(
        &poles,
        &data.weights,
        &data.knots,
        &data.mults,
        degree,
        false,
    )
}

fn piece_curve_2d(
    curve: &Geom2dBSplineCurve,
    piece: &PolePiece,
) -> Result<Geom2dBSplineCurve, Failure> {
    let degree = curve.degree();
    let data = piece_data(
        piece,
        degree,
        |i| curve.knot(i),
        |i| curve.multiplicity(i),
        |i| curve.weight(i),
    )?;
    let mut poles = one_based(piece.nb_poles)?;
    for j in 1..=piece.nb_poles {
        put(&mut poles, j, curve.pole(piece.first_pole + j - 1)?)?;
    }
    Geom2dBSplineCurve::with_weights(
        &poles,
        &data.weights,
        &data.knots,
        &data.mults,
        degree,
        false,
    )
}

/// GeomInt_WLApprox through the points of `line` (`SetParameters(tol, tol, 4, 8, 0, 30,
/// true)`); the multi-curve when it is done and single.
fn approximate(
    line: IntSurfLineOn2S,
    tol: f64,
    range: ApproxIntRange,
) -> Result<Option<MultiBSpCurve>, Failure> {
    let mut approx = GeomIntWLApprox::new();
    approx.set_parameters(
        tol,
        tol,
        4,
        8,
        0,
        30,
        true,
        ParametrizationType::ChordLength,
    );
    let wline = Handle::new(IntPatchWLine::new_undecided(&Handle::new(line), false));
    approx.perform(&wline, range)?;
    if !approx.is_done() || approx.nb_multi_curves() != 1 {
        return Ok(None);
    }
    Ok(Some(approx.value(1).clone()))
}

fn smoothed_3d(piece: Handle<GeomBSplineCurve>) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let curve = piece.get().ok_or_else(|| null_object("piece"))?;
    let jpole = curve.nb_poles();
    if jpole <= 2 {
        return Ok(piece);
    }
    let line = IntSurfLineOn2S::new(None);
    for j in 1..=jpole {
        let mut point = IntSurfPntOn2S::default();
        point.set_value_all(curve.pole(j)?, 0., 0., 1., 1.);
        line.add(&point);
    }
    let range = wline_range(true, false, jpole);
    let Some(mbspc) = approximate(line, precision::approximation(), range)? else {
        return Ok(piece);
    };
    let mut poles: Array1<'static, Point3> = one_based(mbspc.nb_poles())?;
    mbspc.curve_3d(1, &mut poles)?;
    let (knots, mults) = (mbspc.knots()?, mbspc.multiplicities()?);
    Ok(Handle::new(GeomBSplineCurve::new(
        &poles,
        &knots,
        &mults,
        mbspc.degree()?,
        false,
    )?))
}

fn smoothed_2d(piece: Handle<Geom2dBSplineCurve>) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let curve = piece.get().ok_or_else(|| null_object("piece"))?;
    let jpole = curve.nb_poles();
    if jpole <= 10 {
        return Ok(piece);
    }
    let line = IntSurfLineOn2S::new(None);
    for j in 1..=jpole {
        let pole = curve.pole(j)?;
        let mut point = IntSurfPntOn2S::default();
        // The native 3D points are default gp_Pnt: the origin.
        point.set_value_all(Point3::new(0., 0., 0.), pole.x(), pole.y(), 1., 1.);
        line.add(&point);
    }
    let range = wline_range(false, true, jpole);
    let Some(mbspc) = approximate(line, precision::p_approximation(), range)? else {
        return Ok(piece);
    };
    let mut poles: Array1<'static, Point2> = one_based(mbspc.nb_poles())?;
    mbspc.curve_2d(1, &mut poles)?;
    let (knots, mults) = (mbspc.knots()?, mbspc.multiplicities()?);
    Ok(Handle::new(Geom2dBSplineCurve::new(
        &poles,
        &knots,
        &mults,
        mbspc.degree()?,
        false,
    )?))
}

fn wline_range(xyz: bool, u1v1: bool, jpole: i32) -> ApproxIntRange {
    ApproxIntRange {
        approx_xyz: xyz,
        approx_u1v1: u1v1,
        approx_u2v2: false,
        indicemin: 1,
        indicemax: jpole,
    }
}
