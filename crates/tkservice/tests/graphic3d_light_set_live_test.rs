//! Rust-only replay of pinned CPU oracle transitions; native code is not a test dependency.
use rustycad_tkernel::quantity::{Color, ColorSpace};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::{
    Graphic3dCLight, Graphic3dLightSet, Graphic3dLightSetIterationFilter as Filter,
    Graphic3dLightSetIterator as Cursor, Graphic3dTypeOfLightSource as Kind,
};

const NATIVE: &str = concat!(
    include_str!("fixtures/light_set_live/native-0.txt"),
    include_str!("fixtures/light_set_live/native-2.txt"),
    include_str!("fixtures/light_set_live/native-4.txt"),
    include_str!("fixtures/light_set_live/native-8.txt"),
    include_str!("fixtures/light_set_live/native-6.txt"),
    include_str!("fixtures/light_set_live/native-14.txt"),
);
const EVENTS: &str = include_str!("fixtures/light_set_live/events.txt");
const KINDS: [Kind; 4] = [
    Kind::Ambient,
    Kind::Directional,
    Kind::Positional,
    Kind::Spot,
];

struct LiveReplay {
    set: Handle<Graphic3dLightSet>,
    sources: [Handle<Graphic3dCLight>; 6],
    cursor: Cursor,
    other: Cursor,
    filter: Filter,
}

impl LiveReplay {
    fn new(filter: Filter) -> Self {
        let sources = replay_sources();
        Self {
            set: Handle::new(Graphic3dLightSet::default()),
            sources,
            cursor: Cursor::default(),
            other: Cursor::default(),
            filter,
        }
    }

    fn ordinal(&self, source: &Handle<Graphic3dCLight>) -> i32 {
        self.sources
            .iter()
            .position(|light| light == source)
            .map_or(-1, |id| id as i32)
    }

    fn source_action(&self, action: &str, id: i32, value: i32) -> Result<i32, Failure> {
        let source = if id < 0 {
            Handle::default()
        } else {
            self.sources[id as usize].clone()
        };
        match action {
            "add" => return self.set.add(&source).map(i32::from),
            "remove" => return Ok(i32::from(self.set.remove(&source))),
            "contains" => return Ok(i32::from(self.set.contains(&source))),
            "enable" => source.set_enabled(value != 0),
            "shadow" => source.set_cast_shadows(value != 0)?,
            "intensity" => source.set_intensity(value as f32)?,
            "color" => source.set_color(self.sources[value as usize].color()),
            _ => panic!("unknown action {action}; expected a source mutation"),
        }
        Ok(0)
    }

    fn apply(&mut self, action: &str, id: i32, value: i32) -> Result<i32, Failure> {
        match action {
            "start" => self.cursor = Cursor::new(&self.set, self.filter),
            "handle" => self.cursor = Cursor::from_handle(&self.set, self.filter),
            "null" => self.cursor = Cursor::from_handle(&Handle::default(), self.filter),
            "default" => self.cursor = Cursor::default(),
            "next" => self.cursor.next(),
            "skip" => self.cursor.skip_filtered(),
            "copy" => self.other = self.cursor.clone(),
            "othernext" => self.other.next(),
            "update" => return Ok(self.set.update_revision() as i32),
            "index" => return self.set.value(id).map(|source| self.ordinal(&source)),
            _ => return self.source_action(action, id, value),
        }
        Ok(0)
    }

    fn cursor_tokens(&self, cursor: &Cursor) -> [String; 2] {
        let value = cursor
            .value()
            .map(|source| self.ordinal(&source))
            .unwrap_or_else(exception_code);
        [i32::from(cursor.more()).to_string(), value.to_string()]
    }

    fn snapshot(&self, result: i32) -> Vec<String> {
        let mut tokens = vec![result.to_string()];
        tokens.extend(self.cursor_tokens(&self.cursor));
        tokens.extend(self.cursor_tokens(&self.other));
        tokens.push(self.set.revision().to_string());
        tokens.push(self.set.extent().to_string());
        tokens.extend(self.cache_tokens());
        tokens.push(ascii_token(self.set.key_enabled_long().as_bytes()));
        tokens.push(ascii_token(self.set.key_enabled_short().as_bytes()));
        tokens.push(self.membership_token());
        tokens
    }

    fn cache_tokens(&self) -> Vec<String> {
        let mut tokens = Vec::new();
        for kind in KINDS {
            tokens.push(self.set.nb_lights_of_type(kind).to_string());
            tokens.push(self.set.nb_enabled_lights_of_type(kind).to_string());
        }
        tokens.push(self.set.nb_enabled().to_string());
        tokens.push(self.set.nb_cast_shadows().to_string());
        let ambient = self.set.ambient_color();
        tokens
            .extend([ambient.x, ambient.y, ambient.z, ambient.w].map(|v| v.to_bits().to_string()));
        tokens
    }

    fn membership_token(&self) -> String {
        let ids: Vec<String> = (self.set.lower()..=self.set.upper())
            .map(|index| self.ordinal(&self.set.value(index).unwrap()).to_string())
            .collect();
        if ids.is_empty() {
            "-".to_owned()
        } else {
            ids.join(",")
        }
    }
}

fn replay_sources() -> [Handle<Graphic3dCLight>; 6] {
    let sources = std::array::from_fn(|id| {
        let source = Handle::new(Graphic3dCLight::new(KINDS[id % 4]));
        source.set_intensity((id + 1) as f32).unwrap();
        source
    });
    sources[0].set_color(Color::new(0.25, 0.5, 0.75, ColorSpace::Rgb).unwrap());
    sources[4].set_color(Color::new(0.75, 0.25, 0.5, ColorSpace::Rgb).unwrap());
    sources[1].set_cast_shadows(true).unwrap();
    sources[3].set_cast_shadows(true).unwrap();
    sources[2].set_enabled(false);
    sources
}

fn ascii_token(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return "-".to_owned();
    }
    String::from_utf8(bytes.to_vec()).unwrap()
}

fn exception_code(error: Failure) -> i32 {
    match error.exception_type() {
        "Standard_NoSuchObject" => -11,
        "Standard_OutOfRange" => -12,
        "Standard_ProgramError" => -13,
        "Standard_NotImplemented" => -14,
        other => panic!("exception {other}; expected a fixture exception type"),
    }
}

fn replay_filter(filter: Filter) {
    let mut replay = LiveReplay::new(filter);
    let rows = NATIVE
        .lines()
        .filter(|row| row.split_whitespace().next().unwrap() == (filter as i32).to_string());
    for (event, row) in EVENTS.lines().zip(rows) {
        replay_row(&mut replay, event, row);
    }
}

fn replay_row(replay: &mut LiveReplay, event: &str, row: &str) {
    let fields: Vec<&str> = row.split_whitespace().collect();
    assert_eq!(event, fields[1..4].join(" "));
    let result = replay
        .apply(
            fields[1],
            fields[2].parse().unwrap(),
            fields[3].parse().unwrap(),
        )
        .unwrap_or_else(exception_code);
    assert_eq!(
        replay.snapshot(result).join(" "),
        fields[4..].join(" "),
        "filter {:?}, event {event}",
        replay.filter
    );
}

#[test]
fn indexed_membership_filters_exhaustion_and_cache_match_native() {
    for filter in [
        Filter::None,
        Filter::ExcludeAmbient,
        Filter::ExcludeDisabled,
        Filter::ExcludeNoShadow,
        Filter::ExcludeDisabledAndAmbient,
        Filter::ActiveShadowCasters,
    ] {
        replay_filter(filter);
    }
    assert_eq!(NATIVE.lines().count(), 6 * EVENTS.lines().count());
}

#[test]
fn iterator_retains_storage_after_the_set_wrapper_is_dropped() {
    let source = Handle::new(Graphic3dCLight::new(Kind::Directional));
    let cursor = {
        let set = Graphic3dLightSet::default();
        set.add(&source).unwrap();
        Cursor::new(&set, Filter::None)
    };
    assert!(cursor.more());
    assert_eq!(cursor.value().unwrap(), source);
}

#[test]
fn cloned_cursor_indices_are_independent_and_membership_is_shared() {
    let set = Graphic3dLightSet::default();
    let first = Handle::new(Graphic3dCLight::new(Kind::Ambient));
    let last = Handle::new(Graphic3dCLight::new(Kind::Spot));
    set.add(&first).unwrap();
    let mut cursor = Cursor::new(&set, Filter::None);
    let copied = cursor.clone();
    cursor.next();
    set.add(&last).unwrap();
    assert_eq!(cursor.value().unwrap(), last);
    assert_eq!(copied.value().unwrap(), first);
    set.remove(&first);
    assert!(!cursor.more());
    assert_eq!(copied.value().unwrap(), last);
}

#[test]
fn failures_include_the_bad_index_and_expected_shape() {
    let set = Graphic3dLightSet::default();
    let failure = set.value(i32::MIN).err().unwrap();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert!(
        failure
            .message()
            .contains("index -2147483648; expected 1..0")
    );
    let cursor = Cursor::new(&set, Filter::None);
    let exhausted = cursor.value().err().unwrap();
    assert_eq!(exhausted.exception_type(), "Standard_NoSuchObject");
    assert!(exhausted.message().contains("index 1; expected 1..=0"));
}
