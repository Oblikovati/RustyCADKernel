// Rust translation of FoundationClasses/TKMath/PLib/PLib.cxx (JacobiParameters, NivConstr, ConstraintOrder) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Constraint levels and Jacobi discretization parameters.
use super::access::failure;
use crate::geom_abs::Shape;
use rustycad_tkernel::standard::Failure;

// The Gauss point counts the Jacobi tables are tabulated for.
const GAUSS_COUNTS: [i32; 9] = [8, 10, 15, 20, 25, 30, 40, 50, 61];

fn construction(message: &str) -> Failure {
    failure("Standard_ConstructionError", message)
}

/// Translates C0/C1/C2 to 0/1/2 (`PLib::NivConstr`); other shapes raise
/// Standard_ConstructionError; `plib::niv_constr(Shape::C1)? == 1`.
pub fn niv_constr(constraint_order: Shape) -> Result<i32, Failure> {
    match constraint_order {
        Shape::C0 => Ok(0),
        Shape::C1 => Ok(1),
        Shape::C2 => Ok(2),
        other => Err(construction(&format!(
            "Invalid ConstraintOrder ({other:?}; expected C0, C1 or C2)"
        ))),
    }
}

/// Translates 0/1/2 to C0/C1/C2 (`PLib::ConstraintOrder`);
/// `plib::constraint_order(2)? == Shape::C2`.
pub fn constraint_order(niv_constr: i32) -> Result<Shape, Failure> {
    match niv_constr {
        0 => Ok(Shape::C0),
        1 => Ok(Shape::C1),
        2 => Ok(Shape::C2),
        other => Err(construction(&format!(
            "Invalid NivConstr ({other}; expected 0, 1 or 2)"
        ))),
    }
}

/// Chooses the Gauss point count and Jacobi work degree (`PLib::JacobiParameters`).
///
/// Codes -5..-1 and 1..4 request 8..61 points. As natively, `work_degree` is
/// already written when a later MaxDegree or Code check raises;
/// `plib::jacobi_parameters(Shape::C1, 15, 2, &mut points, &mut degree)?`.
pub fn jacobi_parameters(
    constraint_order: Shape,
    max_degree: i32,
    code: i32,
    nb_gauss_points: &mut i32,
    work_degree: &mut i32,
) -> Result<(), Failure> {
    let level: i32 = niv_constr(constraint_order)?;
    if max_degree < 2 * level + 1 {
        return Err(construction(&format!(
            "Invalid MaxDegree ({max_degree}; expected >= {})",
            2 * level + 1
        )));
    }
    *work_degree = if code >= 1 {
        max_degree + 9
    } else {
        max_degree + 6
    };
    // Fewest tabulated points strictly above the work degree.
    let minimum: i32 = *GAUSS_COUNTS
        .iter()
        .find(|&&count| *work_degree < count)
        .ok_or_else(|| {
            construction(&format!(
                "Invalid MaxDegree (work degree {}; expected < 61)",
                *work_degree
            ))
        })?;
    let wanted: i32 = match code {
        -5..=-1 => GAUSS_COUNTS[(code + 5) as usize],
        1..=4 => GAUSS_COUNTS[(code + 4) as usize],
        other => {
            return Err(construction(&format!(
                "Invalid Code ({other}; expected -5..-1 or 1..4)"
            )));
        }
    };
    *nb_gauss_points = minimum.max(wanted);
    Ok(())
}
