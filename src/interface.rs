//#![no_std]
use device_driver::Fieldset as _;

use embedded_hal::spi::Operation;
use heapless::Vec;

use crate::low_level::*;
use crate::{AddressPins, Error};

/// Abstraction for the spi bus.
pub struct Interface<BUS> {
    bus: BUS,
    address: u8,
    last_out_addr: u8,
    sticky_status: SensorStatus,
    read_loop_addrs: Vec<u8, 10>,
}

impl<BUS> Interface<BUS> {
    pub fn new(bus: BUS, address: AddressPins) -> Self {
        Self {
            bus,
            address: address as u8,
            last_out_addr: 0,
            sticky_status: SensorStatus::Initialization,
            read_loop_addrs: Vec::new(),
        }
    }

    pub(crate) fn sticky_status(&mut self) -> SensorStatus {
        self.sticky_status
    }

    pub(crate) fn clear_sticky_status(&mut self) {
        self.sticky_status = SensorStatus::Normal
    }

    pub(crate) fn set_read_loop_addrs(&mut self, addrs: Vec<u8, 10>) {
        self.read_loop_addrs = addrs;
    }

    fn next_addr(&self, current_addr: u8) -> u8 {
        if let Some(pos) = self.read_loop_addrs.iter().position(|&r| r == current_addr) {
            let next_pos = pos + 1;
            if next_pos == self.read_loop_addrs.len() {
                if self.sticky_status != SensorStatus::Normal {
                    0x14
                } else {
                    self.read_loop_addrs[0]
                }
            } else {
                self.read_loop_addrs[next_pos]
            }
        } else {
            current_addr + 1
        }
    }

    fn make_frame(&self, addr: u8, write: bool, data: &[u8]) -> FrameOut {
        let mut frame = FrameOut::default();
        frame.set_chip_address(self.address);
        frame.set_register_address(addr);
        frame.set_write_en(write);
        frame.set_frame_type(true);

        let mut buf = [0u8; 4];
        buf[..data.len()].copy_from_slice(&data);
        frame.set_data(u32::from_le_bytes(buf));
        frame.compute_crc();

        frame
    }
}

impl<BUS: embedded_hal_async::spi::SpiDevice> Interface<BUS> {
    async fn transfer_frame(
        &mut self,
        addr: u8,
        write: bool,
        data: &mut [u8],
    ) -> Result<(), Error<BUS::Error>> {
        let mut frame = self.make_frame(addr, write, data);
        trace!("frame out: {:?}", frame);
        let mut ops = [Operation::TransferInPlace(frame.as_slice_mut())];

        self.bus.transaction(&mut ops).await.map_err(Error::Bus)?;

        let mut frame =
            FrameIn::from(<&[u8] as TryInto<[u8; 6]>>::try_into(frame.as_slice_mut()).unwrap());
        trace!("frame in: {:?}", frame);

        if frame.register_address() != 0 && !frame.crc_ok() {
            return Err(Error::Crc);
        }

        let last_status = frame.sensor_status();

        // Check if most recent status is worse than the sticky status and upgrade if so
        match (self.sticky_status, last_status) {
            (SensorStatus::Normal | SensorStatus::Initialization, _) => {
                self.sticky_status = last_status
            }
            (SensorStatus::Saturation, SensorStatus::SensorError) => {
                self.sticky_status = last_status
            }
            _ => {}
        }

        self.last_out_addr = addr;
        data.copy_from_slice(&frame.data().to_le_bytes()[..data.len()]);

        Ok(())
    }
}

impl<BUS: embedded_hal_async::spi::SpiDevice> device_driver::RegisterInterfaceBase
    for Interface<BUS>
{
    type Error = Error<BUS::Error>;
    type AddressType = u8;
}

impl<BUS: embedded_hal_async::spi::SpiDevice> device_driver::AsyncRegisterInterface
    for Interface<BUS>
{
    async fn write_register(
        &mut self,
        address: Self::AddressType,
        data: &mut [u8],
        _metadata: &device_driver::FieldsetMetadata,
    ) -> Result<(), Self::Error> {
        self.transfer_frame(address, true, data).await?;

        Ok(())
    }

    async fn read_register(
        &mut self,
        address: Self::AddressType,
        data: &mut [u8],
        _metadata: &device_driver::FieldsetMetadata,
    ) -> Result<(), Self::Error> {
        // If the last read wasn't for our address, send a frame to set the address,
        // discard the return frame.
        if self.last_out_addr != address {
            self.transfer_frame(address, false, data).await?;
        };

        // Read a frame, guess that our next read will be the same frame size at the next address
        let next_addr = self.next_addr(address);
        self.transfer_frame(next_addr, false, data).await?;

        Ok(())
    }
}
