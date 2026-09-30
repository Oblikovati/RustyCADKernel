//! Evaluation of canonical polynomial curves: `mmpocrb_`, `mmmpocur_` and the
//! file-static `mvpscr2_`, `mvpscr3_`.
use crate::adv_app2var::f2c::{F2cArray, F2cArrayMut};
use crate::adv_app2var::sys_base::miraz_;

/// The point at `tparam` of the canonical curve `courbe` (`(ndimax, ncoeff)`) into
/// the first `ndim` values of `pntcrb` (`mmpocrb_`), with the 2D and 3D shortcuts
/// and the exact values at 0 and 1; returns 0.
/// `mmpocrb_(3, 4, &courbe, 3, 0.25, &mut pnt)`.
pub fn mmpocrb_(
    ndimax: i32,
    ncoeff: i32,
    courbe: &[f64],
    ndim: i32,
    tparam: f64,
    pntcrb: &mut [f64],
) -> i32 {
    miraz_(ndim << 3, pntcrb);
    if ncoeff <= 0 {
        return 0;
    }
    if ndim == 3 && ndimax == 3 {
        mvpscr_(3, ncoeff, courbe, tparam, pntcrb);
        return 0;
    }
    if ndim == 2 && ndimax == 2 {
        mvpscr_(2, ncoeff, courbe, tparam, pntcrb);
        return 0;
    }
    let dim1: i32 = ndimax;
    let courbe = F2cArray::new(courbe, dim1 + 1);
    let mut pnt = F2cArrayMut::new(pntcrb, 1);
    if tparam == 0. {
        for nd in 1..=ndim {
            pnt[nd] = courbe[nd + dim1];
        }
    } else if tparam == 1. {
        for ncf in 1..=ncoeff {
            for nd in 1..=ndim {
                pnt[nd] += courbe[nd + ncf * dim1];
            }
        }
    } else {
        let ncof2: i32 = ncoeff + 2;
        for nd in 1..=ndim {
            for ncf in 2..=ncoeff {
                let kcf: i32 = ncof2 - ncf;
                pnt[nd] = (pnt[nd] + courbe[nd + kcf * dim1]) * tparam;
            }
            pnt[nd] += courbe[nd + dim1];
        }
    }
    0
}

/// `mvpscr2_` and `mvpscr3_`: the point of a 2D or 3D canonical curve, with the sum
/// of the coefficients at 1 and the constant term at 0.
fn mvpscr_(ndim: i32, ncoeff: i32, curve: &[f64], tparam: f64, pntcrb: &mut [f64]) {
    let curve = F2cArray::new(curve, ndim + 1);
    let mut pnt = F2cArrayMut::new(pntcrb, 1);
    let mut acc: [f64; 3] = [0.; 3];
    if ncoeff < 2 {
        for c in 1..=ndim {
            pnt[c] = if ncoeff <= 0 { 0. } else { curve[ndim + c] };
        }
        return;
    }
    if tparam == 1. {
        for kk in 1..=ncoeff {
            for c in 1..=ndim {
                acc[(c - 1) as usize] += curve[kk * ndim + c];
            }
        }
    } else if tparam == 0. {
        for c in 1..=ndim {
            pnt[c] = curve[ndim + c];
        }
        return;
    } else {
        for c in 1..=ndim {
            acc[(c - 1) as usize] = curve[ncoeff * ndim + c];
        }
        for kk in (1..=ncoeff - 1).rev() {
            for c in 1..=ndim {
                let slot: usize = (c - 1) as usize;
                acc[slot] = acc[slot] * tparam + curve[kk * ndim + c];
            }
        }
    }
    for c in 1..=ndim {
        pnt[c] = acc[(c - 1) as usize];
    }
}

/// The value at `tparam` of the canonical curve `courbe` (`(ncofmx, ndim)`) with
/// `ndeg` coefficients, by the Horner scheme (`mmmpocur_`); zero when `ndeg < 1`.
/// Returns 0. `mmmpocur_(4, 1, 4, &courbe, 0.5, &mut value)`.
pub fn mmmpocur_(
    ncofmx: i32,
    ndim: i32,
    ndeg: i32,
    courbe: &[f64],
    tparam: f64,
    tabval: &mut [f64],
) -> i32 {
    let dim1: i32 = ncofmx;
    let courbe = F2cArray::new(courbe, dim1 + 1);
    let mut tabval = F2cArrayMut::new(tabval, 1);
    if ndeg < 1 {
        for nd in 1..=ndim {
            tabval[nd] = 0.;
        }
        return 0;
    }
    for nd in 1..=ndim {
        let mut fu: f64 = courbe[ndeg + nd * dim1];
        for i in (1..=ndeg - 1).rev() {
            fu = fu * tparam + courbe[i + nd * dim1];
        }
        tabval[nd] = fu;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curves_evaluate_with_every_shortcut() {
        // (1, 10) + (2, 20) t + (3, 30) t^2 in 2D, then in the generic 1D path.
        let courbe: [f64; 6] = [1.0, 10.0, 2.0, 20.0, 3.0, 30.0];
        let mut pnt: [f64; 2] = [9.0; 2];
        mmpocrb_(2, 3, &courbe, 2, 0.5, &mut pnt);
        assert_eq!(pnt, [2.75, 27.5]);
        mmpocrb_(2, 3, &courbe, 2, 1.0, &mut pnt);
        assert_eq!(pnt, [6.0, 60.0]);
        mmpocrb_(2, 3, &courbe, 1, 0.5, &mut pnt);
        assert_eq!(pnt[0], 2.75);
        mmpocrb_(2, 0, &courbe, 1, 0.5, &mut pnt);
        assert_eq!(pnt[0], 0.0);
        let mut value: [f64; 1] = [0.0];
        mmmpocur_(3, 1, 3, &[1.0, 2.0, 3.0], 2.0, &mut value);
        assert_eq!(value[0], 17.0);
    }
}
