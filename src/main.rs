use memmap2::{Advice, Mmap};
use std::{collections::HashMap, fs::File};

const DEFAULT_PATH: &str = "data/weather-measurements.csv";
const DEFAULT_MEASUREMENTS: [f64; 4] = [f64::MAX, 0.0, 0.0, f64::MIN];
const NEW_LINE: u8 = b'\n';
const DELIMITER: u8 = b';';

fn main() {
    let file = File::open(DEFAULT_PATH).unwrap();
    let mmap = unsafe { Mmap::map(&file).unwrap() };
    let _ = mmap.advise(Advice::Sequential);

    let mut measurements: HashMap<String, [f64; 4]> = HashMap::new();
    for line in mmap[..mmap.len() - 1].split(|c| *c == NEW_LINE) {
        let mut columns = line.split(|c| *c == DELIMITER);

        let station = unsafe { str::from_utf8_unchecked(columns.next().unwrap()) };
        let temperature = unsafe { str::from_utf8_unchecked(columns.next().unwrap()) };

        let value: f64 = temperature.parse().unwrap();
        let entry = measurements
            .entry(station.to_string())
            .or_insert(DEFAULT_MEASUREMENTS);

        entry[0] = entry[0].min(value);
        entry[1] += value;
        entry[2] += 1.0;
        entry[3] = entry[3].max(value);
    }

    print!("{{");
    let mut measurements = Vec::from_iter(measurements.drain());
    measurements.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));

    let mut iterator = measurements.into_iter();
    if let Some((station, [min, sum, count, max])) = iterator.next() {
        print!("{station}={min:.1}/{:.1}/{max:.1}", sum / count);
    }
    for (station, [min, sum, count, max]) in iterator {
        print!(", {station}={min:.1}/{:.1}/{max:.1}", sum / count);
    }
    println!("}}");
}
