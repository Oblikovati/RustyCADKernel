// SPDX-License-Identifier: GPL-2.0-only
//! Clamped B-splines (ADR 0083): an admitted spline and its exact-in-structure split into
//! Bézier pieces by knot insertion (Piegl and Tiller, "The NURBS Book", A5.6), in
//! homogeneous coordinates for rational splines. Adjacent pieces share one computed
//! junction control, so the pieces join bitwise.
use super::bezier::BezierPiece;

/// A checked clamped spline in plane coordinates; its end poles are the authored vertices.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AdmittedSpline {
    pub degree: usize,
    pub poles: Vec<[f64; 2]>,
    pub weights: Option<Vec<f64>>,
    pub knots: Vec<f64>,
}

impl AdmittedSpline {
    /// Distinct knots and their multiplicities, as the native constructor wants them.
    pub(crate) fn distinct_knots(&self) -> (Vec<f64>, Vec<i32>) {
        let mut knots: Vec<f64> = Vec::new();
        let mut mults: Vec<i32> = Vec::new();
        for knot in &self.knots {
            match knots.last() {
                Some(last) if last == knot => *mults.last_mut().expect("paired") += 1,
                _ => {
                    knots.push(*knot);
                    mults.push(1);
                }
            }
        }
        (knots, mults)
    }

    /// Bézier pieces in parameter order, one per nonempty knot span.
    pub(crate) fn bezier_pieces(&self) -> Vec<BezierPiece> {
        let weights = |i: usize| self.weights.as_ref().map_or(1.0, |w| w[i]);
        let homogeneous: Vec<[f64; 3]> = (0..self.poles.len())
            .map(|i| {
                let [x, y] = self.poles[i];
                [x * weights(i), y * weights(i), weights(i)]
            })
            .collect();
        let pieces = decompose(self.degree, &self.knots, &homogeneous);
        let rational = self.weights.is_some();
        let mut pieces: Vec<BezierPiece> = pieces.iter().map(|q| cartesian(q, rational)).collect();
        // The clamped ends are the poles themselves; keep them bitwise.
        pieces[0].controls[0] = self.poles[0];
        let last = pieces.last_mut().expect("a clamped spline has a span");
        *last.controls.last_mut().expect("degree >= 1") = *self.poles.last().expect("poles");
        pieces
    }
}

fn cartesian(piece: &[[f64; 3]], rational: bool) -> BezierPiece {
    let controls = piece.iter().map(|[x, y, w]| [x / w, y / w]).collect();
    let weights = rational.then(|| piece.iter().map(|q| q[2]).collect());
    BezierPiece { controls, weights }
}

/// Homogeneous Bézier pieces of a clamped spline of degree `p` (A5.6).
fn decompose(p: usize, knots: &[f64], poles: &[[f64; 3]]) -> Vec<Vec<[f64; 3]>> {
    let m = knots.len() - 1;
    let (mut a, mut b) = (p, p + 1);
    let mut pieces = vec![poles[..=p].to_vec()];
    while b < m {
        let i = b;
        while b < m && knots[b + 1] == knots[b] {
            b += 1;
        }
        let multiplicity = b - i + 1;
        let next = refine(
            p,
            knots,
            [a, b],
            multiplicity,
            pieces.last_mut().expect("piece"),
        );
        if b < m {
            let mut piece = next;
            for (k, slot) in piece.iter_mut().enumerate().skip(p - multiplicity.min(p)) {
                *slot = poles[b - p + k];
            }
            // The junction is one point; share the computed value bitwise.
            piece[0] = pieces.last().expect("piece")[p];
            pieces.push(piece);
            (a, b) = (b, b + 1);
        }
    }
    pieces
}

/// Insert knot `knots[b]` until it has multiplicity `p` in the current piece; returns the
/// next piece's leading controls (the junction and the controls computed for it).
fn refine(
    p: usize,
    knots: &[f64],
    [a, b]: [usize; 2],
    multiplicity: usize,
    piece: &mut [[f64; 3]],
) -> Vec<[f64; 3]> {
    let mut next = vec![[0.0; 3]; p + 1];
    if multiplicity >= p {
        next[0] = piece[p];
        return next;
    }
    let numer = knots[b] - knots[a];
    let alphas: Vec<f64> = (multiplicity + 1..=p)
        .map(|j| numer / (knots[a + j] - knots[a]))
        .collect();
    let r = p - multiplicity;
    for j in 1..=r {
        let (save, s) = (r - j, multiplicity + j);
        for k in (s..=p).rev() {
            let alpha = alphas[k - s];
            piece[k] =
                std::array::from_fn(|c| alpha * piece[k][c] + (1.0 - alpha) * piece[k - 1][c]);
        }
        next[save] = piece[p];
    }
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evaluate(piece: &BezierPiece, t: f64) -> [f64; 2] {
        let n = piece.controls.len() - 1;
        let weight = |i: usize| piece.weights.as_ref().map_or(1.0, |w| w[i]);
        let basis = |i: usize| {
            let binomial = (0..i).fold(1.0, |acc, k| acc * (n - k) as f64 / (k + 1) as f64);
            binomial * t.powi(i as i32) * (1.0 - t).powi((n - i) as i32) * weight(i)
        };
        let total: f64 = (0..=n).map(basis).sum();
        std::array::from_fn(|c| {
            (0..=n)
                .map(|i| basis(i) * piece.controls[i][c])
                .sum::<f64>()
                / total
        })
    }

    #[test]
    fn uniform_cubic_splits_into_joined_pieces_on_the_curve() {
        let spline = AdmittedSpline {
            degree: 3,
            poles: vec![[0., 0.], [1., 2.], [3., 2.], [4., 0.], [6., -1.]],
            weights: None,
            knots: vec![0., 0., 0., 0., 0.5, 1., 1., 1., 1.],
        };
        let pieces = spline.bezier_pieces();
        assert_eq!(pieces.len(), 2);
        assert_eq!(pieces[0].controls[3], pieces[1].controls[0]);
        assert_eq!(pieces[0].controls[0], [0., 0.]);
        assert_eq!(pieces[1].controls[3], [6., -1.]);
        // Knot insertion keeps the curve: the junction is C(0.5) by de Boor.
        let junction = pieces[0].controls[3];
        assert!((junction[0] - 2.75).abs() < 1e-15 && (junction[1] - 1.5).abs() < 1e-15);
        assert_eq!((spline.distinct_knots().1), vec![4, 1, 4]);
        assert!(evaluate(&pieces[1], 1.0) == [6., -1.]);
    }

    #[test]
    fn rational_quadratic_keeps_the_circle() {
        // A unit circle quarter and its continuation, as one rational spline of two spans.
        let h = std::f64::consts::FRAC_1_SQRT_2;
        let spline = AdmittedSpline {
            degree: 2,
            poles: vec![[1., 0.], [1., 1.], [0., 1.], [-1., 1.], [-1., 0.]],
            weights: Some(vec![1., h, 1., h, 1.]),
            knots: vec![0., 0., 0., 0.5, 0.5, 1., 1., 1.],
        };
        let pieces = spline.bezier_pieces();
        assert_eq!(pieces.len(), 2);
        for piece in &pieces {
            for t in [0.1, 0.37, 0.8] {
                let [x, y] = evaluate(piece, t);
                assert!((x.hypot(y) - 1.0).abs() < 1e-14);
            }
        }
    }
}
