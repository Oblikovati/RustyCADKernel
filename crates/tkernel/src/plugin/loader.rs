// Created on: 1997-03-06
// Created by: Mister rmi
// Copyright (c) 1997-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKernel/Plugin/Plugin.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Plugin::Load`: resolves a GUID through the "Plugin" resource file to a library and
//! calls its factory.
use super::failure;
use super::libraries::{PluginFactory, PluginLibraries, library_file_name};
use crate::resource::{
    NativeResourceConsole, ResourceEnvironment, ResourceHost, ResourceIo, ResourceManager,
};
use crate::standard::{Failure, Guid};
use std::collections::BTreeMap;
use std::io::Write;
use std::rc::Rc;

/// Loads plugin products by GUID; `loader.load(&guid, true)?`.
///
/// Native code keeps one process-wide cache of factories; here the cache belongs to the
/// loader, whose resource access, libraries and console are injected.
pub struct PluginLoader<P> {
    io: Rc<dyn ResourceIo>,
    environment: Rc<dyn ResourceEnvironment>,
    libraries: Box<dyn PluginLibraries<P>>,
    console: Box<dyn Write>,
    factories: BTreeMap<String, PluginFactory<P>>,
}

impl<P> PluginLoader<P> {
    /// Builds a loader; pass `NativeResourceIo`/`NativeResourceEnvironment` and `std::io::stdout()`
    /// for native behavior; `PluginLoader::new(io, environment, Box::new(libraries), Box::new(stdout()))`.
    pub fn new(
        io: Rc<dyn ResourceIo>,
        environment: Rc<dyn ResourceEnvironment>,
        libraries: Box<dyn PluginLibraries<P>>,
        console: Box<dyn Write>,
    ) -> Self {
        Self {
            io,
            environment,
            libraries,
            console,
            factories: BTreeMap::new(),
        }
    }

    /// `Plugin::Load`: returns what the GUID's factory makes, reading `<GUID>.Location` from
    /// the "Plugin" resources the first time; `loader.load(&guid, false)?`.
    ///
    /// Raises Plugin_Failure with the native messages when the resource is missing, the
    /// library does not open or has no factory; with `verbose`, the first two are also
    /// printed to the console. Failures are not cached, successes are.
    pub fn load(&mut self, guid: &Guid, verbose: bool) -> Result<P, Failure> {
        let id = guid_text(guid);
        let cached = self.factories.get(&id).cloned();
        let factory = match cached {
            Some(factory) => factory,
            None => self.resolve(&id, verbose)?,
        };
        Ok(factory(guid))
    }

    fn resolve(&mut self, id: &str, verbose: bool) -> Result<PluginFactory<P>, Failure> {
        let name = self.library_name(id, verbose)?;
        let file_name = library_file_name(&name);
        if let Err(reason) = self.libraries.open(&file_name) {
            self.say(
                verbose,
                &format!("could not open: {name} ; reason: {reason}\n"),
            );
            return Err(failure(&format!("could not open:{name}; reason:{reason}")));
        }
        let factory = self
            .libraries
            .factory(&file_name)
            .map_err(|reason| failure(&format!("could not find the factory in:{name}{reason}")))?;
        self.factories.insert(id.to_owned(), factory.clone());
        Ok(factory)
    }

    /// The `<id>.Location` value of a freshly read "Plugin" resource catalog.
    fn library_name(&mut self, id: &str, verbose: bool) -> Result<String, Failure> {
        let resource = format!("{id}.Location");
        // `new Resource_Manager("Plugin")`: its traces, when enabled, go to std::cout.
        let host = ResourceHost::new(
            Rc::clone(&self.io),
            Rc::clone(&self.environment),
            Rc::new(NativeResourceConsole),
        );
        let catalog = ResourceManager::new("Plugin", false, host)?;
        if let Ok(value) = catalog.value(&resource) {
            return Ok(c_text(value.as_bytes()));
        }
        let message = format!("could not find the resource:{resource}\n");
        self.say(verbose, &message);
        Err(failure(&message))
    }

    /// Writes a diagnostic to the console; console failures are ignored, as `std::cout` is.
    fn say(&mut self, verbose: bool, text: &str) {
        if verbose {
            let _ = self.console.write_all(text.as_bytes());
        }
    }
}

/// `Standard_GUID::ToCString` text; `guid_text(&guid).len() == 36`.
fn guid_text(guid: &Guid) -> String {
    c_text(&guid.to_c_string())
}

/// Text of a C string (up to its first NUL); `c_text(b"ab\0c") == "ab"`.
fn c_text(bytes: &[u8]) -> String {
    let text = bytes.split(|byte| *byte == 0).next().unwrap_or_default();
    String::from_utf8_lossy(text).into_owned()
}
