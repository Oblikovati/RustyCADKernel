//! `TransferBRep`: the transfer of shapes (binders of shape results, finders of shapes,
//! result counts and the shape check lists); xsbase-design.md section 3.
mod binders;
mod package;
mod package_checks;
mod shape_mapper;
#[cfg(test)]
mod tests;

pub use binders::{
    TransferBRepBinderOfShape, TransferBRepShapeBinder, TransferBRepShapeListBinder,
};
pub use package::TransferBRep;
pub use shape_mapper::{
    TransferBRepShapeInfo, TransferBRepShapeMapper, TransferBRepTransferResultInfo,
};
