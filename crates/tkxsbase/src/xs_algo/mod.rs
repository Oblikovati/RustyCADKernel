//! `XSAlgo`: the shape processing of the exchange formats (`XSAlgo_ShapeProcessor`,
//! `XSAlgo_AlgoContainer`); xsbase-design.md section 4.
mod algo_container;
mod check_pcurve;
mod de_shape_fix_parameters;
mod merge;
mod shape_processor;
mod shape_processor_types;
#[cfg(test)]
mod tests;

pub use algo_container::{
    XSAlgo, XSAlgoAlgoContainer, XSAlgoAlgoContainerOps, XSAlgoProcessShapeArgs,
};
pub use de_shape_fix_parameters::{DEShapeFixMode, DEShapeFixParameters};
pub use shape_processor::{XSAlgoShapeProcessor, xs_algo_units};
pub use shape_processor_types::{XSAlgoParameterMap, XSAlgoProcessingData, XSAlgoProcessingFlags};
