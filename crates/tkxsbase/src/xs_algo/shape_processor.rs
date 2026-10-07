// Rust port of DataExchange/TKXSBase/XSAlgo/XSAlgo_ShapeProcessor (.hxx, .cxx): the
// parameters and the processing at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XSAlgo_ShapeProcessor`: the shape processing (healing) of the exchange formats, by
//! named parameters and operation flags.
use super::{DEShapeFixMode, DEShapeFixParameters, XSAlgoParameterMap, XSAlgoProcessingData};
use crate::interface::InterfaceStatic;
use crate::transfer::{TransferFinderProcess, TransferTransientProcess};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{
    message::ProgressRange,
    standard::{Failure, Handle},
    tcollection::AsciiString,
    units_methods::UnitsMethods,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkshhealing::shape_process::{
    ShapeProcess, ShapeProcessContext, ShapeProcessOperationsFlags, ShapeProcessShapeContext,
};
use std::sync::OnceLock;

/// The process-wide cascade length unit of the exchange formats (the native global of
/// `UnitsMethods`, which the TKernel port keeps per instance).
pub fn xs_algo_units() -> &'static UnitsMethods {
    static UNITS: OnceLock<UnitsMethods> = OnceLock::new();
    UNITS.get_or_init(UnitsMethods::default)
}

/// `XSAlgo_ShapeProcessor`.
///
/// ```
/// use rustycad_tkxsbase::xs_algo::{XSAlgoParameterMap, XSAlgoShapeProcessor};
/// let processor = XSAlgoShapeProcessor::new(&XSAlgoParameterMap::default(), &Default::default());
/// assert!(processor.get_context().is_null());
/// ```
pub struct XSAlgoShapeProcessor {
    my_parameters: XSAlgoParameterMap,
    my_context: Handle<ShapeProcessShapeContext>,
}

impl XSAlgoShapeProcessor {
    /// `XSAlgo_ShapeProcessor(theParameters, theShapeFixParameters = {})`: the parameters,
    /// completed (not replaced) by the shape fix parameters.
    pub fn new(
        parameters: &XSAlgoParameterMap,
        shape_fix_parameters: &DEShapeFixParameters,
    ) -> Self {
        let mut my_parameters = parameters.clone();
        Self::fill_parameter_map(shape_fix_parameters, false, &mut my_parameters);
        Self {
            my_parameters,
            my_context: Handle::default(),
        }
    }
    /// `XSAlgo_ShapeProcessor(theParameters)`: the shape fix parameters only.
    pub fn from_shape_fix(parameters: &DEShapeFixParameters) -> Self {
        Self::new(&XSAlgoParameterMap::default(), parameters)
    }
    /// `ProcessShape(theShape, theOperations, theProgress)`: the processed shape (the
    /// shape itself when no operation applies).
    pub fn process_shape(
        &mut self,
        shape: &Shape,
        operations: ShapeProcessOperationsFlags,
        progress: &ProgressRange<'_>,
    ) -> Result<Shape, Failure> {
        if shape.is_null() {
            return Ok(shape.clone());
        }
        self.initialize_context(shape)?;
        let context: Handle<ShapeProcessContext> = self.my_context.cast();
        if ShapeProcess::perform_operations(&context, operations, progress)? {
            return Ok(self
                .my_context
                .get()
                .map(ShapeProcessShapeContext::result)
                .unwrap_or_default());
        }
        Ok(shape.clone())
    }
    /// `initializeContext` (private): a new context of `shape` with the parameters as
    /// resources, the detalisation level and the non-manifold flag.
    fn initialize_context(&mut self, shape: &Shape) -> Result<(), Failure> {
        let context = ShapeProcessShapeContext::with_shape(shape, "", "")?;
        if let Some(resource) = context.resource_manager() {
            let mut manager = resource.write();
            for (key, value) in self.my_parameters.items() {
                manager.set_resource(&text(key), value.as_bytes());
            }
        }
        let level = self
            .my_parameters
            .seek(&AsciiString::new("DetalizationLevel"))
            .filter(|v| v.is_integer_value());
        // a value out of TopAbs_ShapeEnum (an undefined cast natively) is ignored
        if let Some(kind) = level
            .and_then(|v| v.integer_value().ok())
            .and_then(|i| usize::try_from(i).ok())
            .and_then(|i| ShapeEnum::ALL.get(i))
        {
            context.set_detalisation(*kind);
        }
        let manifold = self
            .my_parameters
            .seek(&AsciiString::new("NonManifold"))
            .filter(|v| v.is_integer_value());
        if let Some(value) = manifold.and_then(|v| v.integer_value().ok()) {
            context.set_non_manifold(value != 0);
        }
        self.my_context = Handle::new(context);
        Ok(())
    }
    /// `GetContext`: the context of the last processing (null before).
    pub fn get_context(&self) -> Handle<ShapeProcessShapeContext> {
        self.my_context.clone()
    }
    /// `MergeTransferInfo(theTransientProcess, theFirstTPItemIndex)`: the modifications of
    /// the last processing into the shape binders of the process.
    pub fn merge_transfer_info(
        &self,
        transient_process: &Handle<TransferTransientProcess>,
        first_tp_item_index: i32,
    ) -> Result<(), Failure> {
        let Some(context) = self.my_context.get() else {
            return Ok(());
        };
        Self::merge_shape_transfer_info(
            transient_process,
            &context.map(),
            first_tp_item_index,
            &context.messages(),
        )
    }
    /// `MergeTransferInfo(theFinderProcess)`: the modifications of the last processing
    /// into the finder process.
    pub fn merge_transfer_info_finder(
        &self,
        finder_process: &Handle<TransferFinderProcess>,
    ) -> Result<(), Failure> {
        let Some(context) = self.my_context.get() else {
            return Ok(());
        };
        Self::merge_shape_transfer_info_finder(finder_process, &context.map(), &context.messages())
    }
    /// `ReadProcessingData(theFileResourceName, theScopeResourceName)`: the parameters
    /// and the operations of a scope of the resource file named by a static (the statics
    /// themselves without file).
    pub fn read_processing_data(
        file_resource_name: &AsciiString,
        scope_resource_name: &AsciiString,
    ) -> Result<XSAlgoProcessingData, Failure> {
        let file_name = InterfaceStatic::c_val(&text(file_resource_name));
        let context =
            ShapeProcessShapeContext::with_shape(&Shape::default(), &text(&file_name), "")?;
        let resource = context.resource_manager().ok_or_else(|| {
            rustycad_tkernel::standard::exceptions::null_object(
                "XSAlgo_ShapeProcessor::ReadProcessingData: no resource manager",
            )
        })?;
        let mut manager = resource.write();
        if !manager.is_initialized() {
            InterfaceStatic::fill_map(manager.get_map(true));
        }
        let scope = InterfaceStatic::c_val(&text(scope_resource_name));
        let mut parameters = XSAlgoParameterMap::default();
        let mut flags = ShapeProcessOperationsFlags::default();
        for (key, value) in manager.get_map(true).items() {
            scoped_entry(&scope, key, value, &mut parameters, &mut flags)?;
        }
        Ok((parameters, flags))
    }
    /// `FillParameterMap(theParameters, theIsReplace, theMap)`: the shape fix parameters
    /// as `FixShape.` resources.
    pub fn fill_parameter_map(
        parameters: &DEShapeFixParameters,
        is_replace: bool,
        map: &mut XSAlgoParameterMap,
    ) {
        Self::set_parameter_real(
            "FixShape.Tolerance3d",
            parameters.tolerance3d,
            is_replace,
            map,
        );
        Self::set_parameter_real(
            "FixShape.MaxTolerance3d",
            parameters.max_tolerance3d,
            is_replace,
            map,
        );
        Self::set_parameter_real(
            "FixShape.MinTolerance3d",
            parameters.min_tolerance3d,
            is_replace,
            map,
        );
        Self::set_parameter(
            "DetalizationLevel",
            &AsciiString::from_integer(parameters.detalization_level as i32),
            is_replace,
            map,
        );
        Self::set_parameter(
            "NonManifold",
            &AsciiString::from_integer(i32::from(parameters.non_manifold)),
            is_replace,
            map,
        );
        for (name, mode) in parameters.fix_modes() {
            Self::set_parameter_mode(&format!("FixShape.{name}"), mode, is_replace, map);
        }
    }
    /// `SetShapeFixParameters(theParameters, theAdditionalParameters, theTargetParameterMap)`:
    /// the shape fix parameters (replacing), then the additional ones (not replacing).
    pub fn set_shape_fix_parameters(
        parameters: &DEShapeFixParameters,
        additional: &XSAlgoParameterMap,
        target: &mut XSAlgoParameterMap,
    ) {
        Self::fill_parameter_map(parameters, true, target);
        for (key, value) in additional.items() {
            Self::set_parameter(&text(key), value, false, target);
        }
    }
    /// `SetParameter(theKey, FixMode theValue, theIsReplace, theMap)`: the mode as an integer.
    pub fn set_parameter_mode(
        key: &str,
        value: DEShapeFixMode,
        is_replace: bool,
        map: &mut XSAlgoParameterMap,
    ) {
        Self::set_parameter(
            key,
            &AsciiString::from_integer(value as i32),
            is_replace,
            map,
        );
    }
    /// `SetParameter(theKey, double theValue, theIsReplace, theMap)`: the real as text.
    pub fn set_parameter_real(
        key: &str,
        value: f64,
        is_replace: bool,
        map: &mut XSAlgoParameterMap,
    ) {
        Self::set_parameter(key, &AsciiString::from_real(value), is_replace, map);
    }
    /// `SetParameter(theKey, theValue, theIsReplace, theMap)`: binds the value unless the
    /// key exists and is not replaced (or already has this value).
    pub fn set_parameter(
        key: &str,
        value: &AsciiString,
        is_replace: bool,
        map: &mut XSAlgoParameterMap,
    ) {
        let key = AsciiString::new(key);
        if map
            .seek(&key)
            .is_some_and(|existing| !is_replace || existing == value)
        {
            return;
        }
        map.bind(key, value.clone());
    }
    /// `PrepareForTransfer`: the cascade length unit from the static "xstep.cascade.unit".
    pub fn prepare_for_transfer() {
        xs_algo_units().set_cascade_length_unit_code(InterfaceStatic::i_val("xstep.cascade.unit"));
    }
}

/// The text of an ASCII string (a C string natively).
fn text(value: &AsciiString) -> String {
    String::from_utf8_lossy(value.as_bytes()).into_owned()
}

/// One resource of `ReadProcessingData`: a key of `scope` becomes a parameter without the
/// scope prefix, or the operation flags for `exec.op`.
fn scoped_entry(
    scope: &AsciiString,
    key: &AsciiString,
    value: &AsciiString,
    parameters: &mut XSAlgoParameterMap,
    flags: &mut ShapeProcessOperationsFlags,
) -> Result<(), Failure> {
    if !key.starts_with(scope.as_bytes()) {
        return Ok(());
    }
    let short = key.sub_string(scope.len() as i32 + 2, key.len() as i32)?;
    if short.as_bytes() != b"exec.op" {
        parameters.bind(short, value.clone());
        return Ok(());
    }
    let mut count = 1;
    loop {
        let token = value.token(" \t,;", count);
        if token.is_empty() {
            return Ok(());
        }
        let (operation, known) = ShapeProcess::to_operation_flag(Some(&text(&token)));
        if known {
            flags.set(operation);
        }
        count += 1;
    }
}
