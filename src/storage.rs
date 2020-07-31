//! Storage 

use hal::storage::{SingleWrite,SingleRead,MultiRead,MultiWrite,Address};
use crate::pac::{flash, FLASH};

struct Flash {
    flash: FLASH
}

impl SingleWrite<u16,u32> for Flash
{
    type Error = Error<E>;
    fn try_write(&mut self, address: Address<u32>, word: u16) -> nb::Result<(), Self::Error> {
        self.try_erase_address(address);

        while self.flash.sr.read().bsy().bit_is_set() {}

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

            // * location
            //let mut x = 0;
            
            //let z = 0;

            unsafe {
                //use core::ptr;
                //let y = &mut USR_PROG as *mut u16;
                //ptr::write_volatile(y, 0xAAAA as u16);
                //Can only set to zero, not to one
                * address.0 = word;
            }
            

            asm::nop();

            while self.flash.sr.read().bsy().bit_is_set() {}
            
        unsafe {

            self.flash.sr.write(|w| {
                w.eop().clear_bit()
            });

            self.flash.cr.write(|w| {
                w.pg().clear_bit();
                w.lock().set_bit()
            });

        }
    }
}



impl MultiWrite<u16,u32> for Flash 
{
    type Error = Error<E>;
    fn try_write_slice(&mut self, address: Address<u32>, buf: &[u16]) -> nb::Result<(), Self::Error> {
        self.try_erase_address(address);

        while self.flash.sr.read().bsy().bit_is_set() {}

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

            // * location
            //let mut x = 0;
            
            //let z = 0;

            unsafe {
                //use core::ptr;
                //let y = &mut USR_PROG as *mut u16;
                //ptr::write_volatile(y, 0xAAAA as u16);
                //Can only set to zero, not to one
                
                for item in buf {
                    * address.0 = item;
                }
            }
            

            asm::nop();

            while self.flash.sr.read().bsy().bit_is_set() {}
            
        unsafe {

            self.flash.sr.write(|w| {
                w.eop().clear_bit()
            });

            self.flash.cr.write(|w| {
                w.pg().clear_bit();
                w.lock().set_bit()
            });

        }        

        Ok(())        
    }
}

impl ErasePage<u32> for Flash 
{
    fn try_erase_page(&mut self, page: Page<u32>) -> nb::Result<(), Self::Error> {
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
                //w.bits(62)
                w.bits(address.0)
            });
    
            self.flash.cr.write(|w| {
                w.per().set_bit();
                w.strt().set_bit()
                //w.bits(0x42)
            });
    
            asm::nop();
    
            while self.flash.sr.read().bsy().bit_is_set() {}
    
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
    
        }

    }

    fn try_erase_address(&mut self, address: Address<u32>) -> nb::Result<(), Self::Error> {
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
                //w.bits(62)
                w.bits(address.0)
            });
    
            self.flash.cr.write(|w| {
                w.per().set_bit();
                w.strt().set_bit()
                //w.bits(0x42)
            });
    
            asm::nop();
    
            while self.flash.sr.read().bsy().bit_is_set() {}
    
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
    
        }

    }

}

impl SingleRead<u8,u32> for Flash 
{
    type Error = Error<E>;
    fn try_read(&mut self, address: Address<u32>) -> nb::Result<u8, Self::Error> {
        self.read_byte(address.0).map(|d| d).map_err(|e| nb::Error::Other(e))
    }
}

impl MultiRead<u8,u32> for Flash
{
    type Error = Error<E>;
    fn try_read_slice(&mut self, address: Address<u32>,  buf: &mut [u8]) -> nb::Result<(), Self::Error> {
        self.read_data(address.0,buf).map(|_| ()).map_err(|e| nb::Error::Other(e))
    }
}
