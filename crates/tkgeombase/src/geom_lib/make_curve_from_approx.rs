// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib_MakeCurvefromApprox.hxx/.cxx/.lxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomLib_MakeCurvefromApprox`: the B-spline curves of an AdvApprox approximation.
use crate::geom_convert::missing_table;
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::adv_approx::AdvApproxApproxAFunction;
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkmath::gp::{Point2, Point3};

/// The Geom and Geom2d B-splines of the subspaces of an approximation, polynomial
/// or rational (a 1D subspace as the weights).
///
/// The native class copies the approximation; the port borrows it, since the
/// approximation is not modified.
///
/// `let maker = MakeCurvefromApprox::new(&approx); let curve = maker.curve(1)?;`
pub struct MakeCurvefromApprox<'a> {
    my_approx: &'a AdvApproxApproxAFunction,
}

impl<'a> MakeCurvefromApprox<'a> {
    /// The maker of the curves of `approx` (`GeomLib_MakeCurvefromApprox(Approx)`);
    /// `MakeCurvefromApprox::new(&approx)`.
    pub fn new(approx: &'a AdvApproxApproxAFunction) -> Self {
        Self { my_approx: approx }
    }

    /// Whether the approximation is done (`IsDone()`); `maker.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_approx.is_done()
    }

    /// The number of 1D subspaces (`Nb1DSpaces()`); `maker.nb_1d_spaces()?`.
    pub fn nb_1d_spaces(&self) -> Result<i32, Failure> {
        self.my_approx.num_sub_spaces(1)
    }

    /// The number of 2D subspaces (`Nb2DSpaces()`); `maker.nb_2d_spaces()?`.
    pub fn nb_2d_spaces(&self) -> Result<i32, Failure> {
        self.my_approx.num_sub_spaces(2)
    }

    /// The number of 3D subspaces (`Nb3DSpaces()`); `maker.nb_3d_spaces()?`.
    pub fn nb_3d_spaces(&self) -> Result<i32, Failure> {
        self.my_approx.num_sub_spaces(3)
    }

    /// The polynomial 2D B-spline of 2D subspace `index2d`; Standard_OutOfRange for an
    /// index outside `0..=Nb2DSpaces`, StdFail_NotDone when the approximation is not
    /// done (`Curve2d(Index2d)`); `maker.curve2d(1)?`.
    pub fn curve2d(&self, index2d: i32) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
        check_index("Curve2d", &[(index2d, self.nb_2d_spaces()?)])?;
        self.check_done(!self.is_done(), "Curve2d")?;
        let mut poles: Array1<'static, Point2> = one_based(self.my_approx.nb_poles()?)?;
        self.my_approx.poles_2d_of(index2d, &mut poles)?;
        self.curve2d_from_poles(&poles)
    }

    /// The rational 2D B-spline of 2D subspace `index2d` weighted by 1D subspace
    /// `index1d` (the poles divided by the weights; the curve itself is built without
    /// weights, as natively); StdFail_NotDone when the approximation is neither done
    /// nor has a result (`Curve2d(Index1d, Index2d)`); `maker.curve2d_weighted(1, 1)?`.
    pub fn curve2d_weighted(
        &self,
        index1d: i32,
        index2d: i32,
    ) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
        check_index(
            "Curve2d",
            &[
                (index1d, self.nb_1d_spaces()?),
                (index2d, self.nb_2d_spaces()?),
            ],
        )?;
        self.check_done(!self.is_done() && !self.my_approx.has_result(), "Curve2d")?;
        let mut poles: Array1<'static, Point2> = one_based(self.my_approx.nb_poles()?)?;
        let weights: Array1<'static, f64> = self.poles_1d(index1d)?;
        self.my_approx.poles_2d_of(index2d, &mut poles)?;
        for i in 1..=self.my_approx.nb_poles()? {
            let w: f64 = weights[i];
            let (x, y): (f64, f64) = (poles[i].x(), poles[i].y());
            poles[i].set_coordinates(x / w, y / w);
        }
        self.curve2d_from_poles(&poles)
    }

    /// The 2D B-spline whose X and Y are the 1D subspaces `index1d` and `index2d`;
    /// both indices are checked against `Nb1DSpaces`, as natively
    /// (`Curve2dFromTwo1d(Index1d, Index2d)`); `maker.curve2d_from_two_1d(1, 2)?`.
    pub fn curve2d_from_two_1d(
        &self,
        index1d: i32,
        index2d: i32,
    ) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
        let nb_1d: i32 = self.nb_1d_spaces()?;
        check_index("Curve2d", &[(index1d, nb_1d), (index2d, nb_1d)])?;
        self.check_done(!self.is_done() && !self.my_approx.has_result(), "Curve2d")?;
        let poles1d2: Array1<'static, f64> = self.poles_1d(index2d)?;
        let poles1d1: Array1<'static, f64> = self.poles_1d(index1d)?;
        let mut poles: Array1<'static, Point2> = one_based(self.my_approx.nb_poles()?)?;
        for i in 1..=self.my_approx.nb_poles()? {
            poles[i] = Point2::new(poles1d1[i], poles1d2[i]);
        }
        self.curve2d_from_poles(&poles)
    }

    /// The polynomial 3D B-spline of 3D subspace `index3d`; StdFail_NotDone when the
    /// approximation is neither done nor has a result (`Curve(Index3d)`);
    /// `maker.curve(1)?`.
    pub fn curve(&self, index3d: i32) -> Result<Handle<GeomBSplineCurve>, Failure> {
        check_index("Curve", &[(index3d, self.nb_3d_spaces()?)])?;
        self.check_done(!self.is_done() && !self.my_approx.has_result(), "Curve")?;
        let mut poles: Array1<'static, Point3> = one_based(self.my_approx.nb_poles()?)?;
        self.my_approx.poles_of(index3d, &mut poles)?;
        self.curve_from_poles(&poles)
    }

    /// The 3D B-spline of 3D subspace `index3d` with its poles divided by the 1D
    /// subspace `index1d` (built without weights, as natively); StdFail_NotDone when
    /// the approximation is not done (`Curve(Index1D, Index3D)`);
    /// `maker.curve_weighted(1, 1)?`.
    pub fn curve_weighted(
        &self,
        index1d: i32,
        index3d: i32,
    ) -> Result<Handle<GeomBSplineCurve>, Failure> {
        check_index(
            "Curve3d",
            &[
                (index1d, self.nb_1d_spaces()?),
                (index3d, self.nb_3d_spaces()?),
            ],
        )?;
        self.check_done(!self.is_done(), "Curve3d")?;
        let mut poles: Array1<'static, Point3> = one_based(self.my_approx.nb_poles()?)?;
        let weights: Array1<'static, f64> = self.poles_1d(index1d)?;
        self.my_approx.poles_of(index3d, &mut poles)?;
        for i in 1..=self.my_approx.nb_poles()? {
            let w: f64 = weights[i];
            let (x, y, z): (f64, f64, f64) = (poles[i].x(), poles[i].y(), poles[i].z());
            poles[i] = Point3::new(x / w, y / w, z / w);
        }
        self.curve_from_poles(&poles)
    }

    fn check_done(&self, not_done: bool, member: &str) -> Result<(), Failure> {
        if not_done {
            return Err(failure(
                "StdFail_NotDone",
                &format!(" GeomLib_MakeCurvefromApprox : {member}"),
            ));
        }
        Ok(())
    }

    fn poles_1d(&self, index: i32) -> Result<Array1<'static, f64>, Failure> {
        let mut values: Array1<'static, f64> = one_based(self.my_approx.nb_poles()?)?;
        self.my_approx.poles_1d_of(index, &mut values)?;
        Ok(values)
    }

    // The shared knots and multiplicities of the approximation.
    fn knots(&self) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
        let knots = self
            .my_approx
            .knots()
            .ok_or_else(|| missing_table("Knots"))?
            .clone();
        let mults = self
            .my_approx
            .multiplicities()
            .ok_or_else(|| missing_table("Multiplicities"))?
            .clone();
        Ok((knots, mults))
    }

    fn curve2d_from_poles(
        &self,
        poles: &Array1<'static, Point2>,
    ) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
        let (knots, mults) = self.knots()?;
        Ok(Handle::new(Geom2dBSplineCurve::new(
            poles,
            &knots,
            &mults,
            self.my_approx.degree(),
            false,
        )?))
    }

    fn curve_from_poles(
        &self,
        poles: &Array1<'static, Point3>,
    ) -> Result<Handle<GeomBSplineCurve>, Failure> {
        let (knots, mults) = self.knots()?;
        Ok(Handle::new(GeomBSplineCurve::new(
            poles,
            &knots,
            &mults,
            self.my_approx.degree(),
            false,
        )?))
    }
}

// Standard_OutOfRange_Raise_if(Index < 0 || Index > NbSpaces) for each pair.
fn check_index(member: &str, indices: &[(i32, i32)]) -> Result<(), Failure> {
    for &(index, count) in indices {
        if index < 0 || index > count {
            return Err(failure(
                "Standard_OutOfRange",
                &format!(
                    " GeomLib_MakeCurvefromApprox : {member}: index {index}; expected 0..={count}"
                ),
            ));
        }
    }
    Ok(())
}
