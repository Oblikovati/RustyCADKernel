// Rust port of ApplicationFramework/TKVCAF/TPrsStd/TPrsStd_ConstraintDriver.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TPrsStd_ConstraintDriver`: the `PrsDim` presentation of a `TDataXtd_Constraint`
//! (`TPrsStd_ConstraintTools`), colored by the state of the constraint.
use super::TPrsStdDriver;
use super::constraint_tools::{ConstraintPresentation, TPrsStdConstraintTools};
use super::construction_drivers::presentation_driver;
use rustycad_tkcaf::tdata_xtd::{TDataXtdConstraint, TDataXtdConstraintEnum, TDataXtdPosition};
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tklcaf::tdf::TdfLabel;
use rustycad_tkv3d::ais::{AisInteractiveObject, AisInteractiveObjectMembers};
use rustycad_tkv3d::prs_dim::{PrsDimRelationMembers, PrsDimRelationSource};
use rustycad_tkv3d::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkv3d::select_mgr::SelectMgrSelectableObjectMembers;

presentation_driver!(
    /// `TPrsStd_ConstraintDriver`: the presentation of a `TDataXtd_Constraint`.
    TPrsStdConstraintDriver,
    "TPrsStd_ConstraintDriver"
);

/// The `ComputeX` member for a constraint type (the `switch` of `Update`).
fn compute_for(
    kind: TDataXtdConstraintEnum,
    constraint: &Handle<TDataXtdConstraint>,
) -> ConstraintPresentation {
    use TDataXtdConstraintEnum as K;
    type Tools = TPrsStdConstraintTools;
    match kind {
        K::Distance => Tools::compute_distance(constraint),
        K::Parallel => Tools::compute_parallel(constraint),
        K::Perpendicular => Tools::compute_perpendicular(constraint),
        K::Concentric => Tools::compute_concentric(constraint),
        K::Symmetry => Tools::compute_symmetry(constraint),
        K::Midpoint => Tools::compute_mid_point(constraint),
        K::Tangent => Tools::compute_tangent(constraint),
        K::Angle => Tools::compute_angle(constraint),
        K::Radius => Tools::compute_radius(constraint),
        K::MinorRadius => Tools::compute_min_radius(constraint),
        K::MajorRadius => Tools::compute_max_radius(constraint),
        K::Diameter => Tools::compute_diameter(constraint),
        K::Fix => Tools::compute_fix(constraint),
        K::Offset => Tools::compute_offset(constraint),
        K::Coincident => Tools::compute_coincident(constraint),
        K::Round => Tools::compute_round(constraint),
        K::Mate | K::AlignFaces | K::AlignAxes | K::AxesAngle => {
            Tools::compute_placement(constraint)
        }
        K::EqualDistance => Tools::compute_equal_distance(constraint),
        K::EqualRadius => Tools::compute_equal_radius(constraint),
        _ => Tools::compute_others(constraint),
    }
}

/// The color of a constraint's object: red when not verified, purple for a dimension with a
/// captured value, yellow when not planar; `None` keeps the color.
fn state_color(constraint: &TDataXtdConstraint) -> Result<Option<NamedColor>, Failure> {
    if !constraint.verified() {
        return Ok(Some(NamedColor::RED));
    }
    if constraint.is_dimension() {
        let value = constraint.get_value();
        let value = rustycad_tklcaf::live(&value, "TPrsStd_ConstraintDriver::Update")?;
        if value.is_captured()? {
            return Ok(Some(NamedColor::PURPLE));
        }
    }
    if !constraint.is_planar() {
        return Ok(Some(NamedColor::YELLOW));
    }
    Ok(None)
}

impl TPrsStdDriver for TPrsStdConstraintDriver {
    /// `Update`: an object already in a context of an unverified constraint only takes
    /// the new value text and red; otherwise the object is recomputed, placed at the
    /// label's `TDataXtd_Position` and colored by the constraint's state (in a context
    /// only when its color changes).
    fn update(
        &self,
        label: &TdfLabel,
        ais: &mut Handle<dyn AisInteractiveObject>,
    ) -> Result<bool, Failure> {
        let mut constraint: Handle<TDataXtdConstraint> = Handle::default();
        if !label.find_attribute_as(&TDataXtdConstraint::get_id(), &mut constraint)? {
            return Ok(false);
        }
        let value = rustycad_tklcaf::live(&constraint, "TPrsStd_ConstraintDriver::Update")?;
        if let Some(object) = ais.get().filter(|object| object.has_interactive_context())
            && !value.verified()
        {
            TPrsStdConstraintTools::update_only_value(&constraint, ais)?;
            if object.color().name() != NamedColor::RED {
                object.set_color(Color::named(NamedColor::RED))?;
            }
            return Ok(true);
        }
        let computed = compute_for(value.get_type(), &constraint)?;
        let Some(object) = computed.get() else {
            return Ok(false);
        };
        object.reset_transformation()?;
        object.set_to_update_all();
        object.update_selection(-1);
        *ais = computed.clone();
        place_at_position(label, &computed)?;
        let Some(color) = state_color(value)? else {
            return Ok(true);
        };
        if !object.has_interactive_context() || object.color().name() != color {
            object.set_color(Color::named(color))?;
        }
        Ok(true)
    }
}

/// `occ::down_cast<PrsDim_Relation>(anAISObject)->SetPosition(...)` when the label has a
/// `TDataXtd_Position`. A dimension (not a relation) is a null dereference natively, which
/// is `Standard_NullObject` here.
fn place_at_position(
    label: &TdfLabel,
    object: &Handle<dyn AisInteractiveObject>,
) -> Result<(), Failure> {
    let mut position: Handle<TDataXtdPosition> = Handle::default();
    if !label.find_attribute_as(&TDataXtdPosition::get_id(), &mut position)? {
        return Ok(());
    }
    let point =
        rustycad_tklcaf::live(&position, "TPrsStd_ConstraintDriver::Update")?.get_position();
    let relation = object.cast::<dyn PrsDimRelationSource>();
    rustycad_tklcaf::live(&relation, "TPrsStd_ConstraintDriver::Update")?.set_position(&point);
    Ok(())
}
