#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use embedded_hal_async::delay::DelayNs;
use heapless::Vec;

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

mod interface;
#[allow(missing_docs)]
mod low_level;

#[cfg(feature = "uom")]
mod uom_impl;

pub use low_level::{
    AccDynamicRange, DecimationRatio, Filter, HiSpd, Polarity, RateDynamicRange, SensorStatus,
    SpiSupply, StatAccFields, StatComFields, StatInfoFields, StatRateComFields, StatRateFields,
    StatSumFields, StatSumSatFields, StatSyncActiveFields,
};

#[cfg(feature = "uom")]
#[allow(unused_imports)]
pub use uom_impl::*;

/// Main sensor struct.
pub struct Sch16t<BUS, DELAY> {
    /// Raw access to
    pub ll: low_level::LowLevel<interface::Interface<BUS>>,
    delay: DELAY,
    config: Option<Config>,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug)]
#[allow(missing_docs)]
/// Sample counters in each axis of a sample
///
/// See DS section 5.4.5 for full details.
pub struct DataCounter {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug)]
/// Raw accelerometer data
pub struct RawAccel {
    /// x axis raw measurement
    pub x: i32,
    /// y axis raw measurement
    pub y: i32,
    /// z axis raw measurement
    pub z: i32,
    /// x axis was saturated for this sample window
    pub x_saturated: bool,
    /// y axis was saturated for this sample window
    pub y_saturated: bool,
    /// z axis was saturated for this sample window
    pub z_saturated: bool,
    /// LSB/meter/sec/sec for this sample as set by [Config].accel_range
    ///
    /// To get meters/second/second, divide the counts values by this value.
    pub lsb_mss: i32,
    /// Optional [DataCounter], as set in [Config].include_data_counters
    pub data_counter: Option<DataCounter>,
}

impl RawAccel {
    /// Convience fn to check if any axis is saturated
    pub fn saturated(&self) -> bool {
        self.x_saturated || self.y_saturated || self.z_saturated
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug)]
/// Raw gyroscope data
pub struct RawRate {
    /// x axis raw measurement
    pub x: i32,
    /// y axis raw measurement
    pub y: i32,
    /// z axis raw measurement
    pub z: i32,
    /// x axis was saturated for this sample window
    pub x_saturated: bool,
    /// y axis was saturated for this sample window
    pub y_saturated: bool,
    /// z axis was saturated for this sample window
    pub z_saturated: bool,
    /// LSB/degree/sec for this sample as set by [Config].rate_range.
    ///
    /// To get degrees/second, divide the counts values by this value.
    pub lsb_ds: i32,
    /// Optional [DataCounter], as set in [Config].include_data_counters
    pub data_counter: Option<DataCounter>,
}

impl RawRate {
    /// Convience fn to check if any axis is saturated
    pub fn saturated(&self) -> bool {
        self.x_saturated || self.y_saturated || self.z_saturated
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug)]
/// Full gyro and accel data sample
pub struct RawSample {
    /// Accelerometer data
    pub accel: RawAccel,
    /// Gyroscope data
    pub rate: RawRate,
    /// Optional temperature measurement, as set by [Config].include_temperature temperature range
    /// is not configurable, it is always 100 LSB/C. In other words, this value can be read directly
    /// as millicelsius
    pub temp: Option<i16>,
    /// Optional freq_counter, as set by [Config].include_freq_counter
    pub freq_counter: Option<u16>,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug)]
/// Main user-facing error
pub enum Error<E> {
    /// I2c error, see internal error for details.
    Bus(E),
    /// Rx crc mismatch
    Crc,
    /// Tried to read while the sensor was not initialized
    SensorNotInit,
    /// Config did not sucsessfully write to sensor on init
    ConfigFailed(FullStatus),
    /// Errors in status on init
    InitError(FullStatus),
    /// Error while reading sensor data in Common block
    CommonError(StatComFields),
    /// Error while reading sensor data in Gyro X block
    GyroXError((StatRateComFields, StatRateFields)),
    /// Error while reading sensor data in Gyro Y block
    GyroYError((StatRateComFields, StatRateFields)),
    /// Error while reading sensor data in Gyro Z block
    GyroZError((StatRateComFields, StatRateFields)),
    /// Error while reading sensor data in Acc X block
    AccXError(StatAccFields),
    /// Error while reading sensor data in Acc Y block
    AccYError(StatAccFields),
    /// Error while reading sensor data in Acc Z block
    AccZError(StatAccFields),
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq)]
#[allow(missing_docs)]
/// The SCH16T is addressed pulling the TA8 and TA9 pins high or low.
pub enum AddressPins {
    Ta8LowTa9Low,
    Ta8HighTa9Low,
    Ta8LowTa9High,
    Ta8HighTa9High,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
/// Select the main sensor read mode.
///
/// See Section 5.4 of the full datasheet for a full explanation of these modes and their use-cases.
pub enum SampleMode {
    #[default]
    /// Configure [Sch16t::get_raw_sample] to return from the interpolated registers.
    /// In this mode, the axes [DecimationRatio] will not be used.
    Interpolate,
    /// Configure [Sch16t::get_raw_sample] to return from the decimated registers.
    /// In this mode, you should also set each axes [DecimationRatio].
    Decimation,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
/// Select the dry_sync pin's mode.
///
/// See Section 5.4.3 and 5.4.4 of the full datasheet for a full explanation of the SyncDry pin.
pub enum SyncDryMode {
    #[default]
    /// Do not enable the SyncDry pin
    Off,
    /// Configure the SyncDry pin as a Sync input
    Sync,
    /// Configure the SyncDry pin as a data ready output
    Dry,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
/// Sensor configuration
///
/// See datasheet sections 5.3 - 5.5 for full details.
///
/// The Config::default() matches the sensor's reset config
pub struct Config {
    /// Select the main sensor read mode.
    pub sample_mode: SampleMode,
    /// Select the dry_sync pin's mode.
    pub dry_sync_mode: SyncDryMode,
    /// Include temperature in the output of [Sch16t::get_raw_sample].
    pub include_temperature: bool,
    /// Include the data counters in the output of [Sch16t::get_raw_sample].
    pub include_data_counters: bool,
    /// Include the sensor clock counter in the output of [Sch16t::get_raw_sample].
    pub include_freq_counter: bool,

    /// Gyro dynamic range
    pub rate_range: RateDynamicRange,
    /// Low pass filter setting for gyro x axis
    pub rate_x_lpf: Filter,
    /// Low pass filter setting for gyro y axis
    pub rate_y_lpf: Filter,
    /// Low pass filter setting for gyro z axis
    pub rate_z_lpf: Filter,
    /// Gyro x axis decimation ratio, only applys if [SampleMode] is Decimation
    pub rate_x_dec: DecimationRatio,
    /// Gyro y axis decimation ratio, only applys if [SampleMode] is Decimation
    pub rate_y_dec: DecimationRatio,
    /// Gyro z axis decimation ratio, only applys if [SampleMode] is Decimation
    pub rate_z_dec: DecimationRatio,

    /// Accelerometer dynamic range
    pub accel_range: AccDynamicRange,
    /// Low pass filter setting for accelerometer x axis
    pub accel_x_lpf: Filter,
    /// Low pass filter setting for accelerometer y axis
    pub accel_y_lpf: Filter,
    /// Low pass filter setting for accelerometer z axis
    pub accel_z_lpf: Filter,
    /// Gyro x axis decimation ratio, only applys if [SampleMode] is Decimation
    pub accel_x_dec: DecimationRatio,
    /// Gyro y axis decimation ratio, only applys if [SampleMode] is Decimation
    pub accel_y_dec: DecimationRatio,
    /// Gyro z axis decimation ratio, only applys if [SampleMode] is Decimation
    pub accel_z_dec: DecimationRatio,

    /// If you are using 1.8V interface logic, set this as such
    pub spi_supply: SpiSupply,
    /// By default, the SCH16 operates at a maximum of 10Mhz (as specified by SafeSpi), but can
    /// optionally operate at up to 25Mhz.
    pub hi_speed: HiSpd,
    /// Set the polarity of the dry_sync pin
    pub dry_sync_pol: Polarity,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
#[allow(missing_docs)]
/// Full dump of all status registers
///
/// See datasheet section 7.3 for details
pub struct FullStatus {
    pub sum: StatSumFields,
    pub sum_sat: StatSumSatFields,
    pub com: StatComFields,
    pub rate_com: StatRateComFields,
    pub rate_x: StatRateFields,
    pub rate_y: StatRateFields,
    pub rate_z: StatRateFields,
    pub acc_x: StatAccFields,
    pub acc_y: StatAccFields,
    pub acc_z: StatAccFields,
    pub sync_active: StatSyncActiveFields,
    pub info: StatInfoFields,
}

impl FullStatus {
    fn init_ok(&self) -> bool {
        // only check the init rdy bit, other status is checked in more detail later
        if !self.sum.init_rdy() {
            debug!("Init check failed on sum.init_rdy {:?}", self.sum);
            return false;
        }

        // All bits should be set
        if !self.com.ok() {
            debug!("Init check failed on {:?}", self.com);
            return false;
        }

        // All bits should be set
        if !self.rate_com.ok() {
            debug!("Init check failed on {:?}", self.rate_com);
            return false;
        }

        // Only check self test bits on axis status, ignore saturation
        if !self.rate_x.ok() {
            debug!("Init check failed on {:?}", self.rate_x);
            return false;
        }
        if !self.rate_y.ok() {
            debug!("Init check failed on {:?}", self.rate_y);
            return false;
        }
        if !self.rate_z.ok() {
            debug!("Init check failed on {:?}", self.rate_z);
            return false;
        }
        if !self.acc_x.ok() {
            debug!("Init check failed on {:?}", self.acc_x);
            return false;
        }
        if !self.acc_y.ok() {
            debug!("Init check failed on {:?}", self.acc_y);
            return false;
        }
        if !self.acc_z.ok() {
            debug!("Init check failed on {:?}", self.acc_z);
            return false;
        }

        // Ignore sync and info status registers

        true
    }
}

impl<BUS, DELAY: DelayNs> Sch16t<BUS, DELAY> {
    /// Make a [Sch16t] from a [embedded_hal_async::SpiDevice], [embedded_hal_async::DelayNs] and [AddressPins]
    pub fn new(spidev: BUS, delay: DELAY, address_pins: AddressPins) -> Self {
        let ll = low_level::LowLevel::new(interface::Interface::new(spidev, address_pins));

        Self {
            ll,
            delay,
            config: None,
        }
    }
}

impl<BUS: embedded_hal_async::spi::SpiDevice, DELAY: DelayNs> Sch16t<BUS, DELAY> {
    /// Reset the sensor over spi
    pub async fn reset(&mut self) -> Result<(), Error<BUS::Error>> {
        debug!("Reseting SCH16T");

        self.ll.ctrl_reset().write_async(|_| {}).await
    }

    /// Dump full sensor status
    ///
    /// In general use, errors are reported through the result of [Sch16t::get_raw_sample]
    pub async fn get_full_status(&mut self) -> Result<FullStatus, Error<BUS::Error>> {
        Ok(FullStatus {
            sum: self.ll.stat_sum().read_async().await?,
            sum_sat: self.ll.stat_sum_sat().read_async().await?,
            com: self.ll.stat_com().read_async().await?,
            rate_com: self.ll.stat_rate_com().read_async().await?,
            rate_x: self.ll.stat_rate_x().read_async().await?,
            rate_y: self.ll.stat_rate_y().read_async().await?,
            rate_z: self.ll.stat_rate_z().read_async().await?,
            acc_x: self.ll.stat_acc_x().read_async().await?,
            acc_y: self.ll.stat_acc_y().read_async().await?,
            acc_z: self.ll.stat_acc_z().read_async().await?,
            sync_active: self.ll.stat_sync_active().read_async().await?,
            info: self.ll.stat_info().read_async().await?,
        })
    }

    async fn set_config(&mut self, config: &Config) -> Result<(), Error<BUS::Error>> {
        // TODO: May need to mask off saturation bits for unused channels

        // misc/io config
        self.ll
            .ctrl_user_if()
            .write_async(|r| {
                r.set_spi_supply(config.spi_supply);
                r.set_sync_dec_en(config.dry_sync_mode == SyncDryMode::Sync);
                r.set_sync_intp_en(config.dry_sync_mode == SyncDryMode::Sync);
                r.set_dry_drv_en(config.dry_sync_mode == SyncDryMode::Dry);
                r.set_sync_pol(config.dry_sync_pol);
                r.set_dry_pol(config.dry_sync_pol);
                r.set_miso_hi_speed(config.hi_speed);
                r.set_dry_hi_speed(config.hi_speed);
            })
            .await?;

        // rate config
        self.ll
            .ctrl_rate()
            .write_async(|r| {
                r.set_dyn_xyz_1(config.rate_range);
                r.set_dyn_xyz_2(config.rate_range);
                r.set_dec_x_2(config.rate_x_dec);
                r.set_dec_y_2(config.rate_y_dec);
                r.set_dec_z_2(config.rate_z_dec);
            })
            .await?;
        self.ll
            .ctrl_filt_rate()
            .write_async(|r| {
                r.set_x(config.rate_x_lpf);
                r.set_y(config.rate_y_lpf);
                r.set_z(config.rate_z_lpf);
            })
            .await?;

        // accel config
        self.ll
            .ctrl_acc_12()
            .write_async(|r| {
                r.set_dyn_xyz_1(config.accel_range);
                r.set_dyn_xyz_2(config.accel_range);
                r.set_dec_x_2(config.accel_x_dec);
                r.set_dec_y_2(config.accel_y_dec);
                r.set_dec_z_2(config.accel_z_dec);
            })
            .await?;
        self.ll
            .ctrl_filt_acc_12()
            .write_async(|r| {
                r.set_x(config.accel_x_lpf);
                r.set_y(config.accel_y_lpf);
                r.set_z(config.accel_z_lpf);
            })
            .await?;

        Ok(())
    }

    async fn check_config(&mut self, config: &Config) -> Result<(), Error<BUS::Error>> {
        let user_if = self.ll.ctrl_user_if().read_async().await?;
        if user_if.spi_supply() != config.spi_supply
            || user_if.sync_dec_en() != (config.dry_sync_mode == SyncDryMode::Sync)
            || user_if.sync_intp_en() != (config.dry_sync_mode == SyncDryMode::Sync)
            || user_if.dry_drv_en() != (config.dry_sync_mode == SyncDryMode::Dry)
            || user_if.sync_pol() != config.dry_sync_pol
            || user_if.dry_pol() != config.dry_sync_pol
            || user_if.miso_hi_speed() != config.hi_speed
            || user_if.dry_hi_speed() != config.hi_speed
        {
            debug!("Config check failed on {:?}", user_if);
            return Err(Error::ConfigFailed(self.get_full_status().await?));
        }

        let ctrl_rate = self.ll.ctrl_rate().read_async().await?;
        // the map_err calls here account for the posibility of
        // register values that do not map to our enum
        // other values don't need it because all values are covered
        if ctrl_rate.dyn_xyz_1() != config.rate_range
            || ctrl_rate.dyn_xyz_2() != config.rate_range
            || ctrl_rate.dec_x_2() != config.rate_x_dec
            || ctrl_rate.dec_y_2() != config.rate_y_dec
            || ctrl_rate.dec_z_2() != config.rate_z_dec
        {
            debug!("Config check failed on {:?}", ctrl_rate);
            return Err(Error::ConfigFailed(self.get_full_status().await?));
        }
        let acc_12 = self.ll.ctrl_acc_12().read_async().await?;
        if acc_12.dyn_xyz_1() != config.accel_range
            || acc_12.dyn_xyz_2() != config.accel_range
            || acc_12.dec_x_2() != config.accel_x_dec
            || acc_12.dec_y_2() != config.accel_y_dec
            || acc_12.dec_z_2() != config.accel_z_dec
        {
            debug!("Config check failed on {:?}", acc_12);
            return Err(Error::ConfigFailed(self.get_full_status().await?));
        }

        let filt_rate = self.ll.ctrl_filt_rate().read_async().await?;
        if filt_rate.x() != config.rate_x_lpf
            || filt_rate.y() != config.rate_y_lpf
            || filt_rate.z() != config.rate_z_lpf
        {
            debug!("Config check failed on {:?}", filt_rate);
            return Err(Error::ConfigFailed(self.get_full_status().await?));
        }
        let filt_acc_12 = self.ll.ctrl_filt_acc_12().read_async().await?;
        if filt_acc_12.x() != config.accel_x_lpf
            || filt_acc_12.y() != config.accel_y_lpf
            || filt_acc_12.z() != config.accel_z_lpf
        {
            debug!("Config check failed on {:?}", filt_acc_12);
            return Err(Error::ConfigFailed(self.get_full_status().await?));
        }

        Ok(())
    }

    /// Initialize the sensor and driver with the given config.
    pub async fn init(&mut self, config: Config) -> Result<(), Error<BUS::Error>> {
        // See Figure 8 "start-up sequence"
        // This fn is a direct implementation of that figure.
        // Power should aready be on and stable.

        self.reset().await?;
        debug!("Intializing SCH16T");

        // Delay from figure 8
        self.delay.delay_ms(32).await;

        if config != Config::default() {
            self.set_config(&config).await?;
        }

        self.ll
            .ctrl_mode()
            .write_async(|r| r.set_en_sensor(true))
            .await?;

        // Delay from figure 8
        self.delay.delay_ms(215).await;

        let _ = self.get_full_status().await?;

        self.ll
            .ctrl_mode()
            .write_async(|r| {
                r.set_eoi_ctrl(true);
                r.set_en_sensor(true);
            })
            .await?;

        // Delay from figure 8
        self.delay.delay_ms(3).await;

        let _ = self.get_full_status().await?;

        let init_status = self.get_full_status().await?;
        if !init_status.init_ok() {
            return Err(Error::InitError(init_status));
        }

        // read back and check config
        self.check_config(&config).await?;

        // save the config
        self.config = Some(config);
        let mut read_loop_addrs = match config.sample_mode {
            SampleMode::Interpolate => {
                // These are the interpolated rate and accel registers
                Vec::from_slice(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]).unwrap()
            }
            SampleMode::Decimation => {
                // These are the decimated rate and accel registers
                Vec::from_slice(&[0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]).unwrap()
            }
        };
        if config.include_temperature {
            let _ = read_loop_addrs.push(0x10);
        }
        if config.include_data_counters {
            let _ = read_loop_addrs.extend_from_slice(&[0x11, 0x12]);
        }
        if config.include_freq_counter {
            let _ = read_loop_addrs.extend_from_slice(&[0x11, 0x12]);
        }
        self.ll.interface.set_read_loop_addrs(read_loop_addrs);

        Ok(())
    }

    /// Read a raw sample from the imu.
    ///
    /// Saturation errors are reported in-band in the RawRate and RawAccel structs.
    /// Other sensor errors are reported as errors and the sample is thrown out.
    pub async fn get_raw_sample(&mut self) -> Result<RawSample, Error<BUS::Error>> {
        if let Some(config) = self.config {
            self.ll.interface.clear_sticky_status();

            // Only read either the interp or decimate data registers
            let (r_x, r_y, r_z, a_x, a_y, a_z) = match config.sample_mode {
                SampleMode::Interpolate => {
                    let r_x = self.ll.rate_x_1().read_async().await?.data();
                    let r_y = self.ll.rate_y_1().read_async().await?.data();
                    let r_z = self.ll.rate_z_1().read_async().await?.data();

                    let a_x = self.ll.acc_x_1().read_async().await?.data();
                    let a_y = self.ll.acc_y_1().read_async().await?.data();
                    let a_z = self.ll.acc_z_1().read_async().await?.data();
                    (r_x, r_y, r_z, a_x, a_y, a_z)
                }
                SampleMode::Decimation => {
                    let r_x = self.ll.rate_x_2().read_async().await?.data();
                    let r_y = self.ll.rate_y_2().read_async().await?.data();
                    let r_z = self.ll.rate_z_2().read_async().await?.data();

                    let a_x = self.ll.acc_x_2().read_async().await?.data();
                    let a_y = self.ll.acc_y_2().read_async().await?.data();
                    let a_z = self.ll.acc_z_2().read_async().await?.data();
                    (r_x, r_y, r_z, a_x, a_y, a_z)
                }
            };

            let temp = if config.include_temperature {
                Some(self.ll.temp().read_async().await?.data())
            } else {
                None
            };

            let (r_count, a_count) = if config.include_data_counters {
                let r_dcnt = self.ll.rate_dcnt().read_async().await?;
                let a_dcnt = self.ll.acc_dcnt().read_async().await?;

                (
                    Some(DataCounter {
                        x: r_dcnt.x_dcnt(),
                        y: r_dcnt.y_dcnt(),
                        z: r_dcnt.z_dcnt(),
                    }),
                    Some(DataCounter {
                        x: a_dcnt.x_dcnt(),
                        y: a_dcnt.y_dcnt(),
                        z: a_dcnt.z_dcnt(),
                    }),
                )
            } else {
                (None, None)
            };

            let freq_counter = if config.include_freq_counter {
                let count = self.ll.freq_dcnt().read_async().await?.data();
                Some(count)
            } else {
                None
            };

            let (a_x_sat, a_y_sat, a_z_sat, r_x_sat, r_y_sat, r_z_sat) =
                if self.ll.interface.sticky_status() == SensorStatus::Normal {
                    // If every frame returned normal status,
                    // we know there was no saturation and
                    // don't need to read any more registers.
                    (false, false, false, false, false, false)
                } else {
                    let stat = self.ll.stat_sum().read_async().await?;
                    let sat = self.ll.stat_sum_sat().read_async().await?;

                    if !stat.cmn() {
                        let com = self.ll.stat_com().read_async().await?;
                        if !com.ok() {
                            return Err(Error::CommonError(com));
                        }
                    }

                    // Report axis errors, but ignore saturation, those go in-band with the sample
                    if !stat.rate_x() {
                        let com = self.ll.stat_rate_com().read_async().await?;
                        let axis = self.ll.stat_rate_x().read_async().await?;
                        if !com.ok() || !axis.ok() {
                            return Err(Error::GyroXError((com, axis)));
                        }
                    }
                    if !stat.rate_y() {
                        let com = self.ll.stat_rate_com().read_async().await?;
                        let axis = self.ll.stat_rate_y().read_async().await?;
                        if !com.ok() || !axis.ok() {
                            return Err(Error::GyroYError((com, axis)));
                        }
                    }
                    if !stat.rate_z() {
                        let com = self.ll.stat_rate_com().read_async().await?;
                        let axis = self.ll.stat_rate_z().read_async().await?;
                        if !com.ok() || !axis.ok() {
                            return Err(Error::GyroZError((com, axis)));
                        }
                    }

                    if !stat.acc_x() {
                        let axis = self.ll.stat_acc_x().read_async().await?;
                        if !axis.ok() {
                            return Err(Error::AccXError(axis));
                        }
                    }
                    if !stat.acc_y() {
                        let axis = self.ll.stat_acc_x().read_async().await?;
                        if !axis.ok() {
                            return Err(Error::AccYError(axis));
                        }
                    }
                    if !stat.acc_z() {
                        let axis = self.ll.stat_acc_x().read_async().await?;
                        if !axis.ok() {
                            return Err(Error::AccZError(axis));
                        }
                    }

                    // sort out which saturation bits we care about
                    // and invert them from ok to error bits
                    match config.sample_mode {
                        SampleMode::Interpolate => (
                            !sat.acc_x_1(),
                            !sat.acc_y_1(),
                            !sat.acc_z_1(),
                            !sat.rate_x_1(),
                            !sat.rate_y_1(),
                            !sat.rate_z_1(),
                        ),
                        SampleMode::Decimation => (
                            !sat.acc_x_2(),
                            !sat.acc_y_2(),
                            !sat.acc_z_2(),
                            !sat.rate_x_2(),
                            !sat.rate_y_2(),
                            !sat.rate_z_2(),
                        ),
                    }
                };

            let lsb_ds = match config.rate_range {
                // These really are the same. See DS table 63
                RateDynamicRange::Dyn1 => 1600,
                RateDynamicRange::Dyn2 => 1600,
                RateDynamicRange::Dyn3 => 3200,
                RateDynamicRange::Dyn4 => 6400,
            };

            let lsb_mss = match config.accel_range {
                AccDynamicRange::Dyn1 => 3200,
                AccDynamicRange::Dyn2 => 6400,
                AccDynamicRange::Dyn3 => 12800,
                AccDynamicRange::Dyn4 => 25600,
            };

            let rate = RawRate {
                x: r_x,
                y: r_y,
                z: r_z,
                x_saturated: r_x_sat,
                y_saturated: r_y_sat,
                z_saturated: r_z_sat,
                lsb_ds,
                data_counter: r_count,
            };

            let accel = RawAccel {
                x: a_x,
                y: a_y,
                z: a_z,
                x_saturated: a_x_sat,
                y_saturated: a_y_sat,
                z_saturated: a_z_sat,
                lsb_mss,
                data_counter: a_count,
            };

            Ok(RawSample {
                accel,
                rate,
                temp,
                freq_counter,
            })
        } else {
            Err(Error::SensorNotInit)
        }
    }

    /// Read a raw temperature sample from the sensor
    ///
    /// If you want a temperature measurement with every sample, set `Config.include_temp` to true
    /// and the temperature will be in the `RawSample` given by `get_raw_sample`
    pub async fn get_raw_temperature(&mut self) -> Result<i16, Error<BUS::Error>> {
        Ok(self.ll.temp().read_async().await?.data())
    }
}
