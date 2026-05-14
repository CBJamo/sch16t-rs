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
