use embedded_hal_async::delay::DelayNs;
use uom::si::{
    acceleration::meter_per_second_squared,
    angular_velocity::degree_per_second,
    f32::{Acceleration, AngularVelocity, ThermodynamicTemperature},
    thermodynamic_temperature::degree_celsius,
};

use super::*;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug)]
#[allow(missing_docs)]
/// Acceleration data in UOM si f32
pub struct Accel {
    pub x: Acceleration,
    pub y: Acceleration,
    pub z: Acceleration,
    pub x_saturated: bool,
    pub y_saturated: bool,
    pub z_saturated: bool,
    pub data_counter: Option<DataCounter>,
}

impl Accel {
    /// Convience fn to check if any axis is saturated
    pub fn saturated(&self) -> bool {
        self.x_saturated || self.y_saturated || self.z_saturated
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug)]
#[allow(missing_docs)]
/// Gyro data in UOM si f32
pub struct Rate {
    pub x: AngularVelocity,
    pub y: AngularVelocity,
    pub z: AngularVelocity,
    pub x_saturated: bool,
    pub y_saturated: bool,
    pub z_saturated: bool,
    pub data_counter: Option<DataCounter>,
}

impl Rate {
    /// Convience fn to check if any axis is saturated
    pub fn saturated(&self) -> bool {
        self.x_saturated || self.y_saturated || self.z_saturated
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug)]
#[allow(missing_docs)]
/// Full sample in UOM si f32
pub struct Sample {
    pub accel: Accel,
    pub rate: Rate,
    pub temp: Option<ThermodynamicTemperature>,
    pub freq_counter: Option<u16>,
}

impl From<RawSample> for Sample {
    fn from(raw_sample: RawSample) -> Sample {
        let temp = if let Some(raw) = raw_sample.temp {
            Some(ThermodynamicTemperature::new::<degree_celsius>(
                raw as f32 / 100.0,
            ))
        } else {
            None
        };

        Sample {
            accel: Accel {
                x: Acceleration::new::<meter_per_second_squared>(
                    raw_sample.accel.x as f32 / raw_sample.accel.lsb_mss as f32,
                ),
                y: Acceleration::new::<meter_per_second_squared>(
                    raw_sample.accel.y as f32 / raw_sample.accel.lsb_mss as f32,
                ),
                z: Acceleration::new::<meter_per_second_squared>(
                    raw_sample.accel.z as f32 / raw_sample.accel.lsb_mss as f32,
                ),
                x_saturated: raw_sample.accel.x_saturated,
                y_saturated: raw_sample.accel.y_saturated,
                z_saturated: raw_sample.accel.z_saturated,
                data_counter: raw_sample.accel.data_counter,
            },
            rate: Rate {
                x: AngularVelocity::new::<degree_per_second>(
                    raw_sample.rate.x as f32 / raw_sample.rate.lsb_ds as f32,
                ),
                y: AngularVelocity::new::<degree_per_second>(
                    raw_sample.rate.y as f32 / raw_sample.rate.lsb_ds as f32,
                ),
                z: AngularVelocity::new::<degree_per_second>(
                    raw_sample.rate.z as f32 / raw_sample.rate.lsb_ds as f32,
                ),
                x_saturated: raw_sample.rate.x_saturated,
                y_saturated: raw_sample.rate.y_saturated,
                z_saturated: raw_sample.rate.z_saturated,
                data_counter: raw_sample.rate.data_counter,
            },
            temp,
            freq_counter: raw_sample.freq_counter,
        }
    }
}

impl<BUS: embedded_hal_async::spi::SpiDevice, DELAY: DelayNs> Sch16t<BUS, DELAY> {
    /// Read a sample from the imu and return the values in UOM types.
    ///
    /// Saturation errors are reported in-band in the Rate and Accel structs.
    /// Other sensor errors are reported as errors and the sample is thrown out.
    pub async fn get_sample(&mut self) -> Result<Sample, Error<BUS::Error>> {
        let raw_sample = self.get_raw_sample().await?;

        Ok(raw_sample.into())
    }

    /// Read the temperature from the imu and return the value as ThermodynamicTemperature
    ///
    /// Saturation errors are reported in-band in the Rate and Accel structs.
    /// Other sensor errors are reported as errors and the sample is thrown out.
    pub async fn get_temperature(&mut self) -> Result<ThermodynamicTemperature, Error<BUS::Error>> {
        let raw = self.get_raw_temperature().await?;

        Ok(ThermodynamicTemperature::new::<degree_celsius>(
            raw as f32 / 100.0,
        ))
    }
}
