//! `Interface`: the entity model of the exchange formats (models, protocols, generic
//! libraries, checks, graphs, copies, file reading tools and static parameters);
//! docs/xsbase-design.md section 1.
mod bit_map;
mod category;
mod check;
mod check_edit;
mod check_iterator;
mod check_iterator_print;
#[cfg(test)]
mod check_tests;
mod check_tool;
mod check_tool_lists;
mod copy_map;
mod copy_tool;
mod entity_cluster;
mod entity_iterator;
mod entity_list;
mod enums;
mod errors;
mod file_parameter;
mod file_reader_data;
mod file_reader_load;
mod file_reader_tool;
mod float_writer;
mod general_lib;
mod general_module;
mod graph;
mod graph_content;
mod graph_fill;
mod graph_queries;
#[cfg(test)]
mod graph_tests;
mod gtool;
mod h_graph;
mod int_array;
mod int_list;
mod int_val;
mod interface_model;
mod line_buffer;
mod model_base;
mod model_entities;
mod model_reports;
#[cfg(test)]
mod model_tests;
mod msg;
mod msg_text;
mod param_list;
mod param_set;
mod protocol;
mod reader_module;
#[cfg(test)]
mod reader_tests;
mod report_entity;
mod share_flags;
mod share_tool;
mod sign_type;
mod static_registry;
mod static_standards;
#[cfg(test)]
mod static_tests;
mod static_value;
#[cfg(test)]
pub(crate) mod test_norm;
mod typed_value;
mod undefined_content;

pub use bit_map::InterfaceBitMap;
pub use category::InterfaceCategory;
pub use check::InterfaceCheck;
pub use check_iterator::InterfaceCheckIterator;
pub use check_tool::InterfaceCheckTool;
pub use copy_map::{InterfaceCopyControl, InterfaceCopyMap};
pub use copy_tool::InterfaceCopyTool;
pub use entity_cluster::InterfaceEntityCluster;
pub use entity_iterator::{InterfaceEntityIterator, InterfaceEntityListHandle};
pub use entity_list::InterfaceEntityList;
pub use enums::{InterfaceCheckStatus, InterfaceDataState, InterfaceParamType};
pub use errors::{check_failure, interface_error, interface_mismatch};
pub use file_parameter::InterfaceFileParameter;
pub use file_reader_data::{
    InterfaceFileReaderData, InterfaceFileReaderDataBase, InterfaceFileReaderDataParams,
};
pub use file_reader_load::InterfaceFileReaderToolLoad;
pub use file_reader_tool::{
    InterfaceFileReaderTool, InterfaceFileReaderToolBase, InterfaceFileReaderToolOps,
};
pub use float_writer::{InterfaceFloatWriter, InterfaceFloatWriterOptions};
pub use general_lib::{
    InterfaceGeneralLib, InterfaceGeneralLibKind, InterfaceGlobalNodeOfGeneralLib,
    InterfaceNodeOfGeneralLib,
};
pub use general_module::InterfaceGeneralModule;
pub use graph::{InterfaceGraph, InterfaceSharingTable};
pub use graph_content::{InterfaceGraphContent, InterfaceGraphContentEvaluate};
pub use gtool::InterfaceGTool;
pub use h_graph::InterfaceHGraph;
pub use int_array::InterfaceIntArray;
pub use int_list::InterfaceIntList;
pub use int_val::InterfaceIntVal;
pub use interface_model::InterfaceInterfaceModel;
pub use line_buffer::InterfaceLineBuffer;
pub use model_base::InterfaceModelBase;
pub use model_entities::InterfaceModelEntities;
pub use model_reports::InterfaceModelReports;
pub use msg::InterfaceMsg;
pub use param_list::InterfaceParamList;
pub use param_set::InterfaceParamSet;
pub use protocol::{InterfaceProtocol, InterfaceProtocolBase};
pub use reader_module::{
    InterfaceGlobalNodeOfReaderLib, InterfaceNodeOfReaderLib, InterfaceReaderLib,
    InterfaceReaderLibKind, InterfaceReaderModule,
};
pub use report_entity::InterfaceReportEntity;
pub use share_flags::InterfaceShareFlags;
pub use share_tool::InterfaceShareTool;
pub use sign_type::{InterfaceSignType, InterfaceSignTypeBase, interface_sign_type_text};
pub use static_value::{InterfaceStatic, InterfaceStaticSatisfies};
pub use typed_value::InterfaceTypedValue;
pub use undefined_content::InterfaceUndefinedContent;
