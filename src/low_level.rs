use crc::Algorithm;
use device_driver::Fieldset as _;

device_driver::compile!(
    options: [
        "defmt-feature=defmt"
    ],
    manifest: "sch16t.ddsl"
);

const CRC8_ALG: Algorithm<u8> = Algorithm {
    width: 8,
    poly: 0x2F,
    init: 0xFF,
    refin: false,
    refout: false,
    xorout: 0,
    check: 0,
    residue: 0,
};

const CRC8: crc::Crc<u8> = crc::Crc::<u8>::new(&CRC8_ALG);

impl core::fmt::Display for Filter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "{:?}", self)
    }
}

impl TryFrom<&str> for Filter {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Lpf0" => Ok(Filter::Lpf0),
            "Lpf1" => Ok(Filter::Lpf1),
            "Lpf2" => Ok(Filter::Lpf2),
            "Lpf3" => Ok(Filter::Lpf3),
            "Lpf4" => Ok(Filter::Lpf4),
            "Lpf5" => Ok(Filter::Lpf5),
            "Lpf7" => Ok(Filter::Lpf7),
            _ => Err("Filter values are Lfp0/1/2/3/4/5/7"),
        }
    }
}

impl core::fmt::Display for RateDynamicRange {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "{:?}", self)
    }
}

impl TryFrom<&str> for RateDynamicRange {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Dyn1" => Ok(RateDynamicRange::Dyn1),
            "Dyn2" => Ok(RateDynamicRange::Dyn2),
            "Dyn3" => Ok(RateDynamicRange::Dyn3),
            "Dyn4" => Ok(RateDynamicRange::Dyn4),
            _ => Err("Range values are Dyn1/2/3/4"),
        }
    }
}

impl core::fmt::Display for AccDynamicRange {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "{:?}", self)
    }
}

impl TryFrom<&str> for AccDynamicRange {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Dyn1" => Ok(AccDynamicRange::Dyn1),
            "Dyn2" => Ok(AccDynamicRange::Dyn2),
            "Dyn3" => Ok(AccDynamicRange::Dyn3),
            "Dyn4" => Ok(AccDynamicRange::Dyn4),
            _ => Err("Range values are Dyn1/2/3/4"),
        }
    }
}

impl core::fmt::Display for DecimationRatio {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "{:?}", self)
    }
}

impl TryFrom<&str> for DecimationRatio {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Dec1" => Ok(DecimationRatio::Dec1),
            "Dec2" => Ok(DecimationRatio::Dec2),
            "Dec3" => Ok(DecimationRatio::Dec3),
            "Dec4" => Ok(DecimationRatio::Dec4),
            "Dec5" => Ok(DecimationRatio::Dec5),
            _ => Err("Decimation ratio values are Dec1/2/3/4/5"),
        }
    }
}

impl core::fmt::Display for Polarity {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "{:?}", self)
    }
}

impl TryFrom<&str> for Polarity {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ActiveLow" => Ok(Polarity::ActiveLow),
            "ActiveHigh" => Ok(Polarity::ActiveHigh),
            _ => Err("Polarity values are ActiveLow and ActiveHigh"),
        }
    }
}

impl core::fmt::Display for HiSpd {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "{:?}", self)
    }
}

impl TryFrom<&str> for HiSpd {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Mhz10" => Ok(HiSpd::Mhz10),
            "Mhz25" => Ok(HiSpd::Mhz25),
            _ => Err("HiSpd values are Mhz10 and Mhz25"),
        }
    }
}

impl core::fmt::Display for SpiSupply {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "{:?}", self)
    }
}

impl TryFrom<&str> for SpiSupply {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "V18" => Ok(SpiSupply::V18),
            "V33" => Ok(SpiSupply::V33),
            _ => Err("SpiSupply values are V18 and V33"),
        }
    }
}

impl FrameOut {
    pub(crate) fn compute_crc(&mut self) -> u8 {
        self.set_crc(0);

        let mut digest = CRC8.digest();
        digest.update(&[0x00]);
        digest.update(&self.as_slice_mut()[0..5]);
        let sum = digest.finalize();

        self.set_crc(sum);

        sum
    }
}

impl FrameIn {
    pub(crate) fn compute_crc(&mut self) -> u8 {
        self.set_crc(0);

        let mut digest = CRC8.digest();
        digest.update(&[0x00]);
        digest.update(&self.as_slice_mut()[0..5]);
        let sum = digest.finalize();

        self.set_crc(sum);

        sum
    }

    pub(crate) fn crc_ok(&mut self) -> bool {
        let rx_crc = self.crc();

        let crc = self.compute_crc();
        self.set_crc(rx_crc);

        rx_crc == crc
    }
}

impl StatComFields {
    pub(crate) fn ok(&self) -> bool {
        self.mclk_ok()
            && self.dual_clock_ok()
            && self.dsp_ok()
            && self.svm_ok()
            && self.hv_cp_ok()
            && self.supply_ok()
            && self.temp_ok()
            && self.nmode_ok()
            && self.nvm_sts_ok()
            && self.cmn_sts_ok()
            && self.cmn_sts_rdy()
    }
}
impl StatRateComFields {
    pub(crate) fn ok(&self) -> bool {
        self.pri_agc_ok()
            && self.gyro_pri_ok()
            && self.pri_start_ok()
            && self.gyro_hv_ok()
            && self.gyro_sd_sts_ok()
            && self.gyro_bond_sts_ok()
            && self.gyro_sts_rdy_ok()
    }
}
impl StatRateFields {
    pub(crate) fn ok(&self) -> bool {
        self.stc_dig_ok() && self.stc_ana_ok() && self.qc_ok()
    }
}
impl StatAccFields {
    pub(crate) fn ok(&self) -> bool {
        self.stc_dig_ok() && self.stc_tcap_ok() && self.stc_sdd_ok() && self.stc_n_ok()
    }
}
