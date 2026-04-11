# 1️⃣🐝🏎️ The One Billion Row Challenge

## About

The text file contains temperature values for a range of weather stations.
Each row is one measurement in the format `<string: station name>;<double: measurement>`, with the measurement value having exactly one fractional digit.
The following shows ten rows as an example:

```
Hamburg;12.0
Bulawayo;8.9
Palembang;38.8
St. John's;15.2
Cracow;12.6
Bridgetown;26.9
Istanbul;6.2
Roseau;34.4
Conakry;31.2
Istanbul;23.0
```

The task is to write a program which reads the file, calculates the min, mean, and max temperature value per weather station, and emits the results on stdout like this
(i.e. sorted alphabetically by station name, and the result values per station in the format `<min>/<mean>/<max>`, rounded to one fractional digit):

```
{Abha=-23.0/18.0/59.2, Abidjan=-16.2/26.0/67.3, Abéché=-10.0/29.4/69.0, Accra=-10.1/26.4/66.4, Addis Ababa=-23.7/16.0/67.0, Adelaide=-27.8/17.3/58.5, ...}
```

## Profiling

- `perf record --call-graph dwarf -- ./target/release/one-billion-row-challange`
- `flamegraph /target/release/one-billion-row-challange`
- `hyperfine /target/release/one-billion-row-challange`

## Benchmarks

**naive solution**
```sh
Time (mean ± σ):     95.449 s ±  0.391 s    [User: 94.483 s, System: 0.817 s]
Range (min … max):   94.976 s … 96.210 s    10 runs
```
**hashmap + sort**
```sh
Time (mean ± σ):     54.616 s ±  0.377 s    [User: 53.741 s, System: 0.789 s]
Range (min … max):   54.166 s … 55.293 s    10 runs
```
**memmap**
```sh
Time (mean ± σ):     34.469 s ±  0.306 s    [User: 34.187 s, System: 0.226 s]
Range (min … max):   34.227 s … 35.263 s    10 runs
```
**String -> &str**
```sh
Time (mean ± σ):     28.666 s ±  0.177 s    [User: 28.390 s, System: 0.231 s]
Range (min … max):   28.376 s … 28.858 s    10 runs
```
**integer temperature**
```sh
Time (mean ± σ):     26.565 s ±  0.194 s    [User: 26.276 s, System: 0.242 s]
Range (min … max):   26.179 s … 26.868 s    10 runs
```
**rustc hash**
```sh
Time (mean ± σ):     20.716 s ±  0.098 s    [User: 20.435 s, System: 0.240 s]
Range (min … max):   20.585 s … 20.890 s    10 runs
```
**pattern matching parse**
```sh
Time (mean ± σ):     19.893 s ±  0.043 s    [User: 19.626 s, System: 0.235 s]
Range (min … max):   19.810 s … 19.952 s    10 runs
```
**buffered output**
```sh
Time (mean ± σ):     19.306 s ±  0.046 s    [User: 19.047 s, System: 0.227 s]
Range (min … max):   19.235 s … 19.373 s    10 runs
```
**simd division**
```sh
Time (mean ± σ):     18.971 s ±  0.032 s    [User: 18.708 s, System: 0.228 s]
Range (min … max):   18.913 s … 19.003 s    10 runs
```
**split semicolon**
```sh
Time (mean ± σ):     16.263 s ±  0.018 s    [User: 16.007 s, System: 0.229 s]
Range (min … max):   16.241 s … 16.289 s    10 runs
```
