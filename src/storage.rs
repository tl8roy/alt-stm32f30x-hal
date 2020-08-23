//! Storage 

use hal::storage::{SingleWrite,SingleRead,MultiRead,MultiWrite,Address};
use crate::pac::{flash, FLASH};

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

pub impl SingleWrite<u16,u32> for Flash 
{
    type Error = Error<E>;
    fn try_read(&mut self, address: Address<u32>, word: u16) -> nb::Result<(), Self::Error> {
        let mut buf: [u16] = [word]; 
        self.try_write_slice(address,&buf);
        Ok()
    }
}

pub impl MultiWrite<u16,u32> for Flash
{
    type Error = Error<E>;
    fn try_write_slice(&mut self, address: Address<u32>, buf: &[u16]) -> nb::Result<(), Self::Error> {
        use WriteMode::*;
        match self.write_mode {
            Some(EraseStart) | Some(EraseEnd) => {
                self.try_erase_address(address)?;
                self.write_mode = Some(WriteStart);
                
                Error::WouldBlock
            },
            Some(WriteStart) => {
                if self.flash.sr.read().bsy().bit_is_set() {
                    return Error::WouldBlock;
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
                
                let mut start_address = address.0;
                
                for item in buf {
                    unsafe {
                        * start_address = item;
                    }
                    //16 bit word, but 8 byte addressing
                    start_address += 2;
                }
                
                
                asm::nop();
                self.write_mode = Some(WriteEnd);
                Error::WouldBlock
            },
            Some(WriteEnd) => {
                if self.flash.sr.read().bsy().bit_is_set() {
                    return Error::WouldBlock;
                }
                    
                unsafe {
        
                    self.flash.sr.write(|w| {
                        w.eop().clear_bit()
                    });
        
                    self.flash.cr.write(|w| {
                        w.pg().clear_bit();
                        w.lock().set_bit()
                    });
        
                }
                self.write_mode = None;
            },
            None => Ok()
        }

    }
}


pub impl ErasePage<u32> for Flash 
{
    fn try_erase_page(&mut self, page: Page<u32>) -> nb::Result<(), Self::Error> {
        //convert the page ID to an address
        let address = page.0*self.try_page_size()?+self.try_start_address()?;
        let address = Address(address);

        self.try_erase_address(address)
    }

    fn try_erase_address(&mut self, address: Address<u32>) -> nb::Result<(), Self::Error> {
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
                Error::WouldBlock
            },
            Some(EraseEnd) => {
                if self.flash.sr.read().bsy().bit_is_set() {
                    return Error::WouldBlock;
                }
                
                if self.flash.sr.read().eop().bit_is_set() {
                    unsafe {
                        self.flash.sr.write(|w| {
                            w.eop().clear_bit()
                        });
            
                        self.flash.cr.write(|w| {
                            w.per().clear_bit()
                        });
                    }
                }
                unsafe {
                    self.flash.cr.write(|w| {
                            w.lock().set_bit()
                        });
                }
        
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

pub impl SingleRead<u8,u32> for Flash 
{
    type Error = Error<E>;
    fn try_read(&mut self, address: Address<u32>) -> nb::Result<u8, Self::Error> {
        let mut buf: [u8] = [0]; 
        self.try_read_slice(address.0,&buf);
        Ok(buf[0])
    }
}

pub impl MultiRead<u8,u32> for Flash
{
    type Error = Error<E>;
    fn try_read_slice(&mut self, address: Address<u32>,  buf: &mut [u8]) -> nb::Result<(), Self::Error> {
        let address = address.0 as *const _;
        unsafe {
             buf = core::slice::from_raw_parts::<'static, u8>(address,buf.len())
        }
        
        Ok() 
    }
}

pub impl StorageSize<u8, u32> for Flash {
    type Error = Error<E>;

    fn try_start_address(&mut self) -> nb::Result<Address<u32>, Self::Error> {
        Ok(Address(0x0800_0000))
    }

    fn try_total_size(&mut self) -> nb::Result<AddressOffset<u32>, Self::Error> {
        Ok(AddressOffset(0))
    }

    /// 2KB
    fn try_page_size(&mut self) -> nb::Result<AddressOffset<u32>, Self::Error> {
        Ok(AddressOffset(2048))
    }
}