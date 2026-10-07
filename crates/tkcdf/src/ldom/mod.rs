// Rust port of the ApplicationFramework/TKCDF/LDOM package
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! LDOM: the light DOM that reads and writes XML for OCAF (ocaf-design.md section 9).
//!
//! A document's nodes live in the arena of its [`LdomMemManager`]; the native basic-node
//! pointers are [`LdomNodeId`]s, the basic nodes are views ([`LdomBasicElement`]), and the
//! DOM values ([`LdomNode`], [`LdomElement`], ...) hold the document handle and an id.
mod arena;
mod basic_nodes;
mod basic_string;
mod c_text;
mod char_reference;
mod document;
mod element;
mod element_ops;
mod istream;
mod mem_manager;
mod node;
mod node_list;
mod os_stream;
mod parser;
mod replace;
mod store;
mod string;
mod values;
mod xml_reader;
mod xml_states;
mod xml_writer;

pub use arena::{LdomNodeId, LdomNodeType};
pub use basic_nodes::{LdomBasicAttribute, LdomBasicElement, LdomBasicNode, LdomBasicText};
pub use basic_string::{LdomBasicString, LdomChars, LdomStringType};
pub use char_reference::LdomCharReference;
pub use document::LdomDocument;
pub use element::LdomElement;
pub use istream::LdomIStream;
pub use mem_manager::LdomMemManager;
pub use node::LdomNode;
pub use node_list::LdomNodeList;
pub use os_stream::{LdomBomType, LdomOSStream, LdomSBuffer};
pub use parser::{LdomParser, LdomParserHooks};
pub use string::LdomString;
pub use values::{LdomAttr, LdomCdataSection, LdomCharacterData, LdomComment, LdomText};
pub use xml_reader::{LdomXmlReader, LdomXmlRecordType, XML_BUFFER_SIZE};
pub use xml_writer::LdomXmlWriter;
