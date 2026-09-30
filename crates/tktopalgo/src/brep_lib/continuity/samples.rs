// Rust translation of the sampling loop of BRepLib::ContinuityOfFaces (BRepLib.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The sampling loop of `BRepLib::ContinuityOfFaces`: G1/C1 from the derivatives
//! orthogonal to the edge, G2/C2 from the principal curvatures.
use super::properties::SurfaceProperties;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::extrema::LocateExtPC;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Direction3, Vector3};
use std::cell::OnceCell;

/// The inputs of the sampling loop.
pub(super) struct Sampler<'a> {
    pub(super) edge: &'a Edge,
    pub(super) face2: &'a Face,
    pub(super) in_face2: &'a Edge,
    pub(super) angle_tol: f64,
    pub(super) faces_reversed: (bool, bool),
}

/// The derivatives of the two surfaces orthogonal to the edge and their square lengths.
struct Derivatives {
    der1: Vector3,
    len1: f64,
    der2: Vector3,
    len2: f64,
}

impl Derivatives {
    fn is_smooth_suspect(&self, angle_tol2: f64) -> bool {
        self.der1.cross_square_magnitude(self.der2) <= angle_tol2 * self.len1 * self.len2
    }
}

impl Sampler<'_> {
    /// The lowest continuity over the 21 samples, starting from C2 (CN for elementary
    /// surfaces), or C0 as soon as one sample is not smooth.
    pub(super) fn run(
        &self,
        mut sp1: SurfaceProperties,
        mut sp2: SurfaceProperties,
        is_elementary: bool,
    ) -> Result<Continuity, Failure> {
        let (mut f, mut l) = BRepTool::range(self.edge)?;
        let eps: f64 = (l - f) / 100.;
        f += eps; // to avoid calculations on
        l -= eps; // points of pointed squares.
        let adaptor2: OnceCell<BRepAdaptorCurve> = OnceCell::new();
        let mut ext: Option<LocateExtPC<'_>> = None;
        let mut cont: Continuity = if is_elementary {
            Continuity::CN
        } else {
            Continuity::C2
        };
        let mut i: i32 = 0;
        while i <= 20 && cont > Continuity::C0 {
            // First suppose that this is sameParameter
            let u: f64 = f + (l - f) * f64::from(i) / 20.;
            i += 1;
            sp1.calculate(u)?;
            sp2.calculate(u)?;
            let mut der = Derivatives {
                der1: sp1.derivative()?,
                len1: 0.,
                der2: sp2.derivative()?,
                len2: 0.,
            };
            (der.len1, der.len2) = (der.der1.square_magnitude(), der.der2.square_magnitude());
            let mut smooth: bool = der.is_smooth_suspect(self.angle_tol * self.angle_tol);
            if smooth && self.folded(&mut sp1, &mut sp2)? {
                return Ok(Continuity::C0);
            }
            if !smooth {
                // Refine by projection
                if ext.is_none() {
                    let curve = adaptor2.get_or_init_try(|| {
                        BRepAdaptorCurve::with_edge_on_face(self.in_face2, self.face2)
                    })?;
                    let mut locate = LocateExtPC::new();
                    locate.initialize(curve, (f, l), precision::p_confusion())?;
                    ext = Some(locate);
                }
                let locate = ext.as_mut().expect("initialized above");
                locate.perform(sp1.value(), u)?;
                if locate.is_done() && locate.is_min()? {
                    sp2.calculate(locate.point()?.parameter())?;
                    der.der2 = sp2.derivative()?;
                    der.len2 = der.der2.square_magnitude();
                }
                smooth = der.is_smooth_suspect(self.angle_tol * self.angle_tol);
            }
            if !smooth {
                return Ok(Continuity::C0);
            }
            let mut cur_cont = Continuity::G1;
            if (der.len1.sqrt() - der.len2.sqrt()).abs() < precision::confusion()
                && der.der1.dot(der.der2) > precision::square_confusion()
            {
                // <= check vectors are codirectional
                cur_cont = Continuity::C1;
            }
            if cont < Continuity::G2 {
                continue; // no need further processing, because maximal continuity is less than G2
            }
            cur_cont = curvature_continuity(&mut sp1, &mut sp2, cur_cont)?;
            if cur_cont < cont {
                cont = cur_cont;
            }
        }
        Ok(cont)
    }

    /// Whether the normals (oriented by the faces) point to opposite sides.
    fn folded(
        &self,
        sp1: &mut SurfaceProperties,
        sp2: &mut SurfaceProperties,
    ) -> Result<bool, Failure> {
        let mut normal1: Direction3 = sp1.normal()?;
        if self.faces_reversed.0 {
            normal1.reverse();
        }
        let mut normal2: Direction3 = sp2.normal()?;
        if self.faces_reversed.1 {
            normal2.reverse();
        }
        Ok(normal1.dot(normal2) < 0.)
    }
}

/// G2 when the principal directions are parallel and the curvatures equal (in either
/// pairing), C2 when moreover `cur_cont` is C1 and the directions codirectional; else
/// `cur_cont`.
fn curvature_continuity(
    sp1: &mut SurfaceProperties,
    sp2: &mut SurfaceProperties,
    cur_cont: Continuity,
) -> Result<Continuity, Failure> {
    let (dir1, len1) = sp1.curvature()?;
    let (dir2, len2) = sp2.curvature()?;
    let sq_confusion: f64 = precision::square_confusion();
    let confusion: f64 = precision::confusion();
    for step in 0..=1usize {
        let matches: bool = dir1[0].xyz().cross_square_magnitude(dir2[step].xyz()) <= sq_confusion
            && (len1[0] - len2[step]).abs() < confusion
            && dir1[1].xyz().cross_square_magnitude(dir2[1 - step].xyz()) <= sq_confusion
            && (len1[1] - len2[1 - step]).abs() < confusion;
        if !matches {
            continue;
        }
        if cur_cont == Continuity::C1
            && dir1[0].dot(dir2[step]) > confusion
            && dir1[1].dot(dir2[1 - step]) > confusion
        {
            return Ok(Continuity::C2);
        }
        return Ok(Continuity::G2);
    }
    Ok(cur_cont)
}

/// `OnceCell::get_or_try_init`, which is unstable.
trait GetOrInitTry<T> {
    fn get_or_init_try(&self, init: impl FnOnce() -> Result<T, Failure>) -> Result<&T, Failure>;
}

impl<T> GetOrInitTry<T> for OnceCell<T> {
    fn get_or_init_try(&self, init: impl FnOnce() -> Result<T, Failure>) -> Result<&T, Failure> {
        if let Some(value) = self.get() {
            return Ok(value);
        }
        let _ = self.set(init()?);
        Ok(self.get().expect("set above"))
    }
}
