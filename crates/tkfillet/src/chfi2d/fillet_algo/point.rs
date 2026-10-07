// Rust translation of the FilletPoint class of ChFi2d_FilletAlgo.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FilletPoint`: a point on the first curve with the values and derivatives of the
//! fillet function there (one per projection on the second curve).
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point2;

/// `FilletPoint`: the parameter on the first curve, the parameter of the projection on
/// the second curve, the values and derivatives of the fillet function (sorted by
/// value), the center of the fillet, the validity of each value and the index of its
/// nearest value on the next point. Indices are 1-based as the native sequences;
/// Standard_OutOfRange outside.
///
/// ```
/// use rustycad_tkfillet::chfi2d::FilletPoint;
/// let mut p = FilletPoint::new(0.5);
/// p.append_value(2.0, true);
/// p.append_value(-1.0, false);
/// assert_eq!((p.get_nb_values(), p.get_value(1)?), (2, -1.0));
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug, Default)]
pub struct FilletPoint {
    param: f64,
    param2: f64,
    v: Vec<f64>,
    d: Vec<f64>,
    center: Point2,
    valid: Vec<bool>,
    near: Vec<i32>,
}

/// The 1-based item `index` of a sequence; Standard_OutOfRange outside.
fn item<T: Copy>(seq: &[T], index: i32, what: &str) -> Result<T, Failure> {
    usize::try_from(index - 1)
        .ok()
        .and_then(|i| seq.get(i).copied())
        .ok_or_else(|| {
            Failure::out_of_range(&format!(
                "FilletPoint::{what}({index}): index out of 1..={}",
                seq.len()
            ))
        })
}

/// Removes the 1-based item `index`; Standard_OutOfRange outside.
fn remove_at<T>(seq: &mut Vec<T>, index: i32, what: &str) -> Result<(), Failure> {
    let len = seq.len();
    let i = usize::try_from(index - 1)
        .ok()
        .filter(|i| *i < len)
        .ok_or_else(|| {
            Failure::out_of_range(&format!(
                "FilletPoint::{what}: Remove({index}) out of 1..={len}"
            ))
        })?;
    seq.remove(i);
    Ok(())
}

impl FilletPoint {
    /// `FilletPoint(theParam)`.
    pub fn new(param: f64) -> Self {
        Self {
            param,
            ..Self::default()
        }
    }

    /// `setParam(theParam)`.
    pub fn set_param(&mut self, param: f64) {
        self.param = param;
    }

    /// `getParam()`.
    pub fn get_param(&self) -> f64 {
        self.param
    }

    /// `getNBValues()`.
    pub fn get_nb_values(&self) -> i32 {
        self.v.len() as i32
    }

    /// `getValue(theIndex)`.
    pub fn get_value(&self, index: i32) -> Result<f64, Failure> {
        item(&self.v, index, "getValue")
    }

    /// `getDiff(theIndex)`.
    pub fn get_diff(&self, index: i32) -> Result<f64, Failure> {
        item(&self.d, index, "getDiff")
    }

    /// `isValid(theIndex)`.
    pub fn is_valid(&self, index: i32) -> Result<bool, Failure> {
        item(&self.valid, index, "isValid")
    }

    /// `getNear(theIndex)`.
    pub fn get_near(&self, index: i32) -> Result<i32, Failure> {
        item(&self.near, index, "getNear")
    }

    /// `setParam2(theParam2)`.
    pub fn set_param2(&mut self, param2: f64) {
        self.param2 = param2;
    }

    /// `getParam2()`.
    pub fn get_param2(&self) -> f64 {
        self.param2
    }

    /// `setCenter(thePoint)`.
    pub fn set_center(&mut self, center: Point2) {
        self.center = center;
    }

    /// `getCenter()`.
    pub fn get_center(&self) -> Point2 {
        self.center
    }

    /// `appendValue(theValue, theValid)`: inserted before the first larger value.
    pub fn append_value(&mut self, value: f64, valid: bool) {
        let at = self
            .v
            .iter()
            .position(|v| value < *v)
            .unwrap_or(self.v.len());
        self.v.insert(at, value);
        self.valid.insert(at, valid);
    }

    /// `calculateDiff(thePoint)`: the derivatives as differences with `point`; true when
    /// both points have as many values (else the nearest values are used, a set
    /// derivative replaced only by a smaller one).
    pub fn calculate_diff(&mut self, point: &FilletPoint) -> Result<bool, Failure> {
        let diffs_set = !self.d.is_empty();
        let dx = point.param - self.param;
        if point.v.len() == self.v.len() {
            for a in 1..=self.get_nb_values() {
                let dy = item(&point.v, a, "calculateDiff")? - item(&self.v, a, "calculateDiff")?;
                self.store_diff(a, dy / dx, diffs_set, false)?;
            }
            return Ok(true);
        }
        // between the different points searching for nearest analogs
        let mut dy: f64 = 0.0;
        for a in 1..=self.get_nb_values() {
            let va = item(&self.v, a, "calculateDiff")?;
            for (b, vb) in (1..).zip(&point.v) {
                if b == 1 || (vb - va).abs() < dy.abs() {
                    dy = vb - va;
                }
            }
            self.store_diff(a, dy / dx, diffs_set, true)?;
        }
        Ok(false)
    }

    /// Sets the derivative `a` (appends it when none are set); with `smaller`, a set
    /// derivative is replaced only by a smaller one in magnitude.
    fn store_diff(
        &mut self,
        a: i32,
        diff: f64,
        diffs_set: bool,
        smaller: bool,
    ) -> Result<(), Failure> {
        if !diffs_set {
            self.d.push(diff);
            return Ok(());
        }
        let old = item(&self.d, a, "calculateDiff")?;
        if !smaller || diff.abs() < old.abs() {
            self.d[(a - 1) as usize] = diff;
        }
        Ok(())
    }

    /// `Copy()`: a copy of the parameter, the values, the derivatives and the validity
    /// only (not the nearest indices, the second parameter nor the center).
    pub fn copy(&self) -> FilletPoint {
        FilletPoint {
            param: self.param,
            v: self.v.clone(),
            d: self.d.clone(),
            valid: self.valid.clone(),
            ..FilletPoint::default()
        }
    }

    /// `hasSolution(theRadius)`: the index of the first value that is a solution (zero
    /// within Precision::Confusion as a distance), 0 when none.
    pub fn has_solution(&self, radius: f64) -> i32 {
        let is_root = |v: &f64| {
            ((v.abs() + radius * radius).abs().sqrt() - radius).abs() < precision::confusion()
        };
        self.v.iter().position(is_root).map_or(0, |i| i as i32 + 1)
    }

    /// `LowerValue()` (for debug only): the value of smallest magnitude, the last one on
    /// ties; NaN without values (an uninitialized double natively).
    pub fn lower_value(&self) -> f64 {
        let mut best: Option<f64> = None;
        for v in self.v.iter().rev() {
            if best.is_none_or(|b| b.abs() > v.abs()) {
                best = Some(*v);
            }
        }
        best.unwrap_or(f64::NAN)
    }

    /// `remove(theIndex)`: the value, its derivative, validity and nearest index.
    pub fn remove(&mut self, index: i32) -> Result<(), Failure> {
        remove_at(&mut self.v, index, "remove")?;
        remove_at(&mut self.d, index, "remove")?;
        remove_at(&mut self.valid, index, "remove")?;
        remove_at(&mut self.near, index, "remove")
    }

    /// Removes the value `index` with its derivative and validity.
    fn remove_value(&mut self, index: i32) -> Result<(), Failure> {
        remove_at(&mut self.v, index, "FilterPoints")?;
        remove_at(&mut self.d, index, "FilterPoints")?;
        remove_at(&mut self.valid, index, "FilterPoints")
    }
}

impl FilletPoint {
    /// `FilterPoints(thePoint)`: keeps the values that have a near value on the next
    /// point `point` (hypothesis of constant first and second derivatives), each near
    /// value kept once (for the nearest value).
    pub fn filter_points(&mut self, point: &FilletPoint) -> Result<(), Failure> {
        let dx = point.param - self.param;
        let mut diffs: Vec<f64> = Vec::new();
        let mut a = 1;
        while a <= self.get_nb_values() {
            let (near, diff) = self.near_value(point, a, dx)?;
            if near == 0 {
                // there is no near: remove it from the list
                self.remove_value(a)?;
                a -= 1;
            } else {
                a = self.keep_near(near, diff, a, &mut diffs)?;
            }
            a += 1;
        }
        Ok(())
    }

    /// The near value of the value `a` on `point` and the difference to it; 0 when the
    /// signs, the derivatives or the difference reject it.
    fn near_value(&self, point: &FilletPoint, a: i32, dx: f64) -> Result<(i32, f64), Failure> {
        let (va, da) = (
            item(&self.v, a, "FilterPoints")?,
            item(&self.d, a, "FilterPoints")?,
        );
        let (mut near, mut diff) = (0, dx * 10000.0);
        let y = va + da * dx;
        for b in 1..=point.get_nb_values() {
            // calculate hypothesis value of the Y2 with the constant first and second derivative
            let y2 = y + dx * (item(&point.d, b, "FilterPoints")? - da) / 2.0;
            let vb = item(&point.v, b, "FilterPoints")?;
            if near == 0 || (y2 - vb).abs() < diff.abs() {
                (near, diff) = (b, y2 - vb);
            }
        }
        if near != 0 && rejected(point, (va, da), near, dx)? {
            near = 0;
        }
        if near != 0 && (diff / dx).abs() > 1.0e+7 {
            near = 0;
        }
        Ok((near, diff))
    }

    /// The value `a` keeps its near value `near` unless an earlier value already has it
    /// with a smaller difference (then `a` goes); the earlier one goes otherwise and `a`
    /// takes its near value. The new loop index.
    fn keep_near(
        &mut self,
        near: i32,
        diff: f64,
        a: i32,
        diffs: &mut Vec<f64>,
    ) -> Result<i32, Failure> {
        let found_at = self.near.iter().position(|n| *n == near);
        let Some(b) = found_at else {
            self.near.push(near);
            diffs.push(diff);
            return Ok(a);
        };
        let b1 = b as i32 + 1;
        if diffs[b].abs() < diff.abs() {
            // return this 'near'
            self.remove_value(a)?;
            return Ok(a - 1);
        }
        // remove the old 'near' (this value takes it)
        self.remove_value(b1)?;
        self.near.remove(b);
        diffs.remove(b);
        self.near.push(near);
        diffs.push(diff);
        Ok(a - 1)
    }
}

/// Whether the near value `near` of a value `(va, da)` is rejected: the same sign at
/// both ends of the interval with a derivative leading away, or both ends growing.
fn rejected(
    point: &FilletPoint,
    (va, da): (f64, f64),
    near: i32,
    dx: f64,
) -> Result<bool, Failure> {
    let (vn, dn) = (
        item(&point.v, near, "FilterPoints")?,
        item(&point.d, near, "FilterPoints")?,
    );
    let conf = precision::confusion();
    if va * vn > 0.0 {
        // the same sign at the same sides of the interval
        if va * da > 0.0 {
            if da.abs() > conf {
                return Ok(true);
            }
        } else if va.abs() > vn.abs() && vn * dn < 0.0 && dn.abs() > conf {
            return Ok(true);
        }
    }
    let both_grow = (va + da * dx) * va > conf && (vn + dn * dx) * vn > conf;
    Ok(va * vn > 0.0 && both_grow)
}
