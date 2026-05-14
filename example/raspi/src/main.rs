use anyhow::Result;
use embedded_hal::spi::SpiDevice as _;
use embedded_hal_async::delay::DelayNs;
use linux_embedded_hal::SpidevDevice;
use log::*;
use sch16t_rs::*;
use std::time::Duration;
use uom::si::{acceleration::meter_per_second_squared, angular_velocity::degree_per_second};

struct AsyncSpiWrapper {
    inner: SpidevDevice,
}

#[derive(Debug)]
enum WrapperError {}

pub struct Delay;

impl embedded_hal_async::delay::DelayNs for Delay {
    async fn delay_ns(&mut self, n: u32) {
        tokio::time::sleep(Duration::from_nanos(n.into())).await;
    }

    async fn delay_us(&mut self, n: u32) {
        tokio::time::sleep(Duration::from_micros(n.into())).await;
    }

    async fn delay_ms(&mut self, n: u32) {
        tokio::time::sleep(Duration::from_millis(n.into())).await;
    }
}

impl AsyncSpiWrapper {
    pub fn open(path: &str) -> Result<Self> {
        let mut inner = SpidevDevice::open(path)?;

        let options = linux_embedded_hal::spidev::SpidevOptions::new()
            .max_speed_hz(10_000_000)
            .build();
        inner.configure(&options)?;

        Ok(Self { inner })
    }
}

impl embedded_hal_async::spi::Error for WrapperError {
    fn kind(&self) -> embedded_hal_async::spi::ErrorKind {
        match *self {}
    }
}

impl embedded_hal_async::spi::ErrorType for AsyncSpiWrapper {
    type Error = WrapperError;
}

impl embedded_hal_async::spi::SpiDevice<u8> for AsyncSpiWrapper {
    async fn transaction(
        &mut self,
        ops: &mut [embedded_hal_async::spi::Operation<'_, u8>],
    ) -> Result<(), WrapperError> {
        self.inner.transaction(ops).unwrap();

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let spidev = AsyncSpiWrapper::open("/dev/spidev0.1")?;

    let mut del = Delay;
    let mut imu = Sch16t::new(spidev, Delay, AddressPins::Ta8LowTa9Low);

    info!("Initializing sensor");
    loop {
        match imu.init(Default::default()).await {
            Ok(_) => break,
            Err(e) => {
                del.delay_ms(100).await;
                warn!("Init error: {:?}", e);
            }
        }
    }

    loop {
        match imu.get_sample().await {
            Ok(sample) => {
                let Sample {
                    ref accel,
                    ref rate,
                    ..
                } = sample;
                info!(
                    "rate:  x: {:0.3}\ty: {:0.3}\tz: {:0.3} \t\t accel: x: {:0.3}\ty: {:0.3}\tz: {:0.3}",
                    rate.x.get::<degree_per_second>(),
                    rate.y.get::<degree_per_second>(),
                    rate.z.get::<degree_per_second>(),
                    accel.x.get::<meter_per_second_squared>(),
                    accel.y.get::<meter_per_second_squared>(),
                    accel.z.get::<meter_per_second_squared>(),
                );
                if sample.accel.saturated() {
                    warn!("accel sat");
                }
                if sample.rate.saturated() {
                    warn!("rate sat");
                }
            }
            Err(e) => warn!("{:?}", e),
        }
        del.delay_ms(100).await;
    }
}
