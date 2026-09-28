//! Quantity probe operations (twin of `tools/oracle/tkernel_quantity_probe.cxx`).
//!
//! Floats travel as 8-digit hex bits, doubles as 16-digit hex bits; a raised failure is
//! printed as `raise <type> <message>` inside the OK line, as the native operation does.
use rustycad_tkernel::{
    ncollection::{HashKey, Vec3, Vec4},
    quantity::{Color, ColorRgba, ColorSpace, Date, NamedColor, Period},
    standard::{Failure, dump::DumpStream},
    tcollection::AsciiString,
};
use std::str::SplitWhitespace;

type Input<'a, 'b> = &'a mut SplitWhitespace<'b>;

/// The output of a Quantity operation, or `None` when `op` belongs to another family.
pub fn evaluate(op: &str, input: Input<'_, '_>) -> Option<String> {
    Some(match op {
        "qc_named" => named(input),
        "qc_ctor" => constructed(input),
        "qc_vec3" => raised(
            Color::from_vec3(Vec3::new(f(input), f(input), f(input))),
            color_out,
        ),
        "qc_name" => raised(Color::nearest_name(d(input), d(input), d(input)), |name| {
            name.index().to_string()
        }),
        "qc_from_name" => from_name(&token(input)),
        "qc_hex" => hex(&token(input)),
        "qc_argb" => color_out(Color::from_argb(input.next()?.parse().ok()?)),
        "qc_hls" => hls([f(input), f(input), f(input)]),
        "qc_srgb_hls" => srgb_hls([f(input), f(input), f(input)]),
        "qc_lab" => lab([f(input), f(input), f(input)]),
        "qc_gamma" => gamma(
            [f(input), f(input), f(input)],
            [d(input), d(input), d(input)],
        ),
        "qc_pair" => pair(rgb(input), rgb(input)),
        "qc_change" => change(rgb(input), d(input)),
        "qc_epsilon" => epsilon(d(input)),
        "qc_json" => color_json(&rest(input)),
        "qrgba_ctor" => rgba_constructed([f(input), f(input), f(input), f(input)]),
        "qrgba_pair" => rgba_pair(rgba(input), rgba(input)),
        "qrgba_gamma" => rgba_gamma([f(input), f(input), f(input), f(input)]),
        "qrgba_json" => rgba_json(&rest(input)),
        "qperiod_ops" => period_ops(input),
        "qdate_ops" => date_ops(input),
        _ => return None,
    })
}

fn f(input: Input<'_, '_>) -> f32 {
    f32::from_bits(u32::from_str_radix(input.next().expect("float bits"), 16).expect("hex"))
}
fn d(input: Input<'_, '_>) -> f64 {
    f64::from_bits(u64::from_str_radix(input.next().expect("double bits"), 16).expect("hex"))
}
fn int(input: Input<'_, '_>) -> i32 {
    input.next().expect("integer").parse().expect("decimal")
}
fn token(input: Input<'_, '_>) -> String {
    let text = input.next().expect("token");
    if text == "EMPTY" { "" } else { text }.to_string()
}
/// The rest of the line; the corpus separates its tokens by single spaces.
fn rest(input: Input<'_, '_>) -> String {
    input.collect::<Vec<_>>().join(" ")
}
fn fx(value: f32) -> String {
    format!("{:08x}", value.to_bits())
}
fn dx(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}
fn vec3(values: [f32; 3]) -> String {
    values.map(fx).join(":")
}
fn dvec3(values: [f64; 3]) -> String {
    values.map(dx).join(":")
}
fn ascii(text: &AsciiString) -> String {
    String::from_utf8_lossy(text.as_bytes()).into_owned()
}
fn bit(value: bool) -> u8 {
    u8::from(value)
}
fn raise(failure: &Failure) -> String {
    format!("raise {} {}", failure.exception_type(), failure.message())
}
fn raised<T>(result: Result<T, Failure>, show: impl FnOnce(T) -> String) -> String {
    result.map_or_else(|failure| raise(&failure), show)
}
fn rgb(input: Input<'_, '_>) -> Color {
    Color::new(d(input), d(input), d(input), ColorSpace::Rgb).expect("valid corpus color")
}
fn rgba(input: Input<'_, '_>) -> ColorRgba {
    ColorRgba::new(f(input), f(input), f(input), f(input)).expect("valid corpus color")
}
fn dump(write: impl FnOnce(&mut DumpStream) -> std::io::Result<()>) -> String {
    let mut stream = DumpStream::default();
    write(&mut stream).expect("in-memory dump");
    stream.text().to_string()
}

const SPACES: [ColorSpace; 5] = [
    ColorSpace::Rgb,
    ColorSpace::Srgb,
    ColorSpace::Hls,
    ColorSpace::Lab,
    ColorSpace::Lch,
];

fn color_out(color: Color) -> String {
    let values: Vec<String> = SPACES
        .iter()
        .map(|space| dvec3(color.values(*space)))
        .collect();
    format!(
        "{} {} {} {} {} {} {} {} {} {} [{}]",
        vec3(color.rgb()),
        values.join(" "),
        dx(color.hue()),
        dx(color.light()),
        dx(color.saturation()),
        color.name().index(),
        ascii(&Color::to_hex(color, true)),
        ascii(&Color::to_hex(color, false)),
        Color::to_argb(color),
        color.occt_hash(),
        dump(|stream| color.dump_json(stream))
    )
}

fn rgba_out(color: ColorRgba) -> String {
    format!(
        "{} {} {} {} {} [{}]",
        vec3(color.rgb().rgb()),
        fx(color.alpha()),
        ascii(&ColorRgba::to_hex(color, true)),
        ascii(&ColorRgba::to_hex(color, false)),
        color.occt_hash(),
        dump(|stream| color.dump_json(stream))
    )
}

fn named(input: Input<'_, '_>) -> String {
    let name = NamedColor::from_index(int(input)).expect("valid corpus index");
    let color = Color::named(name);
    format!(
        "{} {} {}",
        Color::string_name(name),
        color_out(color),
        rgba_out(ColorRgba::from_color(color))
    )
}

fn constructed(input: Input<'_, '_>) -> String {
    let space = SPACES[usize::try_from(int(input)).expect("type index")];
    raised(Color::new(d(input), d(input), d(input), space), color_out)
}

fn from_name(text: &str) -> String {
    let white = Color::named(NamedColor::WHITE);
    let name = NamedColor::from_name(text);
    let color = Color::from_name(text);
    let rgba = ColorRgba::from_name(text);
    let shown = rgba.unwrap_or(ColorRgba::from_rgb(white, 0.25));
    format!(
        "{} {} {} {} {} {} {}",
        bit(name.is_some()),
        name.unwrap_or(NamedColor::WHITE).index(),
        bit(color.is_some()),
        vec3(color.unwrap_or(white).rgb()),
        bit(rgba.is_some()),
        vec3(shown.rgb().rgb()),
        fx(shown.alpha())
    )
}

fn hex(text: &str) -> String {
    let color = Color::from_hex(text);
    let mut out = format!(
        "{} {}",
        bit(color.is_some()),
        vec3(color.unwrap_or(Color::named(NamedColor::WHITE)).rgb())
    );
    for alpha_off in [false, true] {
        let mut rgba = ColorRgba::from_rgb(Color::named(NamedColor::WHITE), 0.25);
        let done = rgba.set_hex(text, alpha_off);
        out += &format!(
            " {} {} {}",
            bit(done),
            vec3(rgba.rgb().rgb()),
            fx(rgba.alpha())
        );
    }
    out
}

fn hls(values: [f32; 3]) -> String {
    let srgb = match Color::hls_to_srgb(values) {
        Ok(srgb) => srgb,
        Err(failure) => return raise(&failure),
    };
    let linear = Color::hls_to_linear_rgb(values).expect("sRGB conversion succeeded");
    let [h, l, s] = values.map(f64::from);
    let doubles = Color::hls_rgb(h, l, s).expect("sRGB conversion succeeded");
    format!("{} {} {}", vec3(srgb), vec3(linear), dvec3(doubles))
}

fn srgb_hls(values: [f32; 3]) -> String {
    let [r, g, b] = values.map(f64::from);
    format!(
        "{} {} {}",
        vec3(Color::srgb_to_hls(values)),
        vec3(Color::linear_rgb_to_hls(values)),
        dvec3(Color::rgb_hls(r, g, b))
    )
}

fn lab(values: [f32; 3]) -> String {
    format!(
        "{} {} {} {}",
        vec3(Color::lab_to_linear_rgb(values)),
        vec3(Color::lab_to_lch(values)),
        vec3(Color::lch_to_lab(values)),
        vec3(Color::linear_rgb_to_lab(values))
    )
}

fn gamma(floats: [f32; 3], doubles: [f64; 3]) -> String {
    [
        vec3(Color::linear_to_srgb_vec3_f32(floats)),
        vec3(Color::srgb_to_linear_vec3_f32(floats)),
        vec3(Color::linear_to_srgb_approx22_vec3(floats)),
        vec3(Color::srgb_to_linear_approx22_vec3(floats)),
        fx(Color::linear_to_srgb_approx22(floats[0])),
        fx(Color::srgb_to_linear_approx22(floats[0])),
        fx(Color::linear_to_srgb_f32(floats[0])),
        fx(Color::srgb_to_linear_f32(floats[0])),
        dx(Color::linear_to_srgb(doubles[0])),
        dx(Color::srgb_to_linear(doubles[0])),
        dvec3(Color::linear_to_srgb_vec3(doubles)),
        dvec3(Color::srgb_to_linear_vec3(doubles)),
    ]
    .join(" ")
}

fn pair(first: Color, second: Color) -> String {
    let (dc, di) = first.delta(second);
    format!(
        "{} {} {}{}{}{} {} {} {}",
        dx(first.distance(second)),
        dx(first.square_distance(second)),
        bit(first.is_equal(second)),
        bit(first.is_different(second)),
        bit(first == second),
        bit(first != second),
        dx(dc),
        dx(di),
        dx(first.delta_e2000(second))
    )
}

fn change(color: Color, delta: f64) -> String {
    let (mut intensity, mut contrast) = (color, color);
    intensity.change_intensity(delta).expect("valid HLS");
    contrast.change_contrast(delta).expect("valid HLS");
    format!("{} {}", vec3(intensity.rgb()), vec3(contrast.rgb()))
}

fn epsilon(value: f64) -> String {
    let red = Color::named(NamedColor::RED);
    let near = Color::new(1.0, 0.001, 0.0, ColorSpace::Rgb).expect("valid color");
    Color::set_epsilon(value);
    let out = format!(
        "{} {} {} {}",
        dx(Color::epsilon()),
        bit(red.is_equal(near)),
        vec3(Color::lab_to_lch([50.0, 0.001, 0.001])),
        dx(red.delta_e2000(near))
    );
    Color::set_epsilon(0.0001);
    out
}

fn color_json(text: &str) -> String {
    let mut color = Color::named(NamedColor::WHITE);
    let mut position = 1;
    let done = color.init_from_json(text, &mut position);
    let status = raised(done, |done| bit(done).to_string());
    format!("{status} {position} {}", vec3(color.rgb()))
}

fn rgba_constructed([r, g, b, a]: [f32; 4]) -> String {
    format!(
        "{} {}",
        raised(ColorRgba::new(r, g, b, a), rgba_out),
        raised(ColorRgba::from_vec4(Vec4::new(r, g, b, a)), rgba_out)
    )
}

fn rgba_pair(first: ColorRgba, second: ColorRgba) -> String {
    format!(
        "{}{}{}{}",
        bit(first.is_equal(second)),
        bit(first.is_different(second)),
        bit(first == second),
        bit(first != second)
    )
}

fn rgba_gamma(values: [f32; 4]) -> String {
    let mut out = String::new();
    for [r, g, b, a] in [
        ColorRgba::linear_to_srgb_vec4(values),
        ColorRgba::srgb_to_linear_vec4(values),
    ] {
        out += &format!("{}:{} ", vec3([r, g, b]), fx(a));
    }
    out
}

fn rgba_json(text: &str) -> String {
    let mut color = ColorRgba::from_rgb(Color::named(NamedColor::WHITE), 0.25);
    let mut position = 1;
    let done = color.init_from_json(text, &mut position);
    let status = raised(done, |done| bit(done).to_string());
    format!(
        "{status} {position} {} {}",
        vec3(color.rgb().rgb()),
        fx(color.alpha())
    )
}

fn period_text(period: Period) -> String {
    let (seconds, micros) = period.values();
    format!("{seconds}:{micros}")
}

fn period_ops(input: Input<'_, '_>) -> String {
    let first = Period::new(int(input), int(input)).expect("valid corpus period");
    let second = Period::new(int(input), int(input)).expect("valid corpus period");
    let sum = (first + second).expect("corpus avoids overflow");
    let difference = (first - second).map_or_else(
        |error| format!("raise {} {}", error.exception_type(), error.message()),
        period_text,
    );
    format!(
        "{}{}{} {} {difference}",
        bit(first == second),
        bit(first < second),
        bit(first > second),
        period_text(sum)
    )
}

fn date_ops(input: Input<'_, '_>) -> String {
    let fields: Vec<i32> = (0..8).map(|_| int(input)).collect();
    let date = Date::new(
        fields[0], fields[1], fields[2], fields[3], fields[4], fields[5], fields[6], fields[7],
    )
    .expect("valid corpus date");
    let period = Period::new(int(input), int(input)).expect("valid corpus period");
    let epoch = Date::default();
    let mut out = format!(
        "{}{}{}",
        bit(date == epoch),
        bit(date < epoch),
        bit(date > epoch)
    );
    for result in [date + period, date - period] {
        out += &match result {
            Ok(value) => format!(" {}", period_text(value.difference(epoch))),
            Err(error) => format!(" raise {} {}", error.exception_type(), error.message()),
        };
    }
    out
}
