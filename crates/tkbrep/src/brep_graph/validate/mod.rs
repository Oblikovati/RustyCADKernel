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
// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_Validate.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_Validate`: the structural invariant checker of a graph.
//!
//! A read-only algorithm over `&BRepGraph` that verifies cross-reference bounds,
//! relation symmetry, incidence references, representation ownership, removed-node
//! isolation, wire connectivity, active counts, the document roots, UID round trips
//! and the assembly and compound DAGs. [`Mode::Lightweight`] runs the mutation
//! boundary, the removed-node isolation and the document-root checks only;
//! [`Mode::Audit`] runs every check. The issues keep the native report order: each
//! check appends in its native traversal order, and the checks run in the native
//! sequence.
mod bounds;
mod counts;
mod cycles;
mod geometry;
mod identity;
mod isolation;
mod owned_uses;
mod ownership;
mod relations;

use crate::brep_graph::editor::BoundaryIssue;
use crate::brep_graph::{BRepGraph, NodeId};
use rustycad_tkernel::tcollection::AsciiString;

/// `BRepGraph_Validate::Severity`: the level of a reported issue.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Severity {
    /// `Warning`: a diagnostic that does not make the graph invalid.
    #[default]
    Warning,
    /// `Error`: a broken invariant.
    Error,
}

/// `BRepGraph_Validate::Mode`: the depth of the checks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Mode {
    /// `Lightweight`: the boundary-oriented checks for frequent validation points.
    #[default]
    Lightweight,
    /// `Audit`: the full structural audit (a superset of `Lightweight`).
    Audit,
}

/// `BRepGraph_Validate::Issue`: one structural issue.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Issue {
    /// `Sev`.
    pub sev: Severity,
    /// `NodeId`: the node concerned (the invalid id for a graph-wide issue).
    pub node_id: NodeId,
    /// `Description`.
    pub description: AsciiString,
}

/// `BRepGraph_Validate::Result`: the issues in report order.
///
/// ```
/// use rustycad_tkbrep::brep_graph::validate::{Severity, ValidationResult};
/// let result = ValidationResult::default();
/// assert!(result.is_valid());
/// assert_eq!(result.nb_issues(Severity::Error), 0);
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ValidationResult {
    /// `Issues`.
    pub issues: Vec<Issue>,
}

impl ValidationResult {
    /// `IsValid()`: no Error-level issue was found.
    pub fn is_valid(&self) -> bool {
        self.issues.iter().all(|issue| issue.sev != Severity::Error)
    }

    /// `NbIssues(theSev)`: the issues of one severity.
    pub fn nb_issues(&self, sev: Severity) -> i32 {
        self.issues.iter().filter(|issue| issue.sev == sev).count() as i32
    }
}

/// `BRepGraph_Validate::Options`: the validation profile.
///
/// ```
/// use rustycad_tkbrep::brep_graph::validate::{Mode, Options};
/// assert_eq!(Options::default().validation_mode, Mode::Lightweight);
/// assert_eq!(Options::audit().validation_mode, Mode::Audit);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Options {
    /// `ValidationMode`: `Lightweight` by default.
    pub validation_mode: Mode,
}

impl Options {
    /// `Options::Lightweight()`.
    pub fn lightweight() -> Self {
        Self {
            validation_mode: Mode::Lightweight,
        }
    }

    /// `Options::Audit()`.
    pub fn audit() -> Self {
        Self {
            validation_mode: Mode::Audit,
        }
    }
}

/// `Perform(theGraph)`: the lightweight checks.
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// use rustycad_tkbrep::brep_graph::validate;
/// assert!(validate::perform(&BRepGraph::new()).is_valid());
/// ```
pub fn perform(graph: &BRepGraph) -> ValidationResult {
    perform_with_options(graph, &Options::lightweight())
}

/// `Perform(theGraph, theMode)`.
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// use rustycad_tkbrep::brep_graph::validate::{self, Mode};
/// assert!(validate::perform_with_mode(&BRepGraph::new(), Mode::Audit).issues.is_empty());
/// ```
pub fn perform_with_mode(graph: &BRepGraph, mode: Mode) -> ValidationResult {
    perform_with_options(
        graph,
        &Options {
            validation_mode: mode,
        },
    )
}

/// `Perform(theGraph, theOptions)`: an empty graph has no issue.
///
/// ```
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// use rustycad_tkbrep::brep_graph::validate::{self, Options};
/// assert!(validate::perform_with_options(&BRepGraph::new(), &Options::audit()).is_valid());
/// ```
pub fn perform_with_options(graph: &BRepGraph, options: &Options) -> ValidationResult {
    let mut result = ValidationResult::default();
    if graph.is_empty() {
        return result;
    }
    append_mutation_boundary_issues(graph, &mut result.issues);
    if options.validation_mode == Mode::Lightweight {
        isolation::check_removed_node_isolation(graph, &mut result.issues);
        counts::check_document_root_products(graph, &mut result.issues);
        return result;
    }
    run_audit(graph, &mut result.issues);
    result
}

// The audit sequence after the mutation boundary, in the native order.
fn run_audit(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    bounds::check_cross_reference_bounds(graph, issues);
    relations::check_relation_consistency(graph, issues);
    relations::check_relation_face_count_cache(graph, issues);
    relations::check_incidence_ref_consistency(graph, issues);
    geometry::check_geometry_references(graph, issues);
    owned_uses::check_owned_use_references(graph, issues);
    isolation::check_removed_node_isolation(graph, issues);
    isolation::check_wire_connectivity(graph, issues);
    counts::check_active_counts(graph, issues);
    counts::check_document_root_products(graph, issues);
    identity::check_topo_supplement(graph, issues);
    identity::check_uids(graph, issues);
    cycles::check_assembly_cycles(graph, issues);
    cycles::check_compound_cycles(graph, issues);
    ownership::check_ref_ownership(graph, issues);
    ownership::check_occurrence_product_liveness(graph, issues);
    ownership::check_cross_container_ownership(graph, issues);
}

// `appendMutationBoundaryIssues`: the editor's boundary issues as errors.
fn append_mutation_boundary_issues(graph: &BRepGraph, issues: &mut Vec<Issue>) {
    let mut boundary: Vec<BoundaryIssue> = Vec::new();
    if graph
        .editor_state()
        .validate_mutation_boundary(Some(&mut boundary))
    {
        return;
    }
    for issue in boundary {
        issues.push(Issue {
            sev: Severity::Error,
            node_id: issue.node_id,
            description: issue.description,
        });
    }
}

// Appends one issue (the shared body of the native `Issues.Append(Issue{...})`).
fn report(issues: &mut Vec<Issue>, sev: Severity, node: impl Into<NodeId>, text: &str) {
    issues.push(Issue {
        sev,
        node_id: node.into(),
        description: AsciiString::from(text),
    });
}

// `isValidNodeId`: a valid id inside its kind's range.
fn is_valid_node_id(graph: &BRepGraph, node: impl Into<NodeId>) -> bool {
    graph.topo().generic().is_valid(node.into())
}

// `isEntityRemoved`: a definition exists for the id and it is soft-removed.
fn is_entity_removed(graph: &BRepGraph, node: impl Into<NodeId>) -> bool {
    let node = node.into();
    graph.topo_entity(node).is_some() && node.is_removed(graph)
}
