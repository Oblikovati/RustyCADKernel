//! TopOpeBRepDS probe operations, the Rust twin of tools/oracle/tkbool_probe_topopebrepds.cxx
//! (the records are documented there).
use super::common::{Reader, attempt, end, flag, int, text};
use rustycad_tkbool::top_ope_brep_ds::{
    TopOpeBRepDSConfig, TopOpeBRepDSKind, TopOpeBRepDSTransition, is_geometry, is_topology,
    kind_to_shape, shape_to_kind, sprint_config, sprint_kind, sprint_kind_index,
    sprint_orientation, sprint_shape_type_index, sprint_state,
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};

/// The kinds in their native order.
const KINDS: [TopOpeBRepDSKind; 12] = [
    TopOpeBRepDSKind::Point,
    TopOpeBRepDSKind::Curve,
    TopOpeBRepDSKind::Surface,
    TopOpeBRepDSKind::Vertex,
    TopOpeBRepDSKind::Edge,
    TopOpeBRepDSKind::Wire,
    TopOpeBRepDSKind::Face,
    TopOpeBRepDSKind::Shell,
    TopOpeBRepDSKind::Solid,
    TopOpeBRepDSKind::CompSolid,
    TopOpeBRepDSKind::Compound,
    TopOpeBRepDSKind::Unknown,
];

/// The configurations in their native order.
const CONFIGS: [TopOpeBRepDSConfig; 3] = [
    TopOpeBRepDSConfig::UnshGeometry,
    TopOpeBRepDSConfig::SameOriented,
    TopOpeBRepDSConfig::DiffOriented,
];

/// `bl_transition`.
fn transition(t: &TopOpeBRepDSTransition) {
    int(t.before() as i32);
    int(t.after() as i32);
    int(t.shape_before() as i32);
    int(t.shape_after() as i32);
    int(t.index_before());
    int(t.index_after());
}

/// `ds_transition sb sa onb ona ib ia o`.
pub fn transition_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (sb, sa) = (
        State::ALL[r.integer() as usize],
        State::ALL[r.integer() as usize],
    );
    let (onb, ona) = (
        ShapeEnum::ALL[r.integer() as usize],
        ShapeEnum::ALL[r.integer() as usize],
    );
    let (ib, ia) = (r.integer(), r.integer());
    let o = Orientation::ALL[r.integer() as usize];
    let mut t = TopOpeBRepDSTransition::new(sb, sa, onb, ona);
    t.set_index_before(ib);
    t.set_index_after(ia);
    for s in State::ALL {
        int(t.orientation(s, ShapeEnum::Face) as i32);
    }
    transition(&t.complement());
    flag(t.is_unknown());
    attempt(|| t.index().map(int));
    let mut c = t;
    c.set_orientation(o);
    transition(&c);
    transition(&TopOpeBRepDSTransition::from_orientation(o));
    end();
}

/// `ds_kind k i`.
pub fn kind_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (k, i) = (r.integer(), r.integer());
    let kind = KINDS[k as usize];
    text(sprint_kind(kind).as_bytes());
    text(sprint_kind_index(kind, i, "<", ">").as_bytes());
    flag(is_geometry(kind));
    flag(is_topology(kind));
    int(kind_to_shape(kind) as i32);
    if k <= 8 {
        let shape = ShapeEnum::ALL[k as usize];
        attempt(|| shape_to_kind(shape).map(|v| int(v as i32)));
        attempt(|| sprint_shape_type_index(shape, i).map(|v| text(v.as_bytes())));
    }
    if k < 4 {
        text(sprint_state(State::ALL[k as usize]).as_bytes());
        text(sprint_orientation(Orientation::ALL[k as usize]).as_bytes());
    }
    if k < 3 {
        text(sprint_config(CONFIGS[k as usize]).as_bytes());
    }
    end();
}
