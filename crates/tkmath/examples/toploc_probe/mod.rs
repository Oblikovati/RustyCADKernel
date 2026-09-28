//! TopLoc_Location composition, powers, equality and hashing oracle counterpart.
use rustycad_tkernel::standard::{Failure, Handle, dump::DumpStream};
use rustycad_tkmath::{
    gp::{Axis1, Direction3, Point3, Transform3},
    toploc::{Datum3, Location},
};
/// Sequential reader over one request's numeric fields.
struct Fields<'v> {
    values: &'v [f64],
    next: usize,
}
impl Fields<'_> {
    fn real(&mut self) -> f64 {
        self.next += 1;
        self.values[self.next - 1]
    }
    fn int(&mut self) -> i32 {
        self.real() as i32
    }
    fn triple(&mut self) -> [f64; 3] {
        [self.real(), self.real(), self.real()]
    }
}
fn trsf(t: &Transform3) {
    for row in 1..=3 {
        for column in 1..=4 {
            print!("{} ", t.value(row, column).unwrap().to_bits());
        }
    }
    print!("{} ", t.form() as i32);
}
fn datum(fields: &mut Fields<'_>) -> Result<Handle<Datum3>, Failure> {
    let (p, d) = (fields.triple(), fields.triple());
    let (angle, scale, t) = (fields.real(), fields.real(), fields.triple());
    let axis = Axis1::new(
        Point3::new(p[0], p[1], p[2]),
        Direction3::new(d[0], d[1], d[2])?,
    );
    let rotation = Transform3::rotation(axis, angle)?;
    let scaling = Transform3::scaling(Point3::default(), scale)?;
    let translation = Transform3::translation(t);
    Ok(Handle::new(Datum3::new(
        translation.multiplied(scaling).multiplied(rotation),
    )))
}
fn hashes(chain: &Location, datums: &[Handle<Datum3>]) -> Result<(), Failure> {
    let (mut real, mut synthetic) = (Vec::new(), Vec::new());
    let mut item = chain.clone();
    while !item.is_identity() {
        let datum = item.first_datum()?.clone();
        let id = datums
            .iter()
            .position(|d| *d == datum)
            .unwrap_or(datums.len());
        let power = item.first_power()?;
        real.push((datum.as_ptr() as usize, power));
        synthetic.push((id * 8 + 4096, power));
        item = item.next_location().clone();
    }
    let matches = chain.hash_code() == Location::hash_items(&real);
    print!(
        "{} {} {} ",
        i32::from(matches),
        Location::hash_items(&synthetic),
        real.len()
    );
    Ok(())
}
fn guarded(tag: &str, make: impl FnOnce() -> Result<Location, Failure>) {
    print!("{tag} ");
    match make() {
        Ok(location) => {
            print!("{} ", i32::from(location.is_identity()));
            trsf(&location.transformation());
        }
        Err(error) => print!("{} ", error.exception_type()),
    }
}
fn chain_of(fields: &mut Fields<'_>, datums: &[Handle<Datum3>]) -> (Location, Location) {
    let (mut chain, mut first) = (Location::default(), Location::default());
    for index in 0..fields.int() {
        let (id, power) = (fields.int() as usize, fields.int());
        let composed = Location::from_datum(datums[id].clone())
            .powered(power)
            .and_then(|item| {
                if index == 0 {
                    first = item.clone();
                }
                chain.multiplied(&item)
            });
        match composed {
            Ok(next) => chain = next,
            Err(error) => {
                print!("{} ", error.exception_type());
                break;
            }
        }
    }
    (chain, first)
}
fn report(chain: &Location, first: &Location, power: i32) {
    type Make<'a> = Box<dyn FnOnce() -> Result<Location, Failure> + 'a>;
    let steps: [(&str, Make<'_>); 7] = [
        ("inv", Box::new(|| chain.inverted())),
        ("id", Box::new(|| chain.multiplied(&chain.inverted()?))),
        ("pow", Box::new(|| chain.powered(power))),
        ("npow", Box::new(|| chain.powered(-power))),
        ("div", Box::new(|| chain.divided(first))),
        ("pre", Box::new(|| chain.predivided(first))),
        ("self", Box::new(|| chain.predivided(chain))),
    ];
    for (tag, make) in steps {
        guarded(tag, make);
    }
}
pub(crate) fn location_case(values: &[f64]) {
    let mut fields = Fields { values, next: 0 };
    let mut datums = Vec::new();
    for _ in 0..fields.int() {
        match datum(&mut fields) {
            Ok(datum) => datums.push(datum),
            Err(error) => {
                println!("{} ", error.exception_type());
                return;
            }
        }
    }
    let (chain, first) = chain_of(&mut fields, &datums);
    let power = fields.int();
    print!("{} ", i32::from(chain.is_identity()));
    trsf(&chain.transformation());
    hashes(&chain, &datums).unwrap();
    match chain.first_power() {
        Ok(power) => print!("{power} "),
        Err(error) => print!("{} ", error.exception_type()),
    }
    trsf(&chain.next_location().transformation());
    report(&chain, &first, power);
    let copy = Location::new(chain.transformation());
    let identity = Location::default();
    let with_identity = chain.multiplied(&identity).unwrap() == chain;
    let cancels = first.inverted().unwrap().multiplied(&first).unwrap() == identity;
    print!(
        "{}{}{} ",
        i32::from(with_identity),
        i32::from(chain != copy),
        i32::from(cancels)
    );
    let mut json = DumpStream::default();
    chain.dump_json(&mut json, -1).unwrap();
    println!("{}", json.text());
}
