// Rust translation of GeomFill_Frenet::Init of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Frenet.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_Frenet::Init`: the search of the parameters where `C' ^ C''` vanishes
//! (the zeros of `GeomFill_SnglrFunc`, found as its extrema of distance to the
//! origin on each C2 interval) and of the neighbourhood where the frame is replaced.
use super::frame_math::std_min;
use super::frenet::{FrenetSingularities, GeomFillFrenet};
use super::snglr_func::GeomFillSnglrFunc;
use super::sweep_failures::live;
use super::trihedron_law::GeomFillTrihedronLaw;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::extrema::ExtPC;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{self, Point3};

/// `NullTol`: the norm of `C' ^ C''` treated as zero around a singularity.
const NULL_TOL: f64 = 1.0e-10;
/// `MaxSingular`: the largest half width of a singular neighbourhood.
const MAX_SINGULAR: f64 = 1.0e-5;
/// `TolF` of the extrema search.
const TOL_F: f64 = 1.0e-10;
/// `Tol2 = (10 TolF)^2`.
const TOL2: f64 = (10.0 * TOL_F) * (10.0 * TOL_F);

/// The C2 intervals of the curve and, for each, whether the function is null
/// (`IsLin`) or constant (`IsConst`) on ten samples, and the running average of its
/// norm (`AveFunc`, accumulated over the intervals as natively).
struct IntervalSamples {
    disc: Array1<'static, f64>,
    is_lin: Vec<bool>,
    is_const: Vec<bool>,
    ave_func: Vec<f64>,
}

fn sample_intervals(
    curve: &dyn Adaptor3dCurve,
    func: &GeomFillSnglrFunc,
) -> Result<IntervalSamples, Failure> {
    let tol = 10.0 * TOL_F;
    let nb_int = curve.nb_intervals(Shape::C2)?;
    let mut disc = one_based(nb_int + 1)?;
    curve.intervals(&mut disc, Shape::C2)?;
    let mut samples = IntervalSamples {
        disc,
        is_lin: Vec::new(),
        is_const: Vec::new(),
        ave_func: Vec::new(),
    };
    let mut average = 0.0;
    for i in 1..=nb_int {
        let step = (samples.disc[i + 1] - samples.disc[i]) / 10.0;
        let (mut is_lin, mut is_const, mut c1) = (true, true, Point3::default());
        for j in 1..=10 {
            let c = func.d0(samples.disc[i] + f64::from(j - 1) * step)?;
            if j == 1 {
                c1 = c;
            }
            let modulus = c.xyz().modulus();
            if modulus > tol {
                is_lin = false;
            }
            average += modulus;
            if is_const
                && ((c.x() - c1.x()).abs() > tol
                    || (c.y() - c1.y()).abs() > tol
                    || (c.z() - c1.z()).abs() > tol)
            {
                is_const = false;
            }
        }
        samples.is_lin.push(is_lin);
        samples.is_const.push(is_const);
        samples.ave_func.push(average / 10.0);
    }
    Ok(samples)
}

/// The parameters of the extrema of `ext` (run on `[a, b]` towards the origin)
/// whose square distance is below `Tol2`; none when the search is not done.
fn zeros_on<'a>(
    ext: &mut ExtPC<'a>,
    func: &'a GeomFillSnglrFunc,
    a: f64,
    b: f64,
) -> Result<Vec<f64>, Failure> {
    ext.initialize(func, a, b, TOL_F)?;
    ext.perform(Point3::new(0.0, 0.0, 0.0))?;
    let mut zeros = Vec::new();
    if !ext.is_done() {
        return Ok(zeros);
    }
    for j in 1..=ext.nb_ext()? {
        if ext.square_distance(j)? < TOL2 {
            zeros.push(ext.point(j)?.parameter());
        }
    }
    Ok(zeros)
}

fn sort_reals(values: &mut [f64]) {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
}

/// The first search: the zeros on each interval where the function is neither null
/// nor constant, sorted per interval.
fn first_zeros<'a>(
    samples: &IntervalSamples,
    func: &'a GeomFillSnglrFunc,
    ext: &mut ExtPC<'a>,
) -> Result<Vec<Vec<f64>>, Failure> {
    let mut seq_array = vec![Vec::new(); samples.is_lin.len()];
    for (index, seq) in seq_array.iter_mut().enumerate() {
        if samples.is_lin[index] || samples.is_const[index] {
            continue;
        }
        let i = index as i32 + 1;
        func.set_ratio(1.0 / samples.ave_func[index]);
        *seq = zeros_on(ext, func, samples.disc[i], samples.disc[i + 1])?;
        sort_reals(seq);
    }
    Ok(seq_array)
}

/// The second search, between the interval bounds and the zeros already found;
/// the new zeros strictly inside each sub-interval are appended to `sngl_seq`.
fn second_zeros<'a>(
    samples: &IntervalSamples,
    func: &'a GeomFillSnglrFunc,
    ext: &mut ExtPC<'a>,
    seq_array: Vec<Vec<f64>>,
    sngl_seq: &mut Vec<f64>,
) -> Result<(), Failure> {
    let p_tol = precision::p_confusion();
    for (index, mut seq) in seq_array.into_iter().enumerate() {
        if seq.is_empty() {
            continue;
        }
        let i = index as i32 + 1;
        seq.insert(0, samples.disc[i]);
        seq.push(samples.disc[i + 1]);
        func.set_ratio(1.0 / samples.ave_func[index]);
        for pair in seq.windows(2) {
            if pair[1] - pair[0] <= p_tol {
                continue;
            }
            for t in zeros_on(ext, func, pair[0], pair[1])? {
                if t - pair[0] > p_tol && pair[1] - t > p_tol {
                    sngl_seq.push(t);
                }
            }
        }
    }
    Ok(())
}

/// Removes the second of two sorted values closer than PConfusion, restarting the
/// scan at the removal point, as the native `while (found)` loop.
fn merge_close(sngl_seq: &mut Vec<f64>) {
    let p_tol = precision::p_confusion();
    let mut j = 0;
    while let Some(i) =
        (j..sngl_seq.len().saturating_sub(1)).find(|&i| sngl_seq[i + 1] - sngl_seq[i] <= p_tol)
    {
        sngl_seq.remove(i + 1);
        j = i;
    }
}

/// The half width of the neighbourhood of each singular parameter.
fn neighbourhoods(func: &GeomFillSnglrFunc, sngl: &[f64]) -> Result<Vec<f64>, Failure> {
    let mut lengths = Vec::with_capacity(sngl.len());
    for &s in sngl {
        let (_, der, der2) = func.d2(s)?;
        let norm = der.magnitude();
        let length = if norm > gp::resolution() {
            std_min(NULL_TOL / norm, MAX_SINGULAR)
        } else if der2.magnitude() > gp::resolution() {
            std_min((2.0 * NULL_TOL / der2.magnitude()).sqrt(), MAX_SINGULAR)
        } else {
            MAX_SINGULAR
        };
        lengths.push(length);
    }
    Ok(lengths)
}

/// Merges the overlapping neighbourhoods `(center, half width)`.
fn merge_neighbourhoods(sngl: &[f64], lengths: &[f64]) -> Vec<(f64, f64)> {
    let mut tmp = vec![(sngl[0], lengths[0])];
    for i in 1..sngl.len() {
        let last = *tmp.last().expect("one neighbourhood at least");
        let u12 = last.0 + last.1;
        let u21 = sngl[i] - lengths[i];
        if u12 >= u21 {
            let u11 = last.0 - last.1;
            let u22 = sngl[i] + lengths[i];
            *tmp.last_mut().expect("one neighbourhood at least") =
                ((u11 + u22) / 2.0, (u22 - u11) / 2.0);
        } else {
            tmp.push((sngl[i], lengths[i]));
        }
    }
    tmp
}

fn to_array(values: impl ExactSizeIterator<Item = f64>) -> Result<Array1<'static, f64>, Failure> {
    let mut array = one_based(values.len() as i32)?;
    for (index, value) in values.enumerate() {
        array[index as i32 + 1] = value;
    }
    Ok(array)
}

impl GeomFillFrenet {
    /// `Init()`: finds the singular parameters of the loaded curve and their
    /// neighbourhoods; `law.init()?`.
    pub fn init(&self) -> Result<(), Failure> {
        let curve: Handle<dyn Adaptor3dCurve> = self.trihedron_curve();
        let c = live(&curve, "GeomFill_Frenet::Init", "myCurve")?;
        let func = GeomFillSnglrFunc::new(&curve);
        let samples = sample_intervals(c, &func)?;
        let mut ext = ExtPC::new();
        let seq_array = first_zeros(&samples, &func, &mut ext)?;
        let mut sngl_seq: Vec<f64> = seq_array.iter().flatten().copied().collect();
        second_zeros(&samples, &func, &mut ext, seq_array, &mut sngl_seq)?;
        if sngl_seq.is_empty() {
            self.singular.write().is_sngl = false;
            return Ok(());
        }
        sort_reals(&mut sngl_seq);
        merge_close(&mut sngl_seq);
        let lengths = neighbourhoods(&func, &sngl_seq)?;
        let (sngl, sngl_len) = if sngl_seq.len() > 1 {
            let merged = merge_neighbourhoods(&sngl_seq, &lengths);
            (
                to_array(merged.iter().map(|m| m.0))?,
                to_array(merged.iter().map(|m| m.1))?,
            )
        } else {
            (
                to_array(sngl_seq.into_iter())?,
                to_array(lengths.into_iter())?,
            )
        };
        *self.singular.write() = FrenetSingularities {
            sngl,
            sngl_len,
            is_sngl: true,
        };
        Ok(())
    }

    fn trihedron_curve(&self) -> Handle<dyn Adaptor3dCurve> {
        self.trihedron_base().curve()
    }
}
