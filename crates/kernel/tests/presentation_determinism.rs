// SPDX-License-Identifier: GPL-2.0-only
//! Determinism and input isolation of `tessellate` (ADR 0077): byte-identical
//! results across calls, separately built bodies, threads and processes, and an
//! unchanged body afterwards.
mod support;
use rustycad_kernel::presentation::{DisplayOptions, tessellate};
use rustycad_kernel::topology::{BodyTopology, FaceIndex};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkmath::poly::mesh_purpose;
use rustycad_tkmesh::brep_mesh::BRepMeshIncrementalMesh;
use std::process::Command;
use support::bodies::{all_fixtures, torus_fixture};

const HELPER: &str = "print_fixture_mesh_digests";

fn options() -> DisplayOptions {
    DisplayOptions::new(0.02, 0.4).unwrap()
}

fn bytes_of(body: &Shape) -> Vec<u8> {
    tessellate(body, &options()).unwrap().canonical_bytes()
}

/// FNV-1a: a small owned digest so process outputs stay short.
fn digest(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn fixture_digests() -> String {
    let lines: Vec<String> = all_fixtures()
        .iter()
        .map(|fixture| format!("{} {:016x}", fixture.name, digest(&bytes_of(&fixture.body))))
        .collect();
    lines.join("\n")
}

#[test]
fn repeated_calls_and_separate_constructions_give_identical_bytes() {
    for (first, second) in all_fixtures().iter().zip(all_fixtures()) {
        let expected = bytes_of(&first.body);
        assert_eq!(bytes_of(&first.body), expected, "{} repeated", first.name);
        assert_eq!(bytes_of(&second.body), expected, "{} rebuilt", first.name);
    }
}

#[test]
fn eight_threads_on_one_shared_body_give_the_single_thread_bytes() {
    for fixture in all_fixtures() {
        let expected = bytes_of(&fixture.body);
        let results: Vec<Vec<u8>> = std::thread::scope(|scope| {
            let workers: Vec<_> = (0..8)
                .map(|_| scope.spawn(|| bytes_of(&fixture.body)))
                .collect();
            workers
                .into_iter()
                .map(|worker| worker.join().unwrap())
                .collect()
        });
        assert!(
            results.iter().all(|bytes| *bytes == expected),
            "{}",
            fixture.name
        );
    }
}

#[test]
#[ignore = "helper run in a child process by two_processes_print_identical_digests"]
fn print_fixture_mesh_digests() {
    println!("DIGESTS-BEGIN\n{}\nDIGESTS-END", fixture_digests());
}

fn child_digests() -> String {
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            HELPER,
            "--nocapture",
            "--test-threads=1",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    let begin = text.find("DIGESTS-BEGIN\n").unwrap() + "DIGESTS-BEGIN\n".len();
    text[begin..text.find("\nDIGESTS-END").unwrap()].to_string()
}

#[test]
fn two_processes_print_identical_digests() {
    let (first, second) = (child_digests(), child_digests());
    assert_eq!(first, second);
    assert_eq!(first, fixture_digests());
}

fn face_triangle_count(body: &Shape, index: u32) -> i32 {
    let topology = BodyTopology::enumerate(body).unwrap();
    let face = topology.face(FaceIndex(index)).unwrap();
    let (mesh, _) = BRepTool::triangulation(&face, mesh_purpose::NONE).unwrap();
    mesh.get().map_or(0, |mesh| mesh.nb_triangles())
}

#[test]
fn the_body_keeps_no_mesh_from_tessellation() {
    let fixture = torus_fixture();
    tessellate(&fixture.body, &options()).unwrap();
    assert_eq!(face_triangle_count(&fixture.body, 0), 0);
}

#[test]
fn an_existing_body_mesh_is_kept_and_does_not_change_the_result() {
    let fixture = torus_fixture();
    let expected = bytes_of(&torus_fixture().body);
    BRepMeshIncrementalMesh::with_shape(&fixture.body, 0.5, false, 1.0, false).unwrap();
    let coarse = face_triangle_count(&fixture.body, 0);
    assert!(coarse > 0);
    assert_eq!(bytes_of(&fixture.body), expected);
    assert_eq!(face_triangle_count(&fixture.body, 0), coarse);
}

#[test]
fn null_body_is_refused() {
    let error = tessellate(&Shape::default(), &options()).unwrap_err();
    assert_eq!(
        error.class,
        rustycad_kernel::query_error::BodyQueryFailure::NullBody
    );
}
