use super::ShapeUpgradeShapeConvertToBezier;
use rustycad_tkernel::standard::Failure;

pub(super) fn perform(
    tool: &ShapeUpgradeShapeConvertToBezier,
    new_context: bool,
) -> Result<bool, Failure> {
    let level = tool.shape_divide_level() + 1;
    tool.set_shape_divide_level(level);
    let changed = run_at_level(tool, new_context, level)?;
    if level == 1 {
        super::postprocess::run(tool)?;
    }
    tool.set_shape_divide_level(level - 1);
    Ok(changed)
}

fn run_at_level(
    tool: &ShapeUpgradeShapeConvertToBezier,
    new_context: bool,
    level: i32,
) -> Result<bool, Failure> {
    if level != 1 {
        return tool.run_base_divide(new_context);
    }
    run_first_level(tool, new_context)
}

fn run_first_level(
    tool: &ShapeUpgradeShapeConvertToBezier,
    new_context: bool,
) -> Result<bool, Failure> {
    let modes = tool.modes();
    let mut changed = false;
    let mut next_context = new_context;
    if modes.surfaces {
        changed = run_surface_phase(tool, modes, next_context)?;
        next_context = false;
    }
    if modes.curves_2d || modes.curves_3d {
        changed = run_curve_phase(tool, modes, next_context)?;
    }
    Ok(changed)
}

fn run_surface_phase(
    tool: &ShapeUpgradeShapeConvertToBezier,
    modes: super::ConversionModes,
    new_context: bool,
) -> Result<bool, Failure> {
    tool.set_modes(disable_curve_modes(modes));
    let changed = tool.run_base_divide(new_context)?;
    tool.set_source_to_result();
    tool.set_modes(modes);
    Ok(changed)
}

fn run_curve_phase(
    tool: &ShapeUpgradeShapeConvertToBezier,
    modes: super::ConversionModes,
    new_context: bool,
) -> Result<bool, Failure> {
    tool.set_modes(disable_surface_mode(modes));
    let changed = tool.run_base_divide(new_context)?;
    tool.set_modes(modes);
    Ok(changed)
}

fn disable_curve_modes(modes: super::ConversionModes) -> super::ConversionModes {
    super::ConversionModes {
        curves_2d: false,
        curves_3d: false,
        ..modes
    }
}

fn disable_surface_mode(modes: super::ConversionModes) -> super::ConversionModes {
    super::ConversionModes {
        surfaces: false,
        ..modes
    }
}
