// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert_CompBezierSurfacesToBSplineSurface.cxx
// (Perform) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The private `Perform` of `GeomConvert_CompBezierSurfacesToBSplineSurface`: the
//! pole grid of the C0 B-spline that joins a grid of Bezier patches.
use crate::array_access::{at, grid, put};
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::geom::GeomBezierSurface;
use rustycad_tkmath::gp::{Point3, Xyz};

/// The C0 B-spline data Perform computes from the patches and the knot counts.
pub(super) struct JoinedPoles {
    pub poles: Array2<'static, Point3>,
    pub u_mults: Array1<'static, i32>,
    pub v_mults: Array1<'static, i32>,
    pub u_degree: i32,
    pub v_degree: i32,
}

/// `Perform(Beziers)`: the highest degrees, the degree elevation of every patch (the
/// caller's patches are modified in place, as natively), the shared poles summed and
/// halved, and multiplicities Degree inside and Degree + 1 at the ends.
///
/// The patch indices are taken from the array bounds but the pole offsets assume
/// bounds starting at 1, as natively. Standard_NotImplemented ("CompBezierSurfacesToBSpl
/// : rational !") when a patch is rational.
pub(super) fn perform(
    beziers: &Array2<'_, Handle<GeomBezierSurface>>,
    nb_u_knots: i32,
    nb_v_knots: i32,
) -> Result<JoinedPoles, Failure> {
    let (u_degree, v_degree) = degrees(beziers)?;
    let rows: i32 = (u_degree + 1) * beziers.col_length() - nb_u_knots + 2;
    let cols: i32 = (v_degree + 1) * beziers.row_length() - nb_v_knots + 2;
    let mut poles: Array2<'static, Point3> = grid(rows, cols)?;
    for iu in beziers.lower_row()..=beziers.upper_row() {
        for iv in beziers.lower_col()..=beziers.upper_col() {
            let patch: Handle<GeomBezierSurface> = at(beziers, iu, iv, "Beziers")?;
            let patch: &GeomBezierSurface = live(&patch)?;
            // (2.1) Increase the degree.
            patch.increase(u_degree, v_degree)?;
            let placement = PatchPlacement {
                iu,
                iv,
                u_degree,
                v_degree,
            };
            placement.copy(patch, &mut poles)?;
            placement.sum_shared(patch, &mut poles)?;
        }
    }
    halve_shared(&mut poles, [u_degree, v_degree], [nb_u_knots, nb_v_knots])?;
    Ok(JoinedPoles {
        poles,
        u_mults: multiplicities(nb_u_knots, u_degree)?,
        v_mults: multiplicities(nb_v_knots, v_degree)?,
        u_degree,
        v_degree,
    })
}

// (1) The degrees (at least 1) and the rational test.
fn degrees(beziers: &Array2<'_, Handle<GeomBezierSurface>>) -> Result<(i32, i32), Failure> {
    let (mut isrational, mut u_degree, mut v_degree) = (false, 1, 1);
    for iu in beziers.lower_row()..=beziers.upper_row() {
        for iv in beziers.lower_col()..=beziers.upper_col() {
            let patch: Handle<GeomBezierSurface> = at(beziers, iu, iv, "Beziers")?;
            let patch: &GeomBezierSurface = live(&patch)?;
            if patch.is_u_rational() || patch.is_v_rational() {
                isrational = true;
            }
            u_degree = u_degree.max(patch.u_degree());
            v_degree = v_degree.max(patch.v_degree());
        }
    }
    if isrational {
        return Err(failure(
            "Standard_NotImplemented",
            "CompBezierSurfacesToBSpl : rational !",
        ));
    }
    Ok((u_degree, v_degree))
}

// Where patch (iu, iv) goes in the pole grid: its first pole at
// ((iu - 1) UDegree + 1, (iv - 1) VDegree + 1).
struct PatchPlacement {
    iu: i32,
    iv: i32,
    u_degree: i32,
    v_degree: i32,
}

impl PatchPlacement {
    fn u_index(&self) -> i32 {
        (self.iu - 1) * self.u_degree + 1
    }

    fn v_index(&self) -> i32 {
        (self.iv - 1) * self.v_degree + 1
    }

    // The first pole row (column) is shared with the previous patch, except for the
    // first patch of the direction.
    fn starts(&self) -> (i32, i32) {
        (
            if self.iu == 1 { 1 } else { 2 },
            if self.iv == 1 { 1 } else { 2 },
        )
    }

    // (2.2) The poles to copy.
    fn copy(
        &self,
        patch: &GeomBezierSurface,
        poles: &mut Array2<'static, Point3>,
    ) -> Result<(), Failure> {
        let (udeb, vdeb) = self.starts();
        for (upol, uindex) in (udeb..=self.u_degree + 1).zip(self.u_index() + udeb - 1..) {
            for (vpol, vindex) in (vdeb..=self.v_degree + 1).zip(self.v_index() + vdeb - 1..) {
                put(poles, uindex, vindex, patch.pole(upol, vpol)?, "Poles")?;
            }
        }
        Ok(())
    }

    // (2.3) The shared poles to sum: the first column, the first row, and the corner.
    fn sum_shared(
        &self,
        patch: &GeomBezierSurface,
        poles: &mut Array2<'static, Point3>,
    ) -> Result<(), Failure> {
        let (udeb, vdeb) = self.starts();
        let (u_index, v_index) = (self.u_index(), self.v_index());
        if udeb == 2 {
            for (vpol, vindex) in (vdeb..=self.v_degree + 1).zip(v_index + vdeb - 1..) {
                add_coord(poles, u_index, vindex, patch.pole(1, vpol)?.xyz())?;
            }
        }
        if vdeb == 2 {
            for (upol, uindex) in (udeb..=self.u_degree + 1).zip(u_index + udeb - 1..) {
                add_coord(poles, uindex, v_index, patch.pole(upol, 1)?.xyz())?;
            }
        }
        if udeb == 2 && vdeb == 2 {
            add_coord(poles, u_index, v_index, patch.pole(1, 1)?.xyz())?;
        }
        Ok(())
    }
}

// Poles(u, v).ChangeCoord() += coord.
fn add_coord(
    poles: &mut Array2<'static, Point3>,
    u: i32,
    v: i32,
    coord: Xyz,
) -> Result<(), Failure> {
    let mut pole: Point3 = at(poles, u, v, "Poles")?;
    pole.set_xyz(pole.xyz() + coord);
    put(poles, u, v, pole, "Poles")
}

// (3) Elimination of redundancies: the previous loop counts the poles of simple knots
// twice (and those of double knots, in U and V, four times), so every inner knot
// row, then every inner knot column, is halved.
fn halve_shared(
    poles: &mut Array2<'static, Point3>,
    [u_degree, v_degree]: [i32; 2],
    [nb_u_knots, nb_v_knots]: [i32; 2],
) -> Result<(), Failure> {
    let (upper_row, upper_col) = (poles.upper_row(), poles.upper_col());
    for (_, u_index) in (2..nb_u_knots).zip((u_degree + 1..).step_by(u_degree.max(1) as usize)) {
        for vpol in 1..=upper_col {
            scale_pole(poles, u_index, vpol)?;
        }
    }
    for (_, v_index) in (2..nb_v_knots).zip((v_degree + 1..).step_by(v_degree.max(1) as usize)) {
        for upol in 1..=upper_row {
            scale_pole(poles, upol, v_index)?;
        }
    }
    Ok(())
}

fn scale_pole(poles: &mut Array2<'static, Point3>, u: i32, v: i32) -> Result<(), Failure> {
    let mut pole: Point3 = at(poles, u, v, "Poles")?;
    pole.set_xyz(pole.xyz() * 0.5);
    put(poles, u, v, pole, "Poles")
}

// (4) Degree at the inner knots, Degree + 1 at the ends.
fn multiplicities(nb_knots: i32, degree: i32) -> Result<Array1<'static, i32>, Failure> {
    let mut mults: Array1<'static, i32> = one_based(nb_knots)?;
    mults.init(degree);
    mults
        .set_value(1, degree + 1)
        .map_err(|e| crate::native_failure::array_failure(e, "Mults"))?;
    mults
        .set_value(nb_knots, degree + 1)
        .map_err(|e| crate::native_failure::array_failure(e, "Mults"))?;
    Ok(mults)
}

fn live(patch: &Handle<GeomBezierSurface>) -> Result<&GeomBezierSurface, Failure> {
    patch.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "GeomConvert_CompBezierSurfacesToBSplineSurface: null patch; expected a Bezier surface",
        )
    })
}
