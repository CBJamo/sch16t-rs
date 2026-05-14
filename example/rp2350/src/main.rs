#![no_std]
#![no_main]

use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::{main, Spawner};
use embassy_rp::executor::Executor;
use embassy_rp::{
    bind_interrupts, dma,
    gpio::{Input, Level, Output, Pull},
    peripherals, spi,
};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{Delay, Timer};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

use sch16t_rs::*;

type SPI0 = spi::Spi<'static, peripherals::SPI0, spi::Async>;

static SPI_BUS: StaticCell<Mutex<NoopRawMutex, SPI0>> = StaticCell::new();

bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => dma::InterruptHandler<peripherals::DMA_CH0>, embassy_rp::dma::InterruptHandler<peripherals::DMA_CH1>;
});

#[main(executor = "Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("start");

    let mosi = p.PIN_7;
    let miso = p.PIN_20;
    let sclk = p.PIN_6;

    let mut spi_config = spi::Config::default();
    spi_config.frequency = 10_000_000;
    let spi = spi::Spi::new(
        p.SPI0, sclk, mosi, miso, p.DMA_CH0, p.DMA_CH1, Irqs, spi_config,
    );
    let spi_bus = SPI_BUS.init(Mutex::new(spi));

    // The driver intentionally does not handle the interrupt pin.
    // It's awkward to make it optional, and in high-performance applications you
    // probably want to take the timestamp in an ISR to remove any async jitter.
    let mut interrupt = Input::new(p.PIN_15, Pull::Down);

    let mut _reset = Output::new(p.PIN_16, Level::High);

    let cs = p.PIN_13;
    let spidev = SpiDevice::new(spi_bus, Output::new(cs, Level::High));

    let mut imu = Sch16t::new(spidev, Delay, AddressPins::Ta8LowTa9Low);

    info!("Initializing sensor");
    loop {
        let config = Config {
            // This is the most important setting, see Datasheet section 5.4
            // for an excellent rundown on what these modes mean.
            sample_mode: SampleMode::Decimation,
            dry_sync_mode: SyncDryMode::Dry,
            // Optionally read temperature with every sample
            include_temperature: true,
            // Dynamic range is shared for all axes
            rate_range: RateDynamicRange::Dyn1,
            // But filters and decimation ratio can be set per axis
            rate_x_lpf: Filter::Lpf1,
            rate_y_lpf: Filter::Lpf2,
            rate_z_lpf: Filter::Lpf3,
            rate_x_dec: DecimationRatio::Dec4,
            rate_y_dec: DecimationRatio::Dec5,
            // Of course the gyro and accel have independant settings
            accel_range: AccDynamicRange::Dyn2,
            accel_x_lpf: Filter::Lpf0,
            // Other settings can be elided as so
            // The defaults for the hardware interface are probably what you want.
            ..Default::default()
        };

        match imu.init(config).await {
            Ok(_) => break,
            Err(e) => {
                Timer::after_millis(100).await;
                warn!("Init error: {:?}", e);
            }
        }
    }

    info!("Entering measure loop");
    loop {
        interrupt.wait_for_high().await;
        match imu.get_raw_sample().await {
            Ok(sample) => {
                let RawSample {
                    ref accel,
                    ref rate,
                    ref temp,
                    ..
                } = sample;
                info!(
                    "temp: {} rate:  x: {} y: {} z: {} \t accel: x: {} y: {} z: {}",
                    temp, rate.x, rate.y, rate.z, accel.x, accel.y, accel.z,
                );
                if sample.accel.saturated() {
                    warn!("accel sat");
                }
                if sample.rate.x_saturated {
                    warn!("rate x sat");
                }
                if sample.rate.y_saturated {
                    warn!("rate y sat");
                }
                if sample.rate.z_saturated {
                    warn!("rate z sat");
                }
            }
            Err(e) => warn!("{:?}", e),
        }
    }
}
