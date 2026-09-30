// Rust translation of ModelingAlgorithms/TKGeomAlgo/Plate/Plate_Plate.hxx/.lxx/.cxx (the class,
// its loading, copies and evaluation) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Plate_Plate`: the variational plate interpolating pinpoint and linear constraints.
//! `SolveTI` is in `solve`, the kernels in `sol_em`.
use super::sol_em::SolEmCache;
use super::{
    PlateFreeGtoCConstraint, PlateGlobalTranslationConstraint, PlateGtoCConstraint,
    PlateLineConstraint, PlateLinearScalarConstraint, PlateLinearXYZConstraint,
    PlatePinpointConstraint, PlatePlaneConstraint, PlateSampledCurveConstraint,
};
use rustycad_tkernel::ncollection::{HArray2, Sequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::gp::{Xy, Xyz};
use std::sync::Mutex;

/// A plate surface `Xyz(u, v)`: the minimizer of the iterated Laplacian energy of order
/// `ord` under pinpoint, linear XYZ and linear scalar constraints (`Plate_Plate`).
///
/// The solution arrays (`solution`, `points`, `deru`, `derv`) are `None` where the
/// native pointers are null. `SolEm` caches its last point in a lock, which the native
/// code updates through a `const_cast`.
/// `let mut plate = PlatePlate::new(); plate.load_pinpoint(&c); plate.solve_ti(2, 1.0, &range)?;`
pub struct PlatePlate {
    pub(super) order: i32,
    pub(super) n_el: i32,
    pub(super) n_dim: i32,
    pub(super) solution: Option<Vec<Xyz>>,
    pub(super) points: Option<Vec<Xy>>,
    pub(super) deru: Option<Vec<i32>>,
    pub(super) derv: Option<Vec<i32>>,
    pub(super) ok: bool,
    pub(super) my_constraints: Sequence<PlatePinpointConstraint>,
    pub(super) my_lxyz_constraints: Sequence<PlateLinearXYZConstraint>,
    pub(super) my_lscalar_constraints: Sequence<PlateLinearScalarConstraint>,
    pub(super) ddu: [f64; 10],
    pub(super) ddv: [f64; 10],
    pub(super) max_constraint_order: i32,
    pub(super) polynomial_part_only: bool,
    pub(super) sol_em_cache: Mutex<SolEmCache>,
}

impl Default for PlatePlate {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for PlatePlate {
    /// The copy constructor (`Plate_Plate(const Plate_Plate&)`): the solution arrays are
    /// copied only from a solved plate, and the `SolEm` cache starts afresh.
    fn clone(&self) -> Self {
        let mut copy = Self::new();
        copy.ok = self.ok;
        copy.assign_from(self);
        copy
    }
}

/// The largest `Idu + Idv` of a list of pinpoint constraints.
fn max_order<'a>(
    constraints: impl Iterator<Item = &'a PlatePinpointConstraint>,
    start: i32,
) -> i32 {
    constraints.fold(start, |max, c| max.max(c.idu() + c.idv()))
}

impl PlatePlate {
    /// An empty plate (`Plate_Plate()`): not done, no constraint. `PlatePlate::new()`.
    pub fn new() -> Self {
        Self {
            order: 0,
            n_el: 0,
            n_dim: 0,
            solution: None,
            points: None,
            deru: None,
            derv: None,
            ok: false,
            my_constraints: Sequence::default(),
            my_lxyz_constraints: Sequence::default(),
            my_lscalar_constraints: Sequence::default(),
            ddu: [0.0; 10],
            ddv: [0.0; 10],
            max_constraint_order: 0,
            polynomial_part_only: false,
            sol_em_cache: Mutex::new(SolEmCache::default()),
        }
    }

    /// The fields that the copy constructor and `Copy` both take from `reference`.
    fn assign_from(&mut self, reference: &Self) {
        self.order = reference.order;
        self.n_el = reference.n_el;
        self.n_dim = reference.n_dim;
        self.ok = reference.ok;
        if reference.ok {
            if self.n_dim > 0 {
                self.solution = reference.solution.clone();
            }
            if self.n_el > 0 {
                self.points = reference.points.clone();
                self.deru = reference.deru.clone();
                self.derv = reference.derv.clone();
            }
        }
        self.my_constraints = reference.my_constraints.clone();
        self.my_lxyz_constraints = reference.my_lxyz_constraints.clone();
        self.my_lscalar_constraints = reference.my_lscalar_constraints.clone();
        self.max_constraint_order = reference.max_constraint_order;
        self.polynomial_part_only = reference.polynomial_part_only;
        self.ddu = reference.ddu;
        self.ddv = reference.ddv;
    }

    /// Becomes a copy of `reference` (`Copy`, and `operator=`): `Init`, then the copy
    /// constructor's fields; the `SolEm` cache is kept, as natively. `plate.copy_from(&other)`.
    pub fn copy_from(&mut self, reference: &Self) -> &mut Self {
        self.init();
        self.assign_from(reference);
        self
    }

    /// Adds a pinpoint constraint (`Load(Plate_PinpointConstraint)`); the plate is no
    /// longer done. `plate.load_pinpoint(&c)`.
    pub fn load_pinpoint(&mut self, p_const: &PlatePinpointConstraint) {
        self.ok = false;
        self.n_el += 1;
        self.my_constraints.append(*p_const);
        self.max_constraint_order = max_order(std::iter::once(p_const), self.max_constraint_order);
    }

    /// Adds a linear XYZ constraint (`Load(Plate_LinearXYZConstraint)`), sharing its
    /// arrays; Standard_NullObject for a constraint without arrays (a null dereference
    /// natively), after the plate was marked not done. `plate.load_linear_xyz(&c)?`.
    pub fn load_linear_xyz(
        &mut self,
        lxyz_const: &PlateLinearXYZConstraint,
    ) -> Result<(), Failure> {
        self.ok = false;
        let (ppc, coeff) = lxyz_const.snapshot()?;
        self.n_el += coeff.nb_columns() as i32;
        self.my_lxyz_constraints.append(lxyz_const.clone());
        self.max_constraint_order = max_order(ppc.iter(), self.max_constraint_order);
        Ok(())
    }

    /// Adds a linear scalar constraint (`Load(Plate_LinearScalarConstraint)`), sharing
    /// its arrays; Standard_NullObject for a constraint without arrays.
    /// `plate.load_linear_scalar(&c)?`.
    pub fn load_linear_scalar(
        &mut self,
        lscalar_const: &PlateLinearScalarConstraint,
    ) -> Result<(), Failure> {
        self.ok = false;
        let (ppc, coeff) = lscalar_const.snapshot()?;
        self.n_el += coeff.nb_columns() as i32;
        self.my_lscalar_constraints.append(lscalar_const.clone());
        self.max_constraint_order = max_order(ppc.iter(), self.max_constraint_order);
        Ok(())
    }

    /// Adds a global translation constraint (`Load(Plate_GlobalTranslationConstraint)`).
    /// `plate.load_global_translation(&c)?`.
    pub fn load_global_translation(
        &mut self,
        gt_const: &PlateGlobalTranslationConstraint,
    ) -> Result<(), Failure> {
        self.load_linear_xyz(gt_const.lxyzc())
    }

    /// Adds a line constraint (`Load(Plate_LineConstraint)`); `plate.load_line(&c)?`.
    pub fn load_line(&mut self, l_const: &PlateLineConstraint) -> Result<(), Failure> {
        self.load_linear_scalar(l_const.lsc())
    }

    /// Adds a plane constraint (`Load(Plate_PlaneConstraint)`); `plate.load_plane(&c)?`.
    pub fn load_plane(&mut self, p_const: &PlatePlaneConstraint) -> Result<(), Failure> {
        self.load_linear_scalar(p_const.lsc())
    }

    /// Adds a sampled curve constraint (`Load(Plate_SampledCurveConstraint)`).
    /// `plate.load_sampled_curve(&c)?`.
    pub fn load_sampled_curve(
        &mut self,
        sc_const: &PlateSampledCurveConstraint,
    ) -> Result<(), Failure> {
        self.load_linear_xyz(sc_const.lxyzc())
    }

    /// Adds the pinpoint constraints of a G1/G2/G3 constraint (`Load(Plate_GtoCConstraint)`).
    /// `plate.load_gtoc(&c)?`.
    pub fn load_gtoc(&mut self, gtoc_const: &PlateGtoCConstraint) -> Result<(), Failure> {
        for i in 0..gtoc_const.nb_ppc() {
            self.load_pinpoint(&gtoc_const.get_ppc(i)?);
        }
        Ok(())
    }

    /// Adds the pinpoint then the scalar constraints of a free G1/G2/G3 constraint
    /// (`Load(Plate_FreeGtoCConstraint)`). `plate.load_free_gtoc(&c)?`.
    pub fn load_free_gtoc(&mut self, fgtoc_const: &PlateFreeGtoCConstraint) -> Result<(), Failure> {
        for i in 0..fgtoc_const.nb_ppc() {
            self.load_pinpoint(&fgtoc_const.get_ppc(i)?);
        }
        for i in 0..fgtoc_const.nb_lsc() {
            self.load_linear_scalar(fgtoc_const.lsc(i)?)?;
        }
        Ok(())
    }

    /// Whether the last `SolveTI` succeeded (`IsDone`); true after `Init`, as natively.
    /// `plate.is_done()`.
    pub fn is_done(&self) -> bool {
        self.ok
    }

    /// `destroy`, which is `Init`; `plate.destroy()`.
    pub fn destroy(&mut self) {
        self.init();
    }

    /// Clears the constraints and the solution (`Init`); the plate then reports done,
    /// which the native test notes as a known oddity. `plate.init()`.
    pub fn init(&mut self) {
        self.my_constraints.clear(None);
        self.my_lxyz_constraints.clear(None);
        self.my_lscalar_constraints.clear(None);
        self.solution = None;
        self.points = None;
        self.deru = None;
        self.derv = None;
        self.order = 0;
        self.n_el = 0;
        self.n_dim = 0;
        self.ok = true;
        self.max_constraint_order = 0;
    }

    /// The plate value at `point2d` (`Evaluate`); the null vector before a successful
    /// solve. `plate.evaluate(Xy::new(0.5, 0.5))?`.
    pub fn evaluate(&self, point2d: Xy) -> Result<Xyz, Failure> {
        self.evaluate_derivative(point2d, 0, 0)
    }

    /// The `(iu, iv)` derivative at `point2d` (`EvaluateDerivative`); the null vector
    /// before a successful solve. Standard_OutOfRange when a derivative order reaches
    /// past the ten scale factors (undefined natively). `plate.evaluate_derivative(uv, 1, 0)?`.
    pub fn evaluate_derivative(&self, point2d: Xy, iu: i32, iv: i32) -> Result<Xyz, Failure> {
        let Some(solution) = self.solution.as_deref().filter(|_| self.ok) else {
            return Ok(Xyz::new(0.0, 0.0, 0.0));
        };
        let mut valeur = Xyz::new(0.0, 0.0, 0.0);
        if !self.polynomial_part_only {
            let nodes = self.nodes();
            let terms = solution
                .iter()
                .zip(nodes.points.iter().zip(nodes.deru.iter().zip(nodes.derv)));
            for (&coefficient, (&point, (&deru, &derv))) in terms.take(self.n_el as usize) {
                let signe: f64 = if (deru + derv) % 2 == 1 { -1.0 } else { 1.0 };
                let sol_em: f64 = self.sol_em(point2d - point, deru + iu, derv + iv)?;
                valeur += coefficient * (signe * sol_em);
            }
        }
        let polynomial = solution.iter().skip(self.n_el as usize);
        for (&coefficient, (idu, idv)) in polynomial.zip(super::system::monomials(self.order)) {
            valeur += coefficient * self.polm(point2d, idu, idv, iu, iv)?;
        }
        Ok(valeur)
    }

    /// The power-basis coefficients `(iu, iv)` of the polynomial part, zero above the
    /// total degree `order - 1` (`CoefPol`). Standard_NullObject for a plate never
    /// solved with a polynomial part (a null dereference natively).
    /// `plate.coef_pol()?`.
    pub fn coef_pol(&self) -> Result<Handle<HArray2<Xyz>>, Failure> {
        let zero = Xyz::new(0.0, 0.0, 0.0);
        let coefs = HArray2::with_value(0, self.order - 1, 0, self.order - 1, zero)
            .map_err(errors::array_bounds)?;
        let mut i: usize = self.n_el as usize;
        for iu in 0..self.order {
            for iv in 0..self.order - iu {
                let value: Xyz = self.solution_at(i)? * self.scale_u(iu)? * self.scale_v(iv)?;
                coefs
                    .change_array2()
                    .set_value(iu, iv, value)
                    .map_err(errors::array_bounds)?;
                i += 1;
            }
        }
        Ok(Handle::new(coefs))
    }

    /// The deprecated output form of `CoefPol`; `plate.coef_pol_into(&mut coefs)?`.
    pub fn coef_pol_into(&self, coefs: &mut Handle<HArray2<Xyz>>) -> Result<(), Failure> {
        *coefs = self.coef_pol()?;
        Ok(())
    }

    fn solution_at(&self, index: usize) -> Result<Xyz, Failure> {
        let solution = self.solution.as_deref().ok_or_else(|| {
            errors::null_object(
                "Plate_Plate::CoefPol: the plate has no solution; expected a solved plate",
            )
        })?;
        solution.get(index).copied().ok_or_else(|| {
            errors::out_of_range(&format!(
                "Plate_Plate::CoefPol: coefficient {index} outside the solution"
            ))
        })
    }

    /// Evaluates only the polynomial part from now on (`SetPolynomialPartOnly`; the
    /// native default is true). `plate.set_polynomial_part_only(true)`.
    pub fn set_polynomial_part_only(&mut self, pp_only: bool) {
        self.polynomial_part_only = pp_only;
    }

    /// The continuity order `2 * order - 3 - maxConstraintOrder` (`Continuity`).
    /// `plate.continuity()`.
    pub fn continuity(&self) -> i32 {
        2 * self.order - 3 - self.max_constraint_order
    }

    /// The parameters of the pinpoint constraints of order 0 (`UVConstraints`),
    /// appended to `seq`. `plate.uv_constraints(&mut seq)`.
    pub fn uv_constraints(&self, seq: &mut Sequence<Xy>) {
        for c in self.my_constraints.iter() {
            if c.idu() == 0 && c.idv() == 0 {
                seq.append(c.pnt2d());
            }
        }
    }
}
