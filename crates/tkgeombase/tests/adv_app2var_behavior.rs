//! Behaviour of the AdvApp2Var classes that the upstream suites do not reach: the
//! full approximation, its getters and exceptions, the criterion hooks and Dump.
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adv_approx::AdvApproxDichoCutting;
use rustycad_tkgeombase::adv_app2var::{
    AdvApp2VarApproxAFunc2Var, AdvApp2VarContext, AdvApp2VarCriterion,
    AdvApp2VarCriterionRepartition, AdvApp2VarCriterionType, AdvApp2VarEvaluation, AdvApp2VarPatch,
};
use rustycad_tkmath::geom_abs::{IsoType, Shape};

/// The plane F(u, v) = (u, v, u + 2v) with its derivatives: an exact bilinear surface.
struct PlaneFunction;

impl rustycad_tkgeombase::adv_app2var::AdvApp2VarEvaluatorFunc2Var for PlaneFunction {
    fn evaluate(
        &self,
        request: &AdvApp2VarEvaluation<'_>,
        result: &mut [f64],
        error_code: &mut i32,
    ) {
        for i in 0..request.nb_params as usize {
            let t = request.parameters[i];
            let (u, v) = if request.favor_iso == 1 {
                (request.const_param, t)
            } else {
                (t, request.const_param)
            };
            let value = match (request.u_order, request.v_order) {
                (0, 0) => [u, v, u + 2. * v],
                (1, 0) => [1., 0., 1.],
                (0, 1) => [0., 1., 2.],
                _ => [0., 0., 0.],
            };
            result[3 * i..3 * i + 3].copy_from_slice(&value);
        }
        *error_code = 0;
    }
}

/// A criterion that is never satisfied, to count how the approximation cuts.
struct NeverSatisfied(AdvApp2VarCriterionType);

impl AdvApp2VarCriterion for NeverSatisfied {
    fn value(&self, patch: &AdvApp2VarPatch, _context: &AdvApp2VarContext) -> Result<(), Failure> {
        patch.set_crit_value(1.0);
        Ok(())
    }
    fn is_satisfied(&self, patch: &AdvApp2VarPatch) -> bool {
        patch.crit_value() < 0.5
    }
    fn max_value(&self) -> f64 {
        0.5
    }
    fn criterion_type(&self) -> AdvApp2VarCriterionType {
        self.0
    }
    fn repartition(&self) -> AdvApp2VarCriterionRepartition {
        AdvApp2VarCriterionRepartition::Incremental
    }
}

fn tolerances() -> (Array1<'static, f64>, Array2<'static, f64>) {
    let mut tol = Array1::with_bounds(1, 1).unwrap();
    tol.set_value(1, 1.0e-6).unwrap();
    let mut front = Array2::with_bounds(1, 1, 1, 4).unwrap();
    front.init(1.0e-6);
    (tol, front)
}

fn approximate(cont: Shape, max_patch: i32) -> Result<AdvApp2VarApproxAFunc2Var, Failure> {
    let (tol, front) = tolerances();
    let cut = AdvApproxDichoCutting::new();
    AdvApp2VarApproxAFunc2Var::new(
        0,
        0,
        1,
        None,
        None,
        Some(&tol),
        None,
        None,
        Some(&front),
        0.0,
        1.0,
        0.0,
        2.0,
        IsoType::IsoV,
        cont,
        cont,
        1,
        5,
        5,
        max_patch,
        &PlaneFunction,
        &cut,
        &cut,
    )
}

#[test]
fn approximation_of_a_plane_is_one_exact_patch() {
    let approx = approximate(Shape::C1, 4).unwrap();
    assert!(approx.is_done() && approx.has_result());
    assert!(approx.max_error(3, 1).unwrap() < 1.0e-12);
    assert_eq!(approx.num_sub_spaces(3).unwrap(), 1);
    let surface = approx.surface(1).unwrap();
    assert_eq!((surface.nb_u_knots(), surface.nb_v_knots()), (2, 2));
    let corner = surface
        .pole(surface.nb_u_poles(), surface.nb_v_poles())
        .unwrap();
    assert!((corner.x() - 1.0).abs() < 1.0e-12 && (corner.z() - 5.0).abs() < 1.0e-12);
}

#[test]
fn approximation_refuses_continuities_above_c2() {
    let error = approximate(Shape::C3, 4).err().unwrap();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    assert!(approximate(Shape::G1, 4).is_err());
}

#[test]
fn approximation_getters_raise_outside_one_3d_surface() {
    let approx = approximate(Shape::C0, 4).unwrap();
    assert!(approx.max_errors(1).unwrap().is_none());
    assert_eq!(
        approx.max_errors(4).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        approx.crit_error(3, 2).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        approx.surface(2).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        approx.num_sub_spaces(0).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn dump_reports_the_tolerance_and_the_knots() {
    let approx = approximate(Shape::C1, 4).unwrap();
    let mut text: Vec<u8> = Vec::new();
    approx.dump(&mut text).unwrap();
    let text = String::from_utf8(text).unwrap();
    assert!(text.starts_with("\nThere is a result within the requested tolerance 1e-06\n"));
    assert!(text.contains("Number of knots in V : 2\n   1 : 0   mult : "));
}

#[test]
fn a_criterion_never_satisfied_cuts_until_the_patch_budget() {
    let (tol, front) = tolerances();
    let cut = AdvApproxDichoCutting::new();
    let crit = NeverSatisfied(AdvApp2VarCriterionType::Relative);
    let approx = AdvApp2VarApproxAFunc2Var::with_criterion(
        0,
        0,
        1,
        None,
        None,
        Some(&tol),
        None,
        None,
        Some(&front),
        0.0,
        1.0,
        0.0,
        2.0,
        IsoType::IsoV,
        Shape::C1,
        Shape::C1,
        1,
        5,
        5,
        4,
        &PlaneFunction,
        &crit,
        &cut,
        &cut,
    )
    .unwrap();
    let surface = approx.surface(1).unwrap();
    assert!(surface.nb_u_knots() * surface.nb_v_knots() > 4);
    assert_eq!(approx.crit_error(3, 1).unwrap(), 1.0);
}

#[test]
fn patch_cut_sense_follows_the_criterion_type() {
    let patch = AdvApp2VarPatch::with_domain([0.0, 1.0, 0.0, 1.0], 1, 1);
    patch.set_crit_value(1.0);
    // A relative criterion keeps the approximation's decision while it is not approximated.
    assert_eq!(
        patch.cut_sense_with(&NeverSatisfied(AdvApp2VarCriterionType::Relative), 3),
        0
    );
    assert_eq!(
        patch.cut_sense_with(&NeverSatisfied(AdvApp2VarCriterionType::Absolute), 3),
        3
    );
    patch.set_crit_value(0.1);
    assert_eq!(
        patch.cut_sense_with(&NeverSatisfied(AdvApp2VarCriterionType::Absolute), 3),
        0
    );
}
