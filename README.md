# nws_exporter

[![build status](https://circleci.com/gh/56quarters/nws_exporter.svg?style=shield)](https://circleci.com/gh/56quarters/nws_exporter)
[![docs.rs](https://docs.rs/nws_exporter/badge.svg)](https://docs.rs/nws_exporter/)
[![crates.io](https://img.shields.io/crates/v/nws_exporter.svg)](https://crates.io/crates/nws_exporter/)

Prometheus metrics exporter for api.weather.gov

## Features

`nws_exporter` fetches weather information for a particular [NWS station] using the [api.weather.gov] API and emits
it as Prometheus metrics. Users must pick a particular station to fetch weather information from. The following
metrics are emitted when available (not all fields are available for all stations).

* `nws_station{station=$STATION, station_id=$STATION_ID, station_name=$STATION_NAME}` - Station metadata
* `nws_elevation_meters{station=$STATION}` - Elevation of the station, in meters.
* `nws_temperature_degrees{station=$STATION}` - Temperature, in degrees celsius.
* `nws_dewpoint_degrees{station=$STATION}` - Dewpoint, in degrees celsius.
* `nws_barometric_pressure_pascals{station=$STATION}` - Barometric pressure, in pascals.
* `nws_visibility_meters{station=$STATION}` - Visibility, in meters.
* `nws_relative_humidity{station=$STATION}` - Relative humidity (0-100).
* `nws_wind_chill_degrees{station=$STATION}` - Temperature with wind chill, in degrees celsius.

[NWS station]: https://www.weather.gov/documentation/services-web-api#/default/obs_stations
[api.weather.gov]: https://www.weather.gov/documentation/services-web-api

## Build

`nws_exporter` is a Rust program and must be built from source using a [Rust toolchain](https://rustup.rs/).

### Build from source

If you want to build from the latest code in the `nws_exporter` repo, you can build using the following
steps.

```text
git clone git@github.com:56quarters/nws_exporter.git && cd nws_exporter
cargo build --release
```

### Install via cargo

After you have a Rust toolchain, you can also install the latest release directly via `cargo install`

```text
cargo install nws_exporter
```

## Usage

### Picking a station

In order to export NWS forecast information, `nws_exporter` needs to be told which NWS station to request
information for. You can get a list of the available stations in your state by using the API itself. An
example of this using `curl` is below.

```text
curl -sS 'https://api.weather.gov/stations?state=MA' | jq | less
```

This command lists all available stations in the state of Massachusetts. The `properties.stationIdentifier`
field for each station is the ID that you should use with `nws_exporter`. For example `KBOS` is the ID for
the station at Logan Airport in Boston.

You can then run `nws_exporter` for this station as demonstrated below.

```text
./nws_exporter --station KBOS
```

### Run

You can run `nws_exporter` as a Systemd service using the [provided unit file](ext/nws_exporter.service). This
unit file  assumes that you have copied the resulting `nws_exporter` binary to `/usr/local/bin/nws_exporter`.

```text
sudo cp target/release/nws_exporter /usr/local/bin/nws_exporter
sudo cp ext/nws_exporter.service /etc/systemd/system/nws_exporter.service
sudo systemctl daemon-reload
sudo systemctl enable nws_exporter.service
sudo systemctl start nws_exporter.serivce
```

### Prometheus

Prometheus metrics are exposed on port `9782` at `/metrics`. Once `nws_exporter`
is running, configure scrapes of it by your Prometheus server. Add the host running
`nws_exporter` as a target under the Prometheus `scrape_configs` section as described by
the example below.

```yaml
# Sample config for Prometheus.

global:
  scrape_interval:     15s
  evaluation_interval: 15s
  external_labels:
    monitor: 'my_prom'

scrape_configs:
- job_name: nws_exporter
  static_configs:
  - targets: ['example:9782']
```

## License

nws_exporter is available under the terms of the [GPL, version 3](LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be licensed as above, without any
additional terms or conditions.
