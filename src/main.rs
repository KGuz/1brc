#![feature(portable_simd)]

use memmap2::{Advice, Mmap};
use rustc_hash::FxHashMap;
use std::{
    fs::File,
    io::{BufWriter, Write},
    simd::Simd,
};

const WEATHER_MEASUREMENTS: &str = "data/weather-measurements.csv";
const DEFAULT_MEASUREMENTS: [i32; 4] = [i32::MAX, 0, 0, i32::MIN];

const SEMICOLON: u8 = b';';
const DOT: u8 = b'.';
const MINUS: u8 = b'-';
const NEW_LINE: u8 = b'\n';
const ZERO: u8 = b'0';

fn main() {
    let file = File::open(WEATHER_MEASUREMENTS).unwrap();
    let mmap = unsafe { Mmap::map(&file).unwrap() };
    _ = mmap.advise(Advice::Sequential);

    let mut measurements = FxHashMap::with_capacity_and_hasher(10_000, Default::default());
    for line in mmap[..mmap.len() - 1].split(|c| *c == NEW_LINE) {
        let (station, temperature) = split_semicolon(line);
        let station = unsafe { str::from_utf8_unchecked(station) };
        let entry = measurements.entry(station).or_insert(DEFAULT_MEASUREMENTS);
        aggregate(entry, parse(temperature));
    }

    let mut measurements = Vec::from_iter(measurements.drain());
    measurements.sort_unstable_by_key(|(a, _)| *a);
    print(measurements);
}

fn split_semicolon(buffer: &[u8]) -> (&[u8], &[u8]) {
    unsafe {
        let mut pos = buffer.len() - 4;
        while *buffer.get_unchecked(pos) != SEMICOLON {
            pos -= 1;
        }
        let (before, after) = buffer.split_at_unchecked(pos + 1);
        (before.get_unchecked(..before.len() - 1), after)
    }
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

fn aggregate(entry: &mut [i32; 4], value: i32) {
    entry[0] = entry[0].min(value);
    entry[1] += value;
    entry[2] += 10;
    entry[3] = entry[3].max(value);
}

fn calculate([min, sum, count, max]: [i32; 4]) -> [f32; 3] {
    let v = Simd::from([min as f32, sum as f32, max as f32]);
    let w = Simd::from([10.0, count as f32, 10.0]);
    (v / w).to_array()
}

fn print(measurements: Vec<(&str, [i32; 4])>) {
    let stdout = std::io::stdout().lock();
    let mut writer = BufWriter::new(stdout);

    _ = write!(writer, "{{");
    let mut iterator = measurements.into_iter();
    if let Some((station, measurement)) = iterator.next() {
        let [min, avg, max] = calculate(measurement);
        _ = write!(writer, "{station}={min:.1}/{avg:.1}/{max:.1}");
    }
    for (station, measurement) in iterator {
        let [min, avg, max] = calculate(measurement);
        _ = write!(writer, ", {station}={min:.1}/{avg:.1}/{max:.1}");
    }
    _ = writeln!(writer, "}}");
}
