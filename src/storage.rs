//! Storage 

use hal::storage::*;
use crate::pac::{FLASH};
use nb;
use cortex_m::asm;

enum WriteMode {
    EraseStart,
    EraseEnd,
    WriteStart,
    WriteEnd
}

pub struct Flash {
    flash: FLASH,
    write_mode: Option<WriteMode>
}

pub enum FlashError {}

impl SingleWrite<u16,u32> for Flash 
{
    type Error = FlashError;
    fn try_write(&mut self, address: Address<u32>, word: u16) -> nb::Result<(), Self::Error> {
        let mut buf = [word]; 
        self.try_write_slice(address,&mut buf)?;
        Ok(())
    }
}

impl MultiWrite<u16,u32> for Flash
{
    type Error = FlashError;
    fn try_write_slice(&mut self, address: Address<u32>, buf: &mut [u16]) -> nb::Result<(), Self::Error> {
        use WriteMode::*;
        match self.write_mode {
            Some(EraseStart) | Some(EraseEnd) => {
                self.try_erase_address(address)?;
                self.write_mode = Some(WriteStart);
                
                Err(nb::Error::WouldBlock)
            },
            Some(WriteStart) => {
                if self.flash.sr.read().bsy().bit_is_set() {
                    return Err(nb::Error::WouldBlock);
                }
                unsafe {
                    self.flash.keyr.write(|w| {
                        w.bits(0x45670123)
                    });
                    self.flash.keyr.write(|w| {
                        w.bits(0xCDEF89AB)
                    });
        
                    self.flash.cr.write(|w| {
                        w.pg().set_bit()
                    });
                }
                
                let mut start_address = address.0 as *mut _;
                
                for item in buf {
                    unsafe {
                        use core::ptr;
                        ptr::write_volatile(start_address, *item);

                        //16 bit word, but 8 byte addressing
                        start_address = start_address.offset(2);
                    }
                    
                }
                
                
                asm::nop();
                self.write_mode = Some(WriteEnd);
                Err(nb::Error::WouldBlock)
            },
            Some(WriteEnd) => {
                if self.flash.sr.read().bsy().bit_is_set() {
                    return Err(nb::Error::WouldBlock);
                }
                    
                //unsafe {
        
                    self.flash.sr.write(|w| {
                        w.eop().clear_bit()
                    });
        
                    self.flash.cr.write(|w| {
                        w.pg().clear_bit();
                        w.lock().set_bit()
                    });
        
                //}
                self.write_mode = None;
                Ok(())
            },
            None => Ok(())
        }

    }
}


impl ErasePage<u32> for Flash 
{
    type Error = FlashError;
    fn try_erase_page(&mut self, page: Page<u32>) -> nb::Result<(), Self::Error> {
        //convert the page ID to an address
        let address = page.0*self.try_page_size(Address(0))?.0+self.try_start_address()?.0;
        let address = Address(address);

        self.try_erase_address(address)
    }

    fn try_erase_address(&mut self, address: Address<u32>) -> nb::Result<(), Self::Error> {
        use WriteMode::*;
        match self.write_mode {
            Some(EraseStart) => {
                unsafe {

                    self.flash.keyr.write(|w| {
                        w.bits(0x45670123)
                    });
                    self.flash.keyr.write(|w| {
                        w.bits(0xCDEF89AB)
                    });
            
                    self.flash.cr.write(|w| {
                        //w.per().set_bit()
                        w.bits(0x2)
                    });
            
                    self.flash.ar.write(|w| {
                        w.bits(address.0)
                    });
            
                    self.flash.cr.write(|w| {
                        w.per().set_bit();
                        w.strt().set_bit()
                    });
            
                    asm::nop();
                }
                self.write_mode = Some(EraseEnd);
                Err(nb::Error::WouldBlock)
            },
            Some(EraseEnd) => {
                if self.flash.sr.read().bsy().bit_is_set() {
                    return Err(nb::Error::WouldBlock);
                }
                
                if self.flash.sr.read().eop().bit_is_set() {

                    self.flash.sr.write(|w| {
                        w.eop().clear_bit()
                    });
        
                    self.flash.cr.write(|w| {
                        w.per().clear_bit()
                    });
                    
                }

                self.flash.cr.write(|w| {
                        w.lock().set_bit()
                    });
        
        
                self.write_mode = None;
                Ok(())
            },
            //Other modes are not relevant so will be ignored
            _ => {
                Ok(())
            }
        }
    }

}

impl SingleRead<u8,u32> for Flash 
{
    type Error = FlashError;
    fn try_read(&mut self, address: Address<u32>) -> nb::Result<u8, Self::Error> {
        let mut buf = [0]; 
        self.try_read_slice(address,&mut buf)?;
        Ok(buf[0])
    }
}

#[allow(unused_assignments)]
impl MultiRead<u8,u32> for Flash
{
    type Error = FlashError;
    fn try_read_slice(&mut self, address: Address<u32>, mut buf: &mut [u8]) -> nb::Result<(), Self::Error> {
        let address = address.0 as *mut _;
        unsafe {
            
            buf = core::slice::from_raw_parts_mut::<'static, u8>(address,buf.len());
        }
        
        Ok(()) 
    }
}

impl StorageSize<u8, u32> for Flash {
    type Error = FlashError;

    fn try_start_address(&mut self) -> nb::Result<Address<u32>, Self::Error> {
        Ok(Address(0x0800_0000))
    }

    fn try_total_size(&mut self) -> nb::Result<AddressOffset<u32>, Self::Error> {
        Ok(AddressOffset(0))
    }

    /// 2KB
    fn try_page_size(&mut self, _address: Address<u32>) -> nb::Result<AddressOffset<u32>, Self::Error> {
        Ok(AddressOffset(2048))
    }
}