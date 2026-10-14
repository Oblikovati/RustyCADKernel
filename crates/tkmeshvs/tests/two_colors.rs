// SPDX-License-Identifier: GPL-2.0-only
//! Native fixtures are immutable input; these tests need only Rust.
use martensite_tkmeshvs::{MeshVSTwoColors, bind_two_colors, extract_color, extract_colors};
use rustycad_tkernel::{
    ncollection::{HashKey, Map},
    quantity::{Color, ColorSpace},
};

#[test]
fn native_packing_hash_equality_and_extraction_replay() {
    let lines: Vec<&str> = include_str!("fixtures/two-colors-native.txt")
        .lines()
        .collect();
    assert_eq!(lines.len(), 774);
    for line in &lines[..768] {
        let expected = numbers(line);
        let mut pair = fixture_pair(expected[0], expected[1] as u8);
        let mut actual = vec![expected[0], expected[1]];
        actual.extend(pair_record(&mut pair));
        assert_eq!(actual, expected, "native mode/index={:?}", &expected[..2]);
    }
}

fn numbers(line: &str) -> Vec<u64> {
    line.split_whitespace()
        .map(|word| word.parse().unwrap())
        .collect()
}

fn pair_record(pair: &mut MeshVSTwoColors) -> Vec<u64> {
    let mut actual = [pair.r1, pair.g1, pair.b1, pair.r2, pair.g2, pair.b2]
        .map(u64::from)
        .to_vec();
    actual.push(pair.occt_hash() as u64);
    actual.extend(color_bits(extract_color(pair, 1)));
    actual.extend(color_bits(extract_color(pair, 2)));
    let (mut first, mut second) = (Color::default(), Color::default());
    extract_colors(pair, &mut first, &mut second);
    actual.extend(color_bits(first));
    actual.extend(color_bits(second));
    let reversed = raw_pair([pair.r2, pair.g2, pair.b2, pair.r1, pair.g1, pair.b1]);
    let copied = *pair;
    actual.extend([u64::from(*pair == reversed), u64::from(*pair == copied)]);
    actual
}

fn raw_pair([r1, g1, b1, r2, g2, b2]: [u8; 6]) -> MeshVSTwoColors {
    MeshVSTwoColors {
        r1,
        g1,
        b1,
        r2,
        g2,
        b2,
    }
}

fn fixture_pair(mode: u64, index: u8) -> MeshVSTwoColors {
    if mode == 2 {
        return raw_fixture(index);
    }
    let space = if mode == 0 {
        ColorSpace::Srgb
    } else {
        ColorSpace::Rgb
    };
    let [first, second] = fixture_components(index).map(|rgb| fixture_color(rgb, space));
    bind_two_colors(first, second)
}

fn raw_fixture(index: u8) -> MeshVSTwoColors {
    raw_pair([
        index,
        255 - index,
        index.wrapping_mul(17),
        index.wrapping_mul(37),
        index.wrapping_mul(73),
        index,
    ])
}

fn fixture_components(index: u8) -> [[f64; 3]; 2] {
    let value = f64::from(index) / 255.0;
    let [green, blue, second_blue] =
        [17, 37, 73].map(|factor| f64::from(index.wrapping_mul(factor)) / 255.0);
    [[value, green, blue], [1.0 - value, value, second_blue]]
}

fn fixture_color([red, green, blue]: [f64; 3], space: ColorSpace) -> Color {
    Color::new(red, green, blue, space).unwrap()
}

fn color_bits(color: Color) -> [u64; 3] {
    color
        .values(ColorSpace::Rgb)
        .map(|value| u64::from((value as f32).to_bits()))
}

#[test]
fn invalid_indices_preserve_pair_and_return_native_default() {
    let mut pair = fixture_pair(2, 123);
    let before = pair;
    for line in include_str!("fixtures/two-colors-native.txt")
        .lines()
        .skip(768)
        .take(5)
    {
        let words: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(words[0], "invalid");
        let index: i32 = words[1].parse().unwrap();
        let expected = numbers(&words[2..].join(" "));
        assert_eq!(
            color_bits(extract_color(&mut pair, index)).as_slice(),
            expected
        );
        assert_eq!(pair, before);
    }
}

#[test]
fn native_aliased_outputs_write_the_second_color_last() {
    use martensite_tkmeshvs::{MeshVSColorOutputs, extract_colors_into};
    let mut pair = raw_pair([1, 2, 3, 4, 5, 6]);
    let original = pair;
    let mut output = Color::default();
    extract_colors_into(&mut pair, MeshVSColorOutputs::Aliased(&mut output));
    let line = include_str!("fixtures/two-colors-native.txt")
        .lines()
        .last()
        .unwrap();
    assert_eq!(line.split_whitespace().next(), Some("alias"));
    let expected = numbers(line.strip_prefix("alias ").unwrap());
    assert_eq!(color_bits(output).as_slice(), expected);
    assert_eq!(pair, original);
}

#[test]
fn each_channel_controls_ordered_equality_and_map_membership() {
    let original = raw_pair([1, 2, 3, 4, 5, 6]);
    let different = single_channel_changes(original);
    let mut map = Map::<MeshVSTwoColors>::default();
    assert!(map.add(original));
    assert!(!map.add(original));
    for value in different {
        assert_ne!(value, original);
        assert!(map.add(value));
    }
    assert_eq!(map.extent(), 7);
    assert!(map.remove(&original));
    assert!(!map.contains(&original));
    for value in different {
        assert!(map.contains(&value));
    }
}

fn single_channel_changes(original: MeshVSTwoColors) -> [MeshVSTwoColors; 6] {
    [
        MeshVSTwoColors { r1: 7, ..original },
        MeshVSTwoColors { g1: 7, ..original },
        MeshVSTwoColors { b1: 7, ..original },
        MeshVSTwoColors { r2: 7, ..original },
        MeshVSTwoColors { g2: 7, ..original },
        MeshVSTwoColors { b2: 7, ..original },
    ]
}
