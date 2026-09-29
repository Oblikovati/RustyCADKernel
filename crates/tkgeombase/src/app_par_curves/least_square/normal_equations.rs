// Copyright (c) 1995-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingData/TKGeomBase/AppParCurves/AppParCurves_LeastSquare.gxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppParCurves_LeastSquare::MakeTAA` (three overloads) and `SearchIndex`: the
//! normal equations tA A X = tA B in the profile storage of DACTCL.
use super::LeastSquare;
use crate::app_par_curves::constraint::Constraint;
use crate::native_failure::array_failure;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{IntegerVector, Matrix, Vector};

/// The row walk of the banded profile: `(iinit, ifin, jinit)` per knot span, as
/// SearchIndex and MakeTAA advance it (a Bezier fit has one span).
struct Band {
    iinit: i32,
    ifin: i32,
    jinit: i32,
}

impl LeastSquare {
    // The spans of the profile (`len - 1` of them, len = number of knots or 2).
    fn bands(&self) -> Result<Vec<Band>, Failure> {
        let len: i32 = self.knots.as_ref().map_or(2, |knots| knots.length());
        let mut band: Band = Band {
            iinit: self.resinit,
            ifin: self.resfin.min(self.deg + 1),
            jinit: self.resinit,
        };
        let mut bands: Vec<Band> = Vec::new();
        for k in 2..=len {
            let next: Option<Band> = match &self.mults {
                Some(mults) => {
                    let d: i32 = band.ifin
                        + *mults
                            .value(k)
                            .map_err(|error| array_failure(error, "AppParCurves_LeastSquare"))?;
                    Some(Band {
                        iinit: band.ifin + 1,
                        ifin: d.min(self.resfin),
                        jinit: self.resinit.max(d - self.deg),
                    })
                }
                None => None,
            };
            bands.push(Band { ..band });
            if let Some(next) = next {
                band = next;
            }
        }
        Ok(bands)
    }

    /// `SearchIndex(Index)`: the diagonal addresses of the profile of tA A.
    pub(super) fn search_index(&self, index: &mut IntegerVector<'_>) -> Result<(), Failure> {
        let nincx: i32 = self.resfin - self.resinit + 1;
        *index.change_value(1)? = 1;
        if self.knots.is_none() {
            if self.resinit <= self.resfin {
                for i in 2..=nincx {
                    *index.change_value(i)? = *index.value(i - 1)? + i;
                }
            }
            return Ok(());
        }
        let mut i2: i32 = 1;
        for band in self.bands()? {
            for i in band.iinit..=band.ifin {
                if i2 != 1 && band.jinit <= i {
                    *index.change_value(i2)? = *index.value(i2 - 1)? + i - band.jinit + 1;
                }
                i2 += 1;
            }
        }
        Ok(())
    }

    // TheA = tA A over the unknowns (lower triangle), accumulated point by point on
    // the non-null span of each row; with `rhs`, also tA B2.
    fn normal_matrix(&self, mut rhs: Option<&mut Matrix<'_>>) -> Result<Matrix<'static>, Failure> {
        let mut the_a: Matrix<'static> =
            Matrix::filled(self.resinit, self.resfin, self.resinit, self.resfin, 0.)
                .map_err(|error| array_failure(error, "AppParCurves_LeastSquare::MakeTAA"))?;
        for k in self.first_p..=self.last_p {
            let indexdeb: i32 = *self.myindex.value(k)? + 1;
            let (jinit, jfin) = (
                self.resinit.max(indexdeb),
                self.resfin.min(indexdeb + self.deg),
            );
            for i in jinit..=jfin {
                let akj: f64 = *self.a.value(k, i)?;
                for j in jinit..=i {
                    the_a[(i, j)] += *self.a.value(k, j)? * akj;
                }
                if let Some(rhs) = rhs.as_deref_mut() {
                    for j in 1..=self.b2.col_number() {
                        *rhs.change_value(i, j)? += akj * *self.b2.value(k, j)?;
                    }
                }
            }
        }
        Ok(the_a)
    }

    // AA(i2) = TheA(i, j) in profile order.
    fn store_profile(&self, the_a: &Matrix<'_>, aa: &mut Vector<'_>) -> Result<(), Failure> {
        let mut i2: i32 = 1;
        for band in self.bands()? {
            for i in band.iinit..=band.ifin {
                for j in band.jinit..=i {
                    *aa.change_value(i2)? = *the_a.value(i, j)?;
                    i2 += 1;
                }
            }
        }
        Ok(())
    }

    /// `MakeTAA(TheA, TheB)` with a matrix right-hand side.
    pub(super) fn make_taa_blocks(
        &self,
        aa: &mut Vector<'_>,
        tab: &mut Matrix<'_>,
    ) -> Result<(), Failure> {
        let the_a: Matrix<'static> = self.normal_matrix(Some(tab))?;
        self.store_profile(&the_a, aa)
    }

    /// `MakeTAA(TheA)`.
    pub(super) fn make_taa_profile(&self, aa: &mut Vector<'_>) -> Result<(), Failure> {
        let the_a: Matrix<'static> = self.normal_matrix(None)?;
        self.store_profile(&the_a, aa)
    }
}

/// The per-coordinate right-hand side and tangency columns of the full system.
struct FullRhs {
    b: Vector<'static>,
    v1: Vector<'static>,
    v2: Vector<'static>,
}

impl LeastSquare {
    // myB, myV1, myV2: one row per (coordinate, point) of the fitted points.
    fn full_rhs(&self) -> Result<FullRhs, Failure> {
        let myfirst: i32 = self.a.lower_row();
        let mylast: i32 = myfirst + self.nlignes - 1;
        let mut rhs: FullRhs = FullRhs {
            b: Vector::filled(myfirst, mylast, 0.)?,
            v1: Vector::filled(myfirst, mylast, 0.)?,
            v2: Vector::filled(myfirst, mylast, 0.)?,
        };
        let neq: i32 = self.last_p - self.first_p + 1;
        let (first, last) = (self.first_constraint, self.last_constraint);
        let (mut xx, mut yy): (f64, f64) = (0., 0.);
        for i in self.first_p..=self.last_p {
            let (ai2, aid) = (*self.a.value(i, 2)?, *self.a.value(i, self.nbpoles - 1)?);
            if first >= Constraint::PassPoint {
                xx = *self.a.value(i, 1)?;
            }
            if first >= Constraint::TangencyPoint {
                xx += ai2;
            }
            if last >= Constraint::PassPoint {
                yy = *self.a.value(i, self.nbpoles)?;
            }
            if last >= Constraint::TangencyPoint {
                yy += aid;
            }
            self.point_rows(
                &mut rhs,
                i,
                [myfirst - self.first_p, neq],
                [ai2, aid, xx, yy],
            )?;
        }
        Ok(rhs)
    }

    // The rows of point i: 3 per 3D point, 2 per 2D point, Neq apart.
    fn point_rows(
        &self,
        rhs: &mut FullRhs,
        i: i32,
        [mut nrow, neq]: [i32; 2],
        [ai2, aid, xx, yy]: [f64; 4],
    ) -> Result<(), Failure> {
        let (first, last) = (
            self.first_constraint >= Constraint::TangencyPoint,
            self.last_constraint >= Constraint::TangencyPoint,
        );
        let mut i2: i32 = 1;
        let widths = std::iter::repeat_n(3, self.nb_p as usize)
            .chain(std::iter::repeat_n(2, self.nb_p2d as usize));
        for width in widths {
            for c in 0..width {
                let row: i32 = i + nrow + c * neq;
                if first {
                    *rhs.v1.change_value(row)? = ai2 * *self.vec1t.value(i2 + c)?;
                }
                if last {
                    *rhs.v2.change_value(row)? = -aid * *self.vec2t.value(i2 + c)?;
                }
                *rhs.b.change_value(row)? = *self.mypoints.value(i, i2 + c)?
                    - xx * *self.mypoints.value(self.myfirstp, i2 + c)?
                    - yy * *self.mypoints.value(self.mylastp, i2 + c)?;
            }
            i2 += width;
            nrow += width * neq;
        }
        Ok(())
    }

    /// `MakeTAA(TheA, TheB)` with a vector right-hand side: the full system of every
    /// coordinate block plus the lambda columns.
    pub(super) fn make_taa_full(
        &self,
        the_a: &mut Vector<'_>,
        tab: &mut Vector<'_>,
    ) -> Result<(), Failure> {
        let both: bool = self.first_constraint >= Constraint::TangencyPoint
            && self.last_constraint >= Constraint::TangencyPoint;
        let ninc1: i32 = if both { self.ninc - 1 } else { self.ninc };
        let rhs: FullRhs = self.full_rhs()?;
        let mut the_v1: Vector<'static> = Vector::filled(1, self.ninc, 0.)?;
        let mut the_v2: Vector<'static> = Vector::filled(1, self.ninc, 0.)?;
        let sums: [f64; 5] = self.full_products(&rhs, [&mut the_v1, &mut the_v2], tab)?;
        self.lambda_entries(sums, ninc1, [&mut the_v1, &mut the_v2], tab)?;
        self.copy_blocks(the_a)?;
        self.lambda_columns(the_a, ninc1, [&the_v1, &the_v2])
    }

    // tA myB into tab, tA myV1 and tA myV2 into TheV1, TheV2, and the lambda sums
    // [taf1, taf2, taf3, tab1, tab2].
    fn full_products(
        &self,
        rhs: &FullRhs,
        [the_v1, the_v2]: [&mut Vector<'_>; 2],
        tab: &mut Vector<'_>,
    ) -> Result<[f64; 5], Failure> {
        let (first, last) = (
            self.first_constraint >= Constraint::TangencyPoint,
            self.last_constraint >= Constraint::TangencyPoint,
        );
        let (neq, nincx) = (
            self.last_p - self.first_p + 1,
            self.resfin - self.resinit + 1,
        );
        let [mut taf1, mut taf2, mut taf3, mut tab1, mut tab2] = [0.; 5];
        let (mut v1, mut v2): (f64, f64) = (0., 0.);
        for k in self.first_p..=self.last_p {
            let indexdeb: i32 = *self.myindex.value(k)? + 1;
            let (jinit, jfin) = (
                self.resinit.max(indexdeb),
                self.resfin.min(indexdeb + self.deg),
            );
            let k1: i32 = k + self.a.lower_row() - self.first_p;
            for i in 0..self.na {
                let nb: i32 = i * neq + k1;
                if first {
                    v1 = *rhs.v1.value(nb)?;
                }
                if last {
                    v2 = *rhs.v2.value(nb)?;
                }
                let b: f64 = *rhs.b.value(nb)?;
                let inc: i32 = i * nincx - self.resinit + 1;
                for j in jinit..=jfin {
                    let akj: f64 = *self.a.value(k, j)?;
                    let u: i32 = j + inc;
                    if first {
                        *the_v1.change_value(u)? += akj * v1;
                    }
                    if last {
                        *the_v2.change_value(u)? += akj * v2;
                    }
                    *tab.change_value(u)? += akj * b;
                }
                if first {
                    taf1 += v1 * v1;
                    tab1 += v1 * b;
                }
                if last {
                    taf2 += v2 * v2;
                    tab2 += v2 * b;
                }
                if first && last {
                    taf3 += v1 * v2;
                }
            }
        }
        Ok([taf1, taf2, taf3, tab1, tab2])
    }

    fn lambda_entries(
        &self,
        [taf1, taf2, taf3, tab1, tab2]: [f64; 5],
        ninc1: i32,
        [the_v1, the_v2]: [&mut Vector<'_>; 2],
        tab: &mut Vector<'_>,
    ) -> Result<(), Failure> {
        let (first, last) = (
            self.first_constraint >= Constraint::TangencyPoint,
            self.last_constraint >= Constraint::TangencyPoint,
        );
        if first {
            *the_v1.change_value(ninc1)? = taf1;
            *tab.change_value(ninc1)? = tab1;
        }
        if last {
            *the_v2.change_value(self.ninc)? = taf2;
            *tab.change_value(self.ninc)? = tab2;
        }
        if first && last {
            *the_v2.change_value(ninc1)? = taf3;
        }
        Ok(())
    }

    // TheA starts with NA copies of the block profile (one per coordinate).
    fn copy_blocks(&self, the_a: &mut Vector<'_>) -> Result<(), Failure> {
        if self.resinit > self.resfin {
            return Ok(());
        }
        let nincx: i32 = self.resfin - self.resinit + 1;
        let mut index: IntegerVector<'static> = IntegerVector::filled(1, nincx, 0)?;
        self.search_index(&mut index)?;
        let mut aa: Vector<'static> = Vector::filled(1, index[nincx], 0.)?;
        self.make_taa_profile(&mut aa)?;
        let mut kk: i32 = 1;
        for _ in 1..=self.na {
            for i in 1..=aa.length() {
                *the_a.change_value(kk)? = aa[i];
                kk += 1;
            }
        }
        Ok(())
    }

    // The last one or two columns of TheA: TheV1 and TheV2.
    fn lambda_columns(
        &self,
        the_a: &mut Vector<'_>,
        ninc1: i32,
        [the_v1, the_v2]: [&Vector<'_>; 2],
    ) -> Result<(), Failure> {
        let (first, last) = (
            self.first_constraint >= Constraint::TangencyPoint,
            self.last_constraint >= Constraint::TangencyPoint,
        );
        let (length, ninc) = (the_a.length(), self.ninc);
        if first && last {
            for j in 1..=ninc1 {
                *the_a.change_value(length - 2 * ninc + j + 1)? = *the_v1.value(j)?;
            }
            for j in 1..=ninc {
                *the_a.change_value(length - ninc + j)? = *the_v2.value(j)?;
            }
        } else if first {
            for j in 1..=ninc {
                *the_a.change_value(length - ninc + j)? = *the_v1.value(j)?;
            }
        } else if last {
            for j in 1..=ninc {
                *the_a.change_value(length - ninc + j)? = *the_v2.value(j)?;
            }
        }
        Ok(())
    }
}
