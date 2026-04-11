#![feature(portable_simd)]

use memmap2::{Advice, Mmap};
use rustc_hash::FxHashMap;
use std::{
    fs::File,
    io::{BufWriter, Write},
    simd::{Simd, cmp::SimdPartialEq, u8x16},
    thread,
};

const WEATHER_MEASUREMENTS: &str = "data/weather-measurements.csv";
const DEFAULT_MEASUREMENTS: [i32; 4] = [i32::MAX, 0, 0, i32::MIN];

fn main() {
    let file = File::open(WEATHER_MEASUREMENTS).unwrap();
    let mmap = unsafe { Mmap::map(&file).unwrap() };
    _ = mmap.advise(Advice::Sequential);

    let mut measurements = FxHashMap::with_capacity_and_hasher(8192, Default::default());
    let (mut ptr, len) = (0, mmap.len());
    let threads = unsafe { thread::available_parallelism().unwrap_unchecked() };
    let size = len / threads;

    thread::scope(|s| {
        let (tx, rx) = std::sync::mpsc::channel();
        while ptr < len - size {
            let mut end = ptr + size;
            end += find_new_line(&mmap[end..]) + 1;

            let buffer = &mmap[ptr..end];
            let tx = tx.clone();
            s.spawn(move || tx.send(process(buffer)));

            ptr = end;
        }
        let buffer = &mmap[ptr..];
        s.spawn(move || tx.send(process(buffer)));

        while let Ok(result) = rx.recv() {
            for (station, measurement) in result {
                let entry = measurements.entry(station).or_insert(DEFAULT_MEASUREMENTS);
                merge(entry, measurement);
            }
        }
    });

    let mut measurements = Vec::from_iter(measurements.drain());
    measurements.sort_unstable_by_key(|(a, _)| *a);
    print(measurements);
}

fn process(buffer: &[u8]) -> FxHashMap<&str, [i32; 4]> {
    let mut measurements = FxHashMap::with_capacity_and_hasher(8192, Default::default());
    let (mut ptr, end) = (0, buffer.len());

    while ptr < end {
        let idx = ptr + find_new_line(&buffer[ptr..]);

        let line = unsafe { buffer.get_unchecked(ptr..idx) };
        let (station, temperature) = split_semicolon(line);
        let station = unsafe { str::from_utf8_unchecked(station) };

        let entry = measurements.entry(station).or_insert(DEFAULT_MEASUREMENTS);
        aggregate(entry, parse(temperature));

        ptr = idx + 1;
    }
    measurements
}

fn find_new_line(mut buffer: &[u8]) -> usize {
    const SPLAT: u8x16 = Simd::splat(b'\n');
    const COUNT: usize = 16;

    let mut ptr = 0;
    while let Some((chunk, rest)) = buffer.split_first_chunk() {
        let bytes = Simd::from_array(*chunk);
        let index = bytes.simd_eq(SPLAT).first_set().map(|i| i + ptr);
        if let Some(index) = index {
            return index;
        }
        ptr += COUNT;
        buffer = rest;
    }

    let bytes = Simd::load_or_default(buffer);
    let index = bytes.simd_eq(SPLAT).first_set().map(|i| i + ptr);
    unsafe { index.unwrap_unchecked() }
}

fn split_semicolon(buffer: &[u8]) -> (&[u8], &[u8]) {
    let mut pos = buffer.len() - 4;
    while unsafe { *buffer.get_unchecked(pos) } != b';' {
        pos -= 1;
    }
    let (before, after) = unsafe { buffer.split_at_unchecked(pos + 1) };
    unsafe { (before.get_unchecked(..before.len() - 1), after) }
}

#[rustfmt::skip]
fn parse(temperature: &[u8]) -> i32 {
    let f = |x| (x - b'0') as i32;
    match temperature {
        [b'-', h, d, b'.', u] => -f(h) * 100 - f(d) * 10 - f(u),
        [b'-',    d, b'.', u] =>             - f(d) * 10 - f(u),
        [      h, d, b'.', u] =>  f(h) * 100 + f(d) * 10 + f(u),
        [         d, b'.', u] =>               f(d) * 10 + f(u),
        _ => unreachable!(),
    }
}

fn aggregate([min, agg, cnt, max]: &mut [i32; 4], val: i32) {
    *min = val.min(*min);
    *agg += val;
    *cnt += 10;
    *max = val.max(*max);
}

fn merge([a_min, a_agg, a_cnt, a_max]: &mut [i32; 4], [b_min, b_agg, b_cnt, b_max]: [i32; 4]) {
    *a_min = b_min.min(*a_min);
    *a_agg += b_agg;
    *a_cnt += b_cnt;
    *a_max = b_max.max(*a_max);
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
