# SCH16T Rust Driver

Async, no_std driver for the Murata SCH16T family of 6-axis IMUs.

The main access to the driver is through the [Sch16t] struct and the [RawSample] output or, with uom, [Sample]


### Usage

#### Basics
```rust
let mut imu = Sch16t::new(spidev, Delay, AddressPins::Ta8LowTa9Low);

info!("Initializing sensor");
imu.init(Default::default()).await?;

let mut ticker = Ticker::every(Duration::from_millis(100));

loop {
    match imu.get_raw_sample().await {
        Ok(sample) => info!("{:?}", sample),
        Err(e) => warn!("{:?}", e),
    }
    ticker.next().await;
}

```

#### [UOM](https://crates.io/crates/uom)
``` rust
let sample = imu.get_sample().await?;

info!(
    "rate:  x: {:0.3}\ty: {:0.3}\tz: {:0.3} \t\t accel: x: {:0.3}\ty: {:0.3}\tz: {:0.3}",
    sample.rate.x.get::<degree_per_second>(),
    sample.rate.y.get::<degree_per_second>(),
    sample.rate.z.get::<degree_per_second>(),
    sample.accel.x.get::<meter_per_second_squared>(),
    sample.accel.y.get::<meter_per_second_squared>(),
    sample.accel.z.get::<meter_per_second_squared>(),
);
```

#### Examples

See also full examples in the repo [here](https://github.com/CBJamo/sch16t-rs/tree/main/example)

## Optional Features

| Feature | Default | Description                                                               |
| ------- | ------- | ------------------------------------------------------------------------- |
| `defmt` | yes     | Enables `defmt::Format` derives and routes internal logging through defmt |
| `log`   | no      | Routes internal logging through [Log](https://crates.io/crates/log)       |
| `serde` | no      | Enables `serde::Serialize`/`Deserialize` on output data types             |
| `uom`   | no      | Enables methods returning `uom` SI unit types                             |

## Links
* [Embedded-hal](https://github.com/rust-embedded/embedded-hal)
* [SCH16 Product Page](https://www.murata.com/products/sensor/gyro/overview/lineup/sch16t)
* [Short DS](https://www.murata.com/-/media/webrenewal/products/sensor/pdf/datasheet/datasheet-sch16t-k01-short.ashx?la=en-us&cvid=20251211010000000000)
* The full datasheet requires agreeing to the [use restriction](https://www.murata.com/support/militaryrestriction) in [this form](https://www.murata.com/en-us/products/sensor/gyro/overview/lineup/sch16t/form), but is otherwise automatic.


## License

This library is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
