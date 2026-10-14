// SPDX-License-Identifier: GPL-2.0-only
//! Standalone owned-Rust reproducibility/ThreadSanitizer probe; no native dependencies.
#[path = "../../../crates/tkmeshvs/src/volume_topology.rs"]
mod volume_topology;
use volume_topology::{
    MeshVSTopologyError, MeshVSVolumeTopology, create_prism_topology, create_pyramid_topology,
};
type Factory = fn(i32) -> Result<MeshVSVolumeTopology, MeshVSTopologyError>;

fn render_connectivity(factory: Factory) -> String {
    let mut rendered = String::new();
    for count in 3..=64 {
        let topology = factory(count).unwrap();
        rendered.push_str(&format!("{count}"));
        for face_id in 1..=topology.face_count() as i32 {
            rendered.push_str(&format!(" {:?}", topology.face(face_id).unwrap()));
        }
        rendered.push('\n');
    }
    rendered
}

fn shared_read_stress() {
    let shared = create_prism_topology(64).unwrap();
    std::thread::scope(|scope| {
        for _ in 0..8 {
            let shared = &shared;
            scope.spawn(move || {
                for _ in 0..256 {
                    assert_eq!(&create_prism_topology(64).unwrap(), shared);
                    assert_eq!(shared.clone().face(1), shared.face(1));
                }
            });
        }
    });
}

fn main() {
    let factories: [Factory; 2] = [create_prism_topology, create_pyramid_topology];
    if std::env::args().nth(1).as_deref() == Some("parallel") {
        shared_read_stress();
        let workers =
            factories.map(|factory| std::thread::spawn(move || render_connectivity(factory)));
        for worker in workers {
            print!("{}", worker.join().unwrap());
        }
        return;
    }
    for factory in factories {
        print!("{}", render_connectivity(factory));
    }
}
