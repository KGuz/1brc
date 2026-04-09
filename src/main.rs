use memmap2::{Advice, Mmap};
use rustc_hash::FxHashMap;
use std::fs::File;

const DEFAULT_PATH: &str = "data/weather-measurements.csv";
const DEFAULT_MEASUREMENTS: [i32; 4] = [i32::MAX, 0, 0, i32::MIN];

pub const DELIMITER: u8 = b';';
pub const DOT: u8 = b'.';
pub const MINUS: u8 = b'-';
pub const NEW_LINE: u8 = b'\n';
pub const ZERO: u8 = b'0';

fn main() {
    let file = File::open(DEFAULT_PATH).unwrap();
    let mmap = unsafe { Mmap::map(&file).unwrap() };
    let _ = mmap.advise(Advice::Sequential);

    let mut measurements = FxHashMap::with_capacity_and_hasher(10_000, Default::default());
    for line in mmap[..mmap.len() - 1].split(|c| *c == NEW_LINE) {
        let mut columns = line.split(|c| *c == DELIMITER);

        let station = unsafe { str::from_utf8_unchecked(columns.next().unwrap()) };
        let value = parse(columns.next().unwrap());
        let entry = measurements.entry(station).or_insert(DEFAULT_MEASUREMENTS);

        entry[0] = entry[0].min(value);
        entry[1] += value;
        entry[2] += 10;
        entry[3] = entry[3].max(value);
    }

    print!("{{");
    let mut measurements = Vec::from_iter(measurements.drain());
    measurements.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));

    let mut iterator = measurements.into_iter();
    if let Some((station, measurement)) = iterator.next() {
        let [min, sum, count, max] = measurement.map(f64::from);
        let (min, max) = (min / 10.0, max / 10.0);

        print!("{station}={min:.1}/{:.1}/{max:.1}", sum / count);
    }
    for (station, measurement) in iterator {
        let [min, sum, count, max] = measurement.map(f64::from);
        let (min, max) = (min / 10.0, max / 10.0);

        print!(", {station}={min:.1}/{:.1}/{max:.1}", sum / count);
    }
    println!("}}");
}

#[rustfmt::skip]
fn parse(temperature: &[u8]) -> i32 {
    let f = |x| (x - ZERO) as i32;
    match temperature {
        [MINUS, h, d, DOT, u] => -f(h) * 100 - f(d) * 10 - f(u),
        [MINUS,    d, DOT, u] =>             - f(d) * 10 - f(u),
        [       h, d, DOT, u] =>  f(h) * 100 + f(d) * 10 + f(u),
        [          d, DOT, u] =>               f(d) * 10 + f(u),
        _ => unreachable!(),
    }
}
