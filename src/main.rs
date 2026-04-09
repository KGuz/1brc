use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

const DEFAULT_PATH: &str = "data/weather-measurements.csv";
const DEFAULT_MEASUREMENTS: [f64; 4] = [f64::MAX, 0.0, 0.0, f64::MIN];

fn main() {
    let file = File::open(DEFAULT_PATH).unwrap();
    let reader = BufReader::new(file);
    let mut measurements: HashMap<String, [f64; 4]> = HashMap::new();

    for line in reader.lines() {
        let line = line.unwrap();
        let (station, temp) = line.split_once(';').unwrap();
        let value: f64 = temp.parse().unwrap();

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
