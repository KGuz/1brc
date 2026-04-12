#![feature(portable_simd)]
mod temp;

use memmap2::{Advice, Mmap};
use rustc_hash::FxHashMap;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufWriter, Write},
    simd::{Simd, cmp::SimdPartialEq, u8x16},
    thread,
};
use temp::Temp;

const WEATHER_MEASUREMENTS: &str = "data/weather-measurements.csv";

fn main() {
    let file = File::open(WEATHER_MEASUREMENTS).unwrap();
    let mmap = unsafe { Mmap::map(&file).unwrap() };
    _ = mmap.advise(Advice::Sequential);

    let mut measurements = BTreeMap::new();
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
                *measurements.entry(station).or_default() += measurement;
            }
        }
    });
    print(measurements);
}

fn process(buffer: &[u8]) -> FxHashMap<&str, Temp> {
    let mut measurements = FxHashMap::with_capacity_and_hasher(8192, Default::default());
    let (mut ptr, end) = (0, buffer.len());

    while ptr < end {
        let idx = ptr + find_new_line(&buffer[ptr..]);
        let line = unsafe { buffer.get_unchecked(ptr..idx) };

        let (station, temperature) = split_semicolon(line);
        let station = unsafe { str::from_utf8_unchecked(station) };
        let temperature = temp::parse(temperature);

        *measurements.entry(station).or_default() += temperature;
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

fn print(measurements: BTreeMap<&str, Temp>) {
    let stdout = std::io::stdout().lock();
    let mut writer = BufWriter::new(stdout);

    _ = write!(writer, "{{");
    let mut iterator = measurements.into_iter();
    if let Some((station, measurement)) = iterator.next() {
        _ = write!(writer, "{station}={measurement}");
    }
    for (station, measurement) in iterator {
        _ = write!(writer, ", {station}={measurement}");
    }
    _ = writeln!(writer, "}}");
}
