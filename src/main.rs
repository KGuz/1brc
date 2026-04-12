#![feature(portable_simd)]
mod cursor;
mod temp;

use cursor::Cursor;
use memmap2::{Advice, Mmap};
use rustc_hash::FxHashMap;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufWriter, Write, stdout},
    thread,
};
use temp::Temp;

const WEATHER_MEASUREMENTS: &str = "data/weather-measurements.csv";

fn main() {
    let file = File::open(WEATHER_MEASUREMENTS).unwrap();
    _ = file.lock();
    let mmap = unsafe { Mmap::map(&file).unwrap() };
    _ = mmap.advise(Advice::Sequential);

    let mut measurements = BTreeMap::new();
    let threads = unsafe { thread::available_parallelism().unwrap_unchecked().get() };

    thread::scope(|s| {
        let (tx, rx) = std::sync::mpsc::channel();
        let cursor = Cursor::from(mmap.as_ref());

        for chunk in cursor.chunks(threads) {
            let tx = tx.clone();
            s.spawn(move || tx.send(process(chunk)));
        }
        drop(tx);

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
    let cursor = Cursor::from(buffer);

    for (station, temperature) in cursor.lines() {
        let station = unsafe { str::from_utf8_unchecked(station) };
        let temperature = temp::parse(temperature);

        *measurements.entry(station).or_default() += temperature;
    }
    measurements
}

fn print(measurements: BTreeMap<&str, Temp>) {
    let mut writer = BufWriter::new(stdout().lock());
    let mut iterator = measurements.into_iter();

    _ = write!(writer, "{{");
    if let Some((station, measurement)) = iterator.next() {
        _ = write!(writer, "{station}={measurement}");
    }
    for (station, measurement) in iterator {
        _ = write!(writer, ", {station}={measurement}");
    }
    _ = writeln!(writer, "}}");
}
