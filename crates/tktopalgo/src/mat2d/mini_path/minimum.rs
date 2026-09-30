// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT2d/MAT2d_MiniPath.cxx (MinimumL1L2 and the table of connexions of Perform) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The shortest connexion between two lines of a figure, by the extrema of their
//! items (points and curves).
use crate::mat2d::connexion::Mat2dConnexion;
use crate::mat2d::{Mat2dFigure, Mat2dLine, seq};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dCartesianPoint, Geom2dCurve, Geom2dGeometry, Geom2dPoint, errors,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeombase::extrema::{ExtCC2d, ExtPC2d};
use rustycad_tkmath::gp::Point2;

/// The table `Connexion(i, j)` of Perform: the shortest connexion from the line `i`
/// to the line `j` (the reverse of `(j, i)` below the diagonal).
pub(super) struct ConnexionTable {
    size: i32,
    cells: Vec<Handle<Mat2dConnexion>>,
}

impl ConnexionTable {
    pub(super) fn get(&self, i: i32, j: i32) -> Result<Handle<Mat2dConnexion>, Failure> {
        if i < 1 || j < 1 || i > self.size || j > self.size {
            return Err(errors::out_of_range(&format!(
                "MAT2d_MiniPath: connexion ({i}, {j}); expected 1..={} for both lines",
                self.size
            )));
        }
        let cell: &Handle<Mat2dConnexion> = &self.cells[((i - 1) * self.size + (j - 1)) as usize];
        if cell.is_null() {
            return Err(errors::null_object(&format!(
                "MAT2d_MiniPath: no connexion ({i}, {j}); expected two lines"
            )));
        }
        Ok(cell.clone())
    }
    fn set(&mut self, i: i32, j: i32, c: Handle<Mat2dConnexion>) {
        self.cells[((i - 1) * self.size + (j - 1)) as usize] = c;
    }
}

/// The connexions of every pair of lines.
pub(super) fn all_connexions(
    figure: &Mat2dFigure,
    nb_lines: i32,
) -> Result<ConnexionTable, Failure> {
    let size: i32 = nb_lines.max(0);
    let mut table = ConnexionTable {
        size,
        cells: vec![Handle::default(); (size * size) as usize],
    };
    for i in 1..nb_lines {
        for j in i + 1..=nb_lines {
            let c: Handle<Mat2dConnexion> = Handle::new(minimum_l1_l2(figure, i, j)?);
            table.set(j, i, c.reverse());
            table.set(i, j, c);
        }
    }
    Ok(table)
}

/// An item of a line: a point or a curve.
enum Item {
    Point(Point2),
    Curve(Handle<dyn Geom2dCurve>),
}

fn item(geometry: &Handle<dyn Geom2dGeometry>) -> Result<Item, Failure> {
    if geometry.cast::<Geom2dCartesianPoint>().is_some() {
        let point: Handle<dyn Geom2dPoint> = geometry.cast();
        return Ok(Item::Point(point.pnt2d()));
    }
    Ok(Item::Curve(geometry.cast()))
}

/// The best extremum so far: the square distance, the items and the parameters and
/// points on both lines.
struct Best {
    dist_2: f64,
    items: (i32, i32),
    params: (f64, f64),
    points: (Point2, Point2),
}

/// `MinimumL1L2(Figure, L1, L2)`: the connexion that realizes the minimum distance
/// between the lines `l1` and `l2`, from `l1` to `l2`.
fn minimum_l1_l2(figure: &Mat2dFigure, il1: i32, il2: i32) -> Result<Mat2dConnexion, Failure> {
    let l1: Mat2dLine = seq::at(figure, il1)?;
    let l2: Mat2dLine = seq::at(figure, il2)?;
    let mut best = Best {
        dist_2: f64::MAX,
        items: (0, 0),
        params: (0., 0.),
        points: (Point2::default(), Point2::default()),
    };
    for ic1 in 1..=l1.length() {
        let item1: Item = item(&seq::at(&l1, ic1)?)?;
        for ic2 in 1..=l2.length() {
            let item2: Item = item(&seq::at(&l2, ic2)?)?;
            best.compare(&item1, &item2, (ic1, ic2))?;
        }
    }
    let (p1, p2) = best.points;
    Ok(Mat2dConnexion::with_values(
        il1,
        il2,
        best.items.0,
        best.items.1,
        best.dist_2.sqrt(),
        best.params.0,
        best.params.1,
        p1,
        p2,
    ))
}

impl Best {
    // The extrema of one pair of items.
    fn compare(&mut self, item1: &Item, item2: &Item, items: (i32, i32)) -> Result<(), Failure> {
        match (item1, item2) {
            (Item::Point(p1), Item::Point(p2)) => {
                let dist_2: f64 = p1.square_distance(*p2);
                if dist_2 <= self.dist_2 {
                    *self = Best {
                        dist_2,
                        items,
                        params: (0., 0.),
                        points: (*p1, *p2),
                    };
                }
                Ok(())
            }
            (Item::Point(p1), Item::Curve(c2)) => self.point_curve(*p1, c2, items, false),
            (Item::Curve(c1), Item::Point(p2)) => self.point_curve(*p2, c1, items, true),
            (Item::Curve(c1), Item::Curve(c2)) => self.curves(c1, c2, items),
        }
    }
    // The extrema of a point and a curve; `curve_first` when the curve is on the
    // first line.
    fn point_curve(
        &mut self,
        p: Point2,
        c: &Handle<dyn Geom2dCurve>,
        items: (i32, i32),
        curve_first: bool,
    ) -> Result<(), Failure> {
        let adaptor = Geom2dAdaptorCurve::with_curve(c)?;
        let extremas = ExtPC2d::with_curve(p, &adaptor, 1.0e-10)?;
        if !extremas.is_done() {
            return Ok(());
        }
        for i in 1..=extremas.nb_ext()? {
            let dist_2: f64 = extremas.square_distance(i)?;
            if dist_2 < self.dist_2 {
                let on_curve = extremas.point(i)?;
                let (u, q) = (on_curve.parameter(), *on_curve.value());
                let (params, points) = if curve_first {
                    ((u, 0.), (q, p))
                } else {
                    ((0., u), (p, q))
                };
                *self = Best {
                    dist_2,
                    items,
                    params,
                    points,
                };
            }
        }
        Ok(())
    }
    // The extrema of two curves, unless they are parallel.
    fn curves(
        &mut self,
        c1: &Handle<dyn Geom2dCurve>,
        c2: &Handle<dyn Geom2dCurve>,
        items: (i32, i32),
    ) -> Result<(), Failure> {
        let a1 = Geom2dAdaptorCurve::with_curve(c1)?;
        let a2 = Geom2dAdaptorCurve::with_curve(c2)?;
        let extremas = ExtCC2d::with_curves(&a1, &a2, 1.0e-10, 1.0e-10)?;
        if extremas.is_parallel()? || !extremas.is_done() {
            return Ok(());
        }
        for i in 1..=extremas.nb_ext()? {
            let dist_2: f64 = extremas.square_distance(i)?;
            if dist_2 < self.dist_2 {
                let (on1, on2) = extremas.points(i)?;
                let params = (on1.parameter(), on2.parameter());
                *self = Best {
                    dist_2,
                    items,
                    params,
                    points: (*on1.value(), *on2.value()),
                };
            }
        }
        Ok(())
    }
}
