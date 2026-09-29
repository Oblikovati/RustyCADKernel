// Copyright (c) 2026 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Direct translation of BRepGraph_LayerIterator_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkbrep::brep_graph::LayerIterator;
use rustycad_tkbrep::brep_graph::NodeId;
use rustycad_tkbrep::brep_graph::copy_remap::CopyRemap;
use rustycad_tkbrep::brep_graph::layer::{AttachedGraph, BRepGraphLayer, LayerBase, as_layer};
use rustycad_tkbrep::brep_graph::layer_registry::LayerRegistry;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Guid, Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// Minimal concrete layer for testing the iterator.
struct TestLayer {
    transient: StandardTransient,
    base: LayerBase,
    id: Guid,
    name: AsciiString,
}
impl_transient!(TestLayer, "TestLayer", dyn BRepGraphLayer, transient,
    interfaces: [dyn BRepGraphLayer]);

impl TestLayer {
    fn new(id: Guid, name: &str) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: LayerBase::new(),
            id,
            name: AsciiString::new(name),
        }
    }
}

impl BRepGraphLayer for TestLayer {
    fn layer_base(&self) -> &LayerBase {
        &self.base
    }
    fn id(&self) -> Guid {
        self.id
    }
    fn name(&self) -> &AsciiString {
        &self.name
    }
    fn on_node_removed(&self, _attached: AttachedGraph<'_>, _node: NodeId) {}
    fn copy_to(&self, _copy: &CopyRemap<'_>) -> Result<(), Failure> {
        Ok(())
    }
    fn invalidate_all(&self, _attached: AttachedGraph<'_>) {}
    fn clear(&self, _attached: AttachedGraph<'_>) {}
}

fn guid(text: &str) -> Guid {
    Guid::from_ascii(text.as_bytes()).unwrap()
}

fn test_layer(id: &str, name: &str) -> Handle<dyn BRepGraphLayer> {
    as_layer(&Handle::new(TestLayer::new(guid(id), name)))
}

#[test]
fn BRepGraph_LayerIteratorTest__EmptyRegistry_IsEmpty() {
    let aRegistry = LayerRegistry::new();
    let anIt = LayerIterator::new(&aRegistry);
    assert!(!anIt.more());
    assert_eq!(anIt.nb_layers(), 0u32);
    assert!(aRegistry.layer(0).is_null());
}

#[test]
fn BRepGraph_LayerIteratorTest__SingleLayer_IteratesOnce() {
    let aRegistry = LayerRegistry::new();
    let aLayer = test_layer("aaaaaaaa-1111-2222-3333-444444444444", "TestA");
    aRegistry.register_layer(&aLayer).unwrap();

    let mut anIt = LayerIterator::new(&aRegistry);
    assert!(anIt.more());
    assert_eq!(anIt.nb_layers(), 1u32);
    assert_eq!(anIt.slot(), 0u32);
    assert!(!anIt.value().is_null());
    assert!(*anIt.value().get().unwrap().name() == AsciiString::new("TestA"));

    anIt.next_slot();
    assert!(!anIt.more());
}

#[test]
fn BRepGraph_LayerIteratorTest__MultipleLayers_IteratesAll() {
    let aRegistry = LayerRegistry::new();
    let aLayerA = test_layer("aaaaaaaa-1111-2222-3333-444444444444", "LayerA");
    let aLayerB = test_layer("bbbbbbbb-1111-2222-3333-444444444444", "LayerB");
    let aLayerC = test_layer("cccccccc-1111-2222-3333-444444444444", "LayerC");
    aRegistry.register_layer(&aLayerA).unwrap();
    aRegistry.register_layer(&aLayerB).unwrap();
    aRegistry.register_layer(&aLayerC).unwrap();

    let mut aCount = 0u32;
    let mut anIt = LayerIterator::new(&aRegistry);
    while anIt.more() {
        assert!(!anIt.value().is_null());
        assert_eq!(anIt.slot(), aCount);
        aCount += 1;
        anIt.next_slot();
    }
    assert_eq!(aCount, 3u32);
}

#[test]
fn BRepGraph_LayerIteratorTest__RangeFor_WorksCorrectly() {
    let aRegistry = LayerRegistry::new();
    let aLayerA = test_layer("aaaaaaaa-1111-2222-3333-444444444444", "LayerA");
    let aLayerB = test_layer("bbbbbbbb-1111-2222-3333-444444444444", "LayerB");
    aRegistry.register_layer(&aLayerA).unwrap();
    aRegistry.register_layer(&aLayerB).unwrap();

    let mut aCount = 0u32;
    for aLayer in LayerIterator::new(&aRegistry) {
        assert!(!aLayer.is_null());
        aCount += 1;
    }
    assert_eq!(aCount, 2u32);
}
