use super::cartridges::ROM;
use super::joypads::Joypad;
use super::joypads::JoypadButton;
use super::ppu::PPU;
use super::ram::RAM;

const RAM_START: u16 = 0x0000;
const RAM_END: u16 = 0x1FFF;
const PPU_REGISTERS_START: u16 = 0x2000;
const PPU_REGISTERS_END: u16 = 0x3FFF;

pub struct BUS<'call> {
    ram: RAM,
    prg_rom: Vec<u8>,
    ppu: PPU,
    joypad: Joypad,
    joypad2: Joypad,
    cycles: usize,
    gameloop_callback: Box<dyn FnMut(&PPU, &mut Joypad) + 'call>,
}

impl<'a> BUS<'a> {
    pub fn new<'call, F>(rom: ROM, gameloop_callback: F) -> BUS<'call>
    where
        F: FnMut(&PPU, &mut Joypad) + 'call,
    {
        let ppu = PPU::new(rom.chr_rom, rom.screen_mirroring);

        BUS {
            ram: RAM::new(),
            prg_rom: rom.prg_rom,
            ppu,
            joypad: Joypad::new(),
            joypad2: Joypad::new(),
            cycles: 0,
            gameloop_callback: Box::from(gameloop_callback),
        }
    }

    pub fn memory_read(&mut self, address: u16) -> u8 {
        match address {
            RAM_START..=RAM_END => {
                let adjusted_address = address & 0b00000111_11111111;
                self.ram.read(adjusted_address)
            }

            PPU_REGISTERS_START | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => 0,

            0x2002 => self.ppu.read_status(),

            0x2004 => self.ppu.read_oam_data(),

            0x2007 => self.ppu.read_data(),

            0x4000..=0x4015 => {
                //ignore APU
                0
            }

            0x4016 => self.joypad.read(),

            0x4017 => self.joypad2.read(),

            0x2008..=PPU_REGISTERS_END => {
                let adjusted_address = address & 0b00100000_00000111;
                self.memory_read(adjusted_address)
            }

            0x8000..=0xFFFF => self.read_prg_rom(address),

            _ => {
                println!("Ignoring memory access at {}", address);
                0
            }
        }
    }

    pub fn memory_write(&mut self, address: u16, data: u8) {
        match address {
            RAM_START..=RAM_END => {
                let adjusted_address = address & 0b11111111111;
                self.ram.write(adjusted_address, data);
            }

            PPU_REGISTERS_START => self.ppu.write_in_control(data),

            0x2001 => self.ppu.write_to_mask(data),

            0x2002 => panic!("Don't write to PPU status register"),

            0x2003 => self.ppu.write_to_oam_address(data),

            0x2004 => self.ppu.write_to_oam_data(data),

            0x2005 => self.ppu.write_to_scroll(data),

            0x2006 => self.ppu.write_in_ppu_address(data),

            0x2007 => self.ppu.write_in_data(data),

            0x4000..=0x4013 | 0x4015 => {
                //ignore APU
            }

            0x4016 => {
                self.joypad.write(data);
                self.joypad2.write(data);
            }

            0x4017 => {
                self.joypad2.write(data);
            }

            0x4014 => {
                let mut buffer: [u8; 256] = [0; 256];
                let hi: u16 = (data as u16) << 8;
                for i in 0..256u16 {
                    buffer[i as usize] = self.memory_read(hi + i);
                }

                self.ppu.write_oam_dma(&buffer);
            }

            0x2008..=PPU_REGISTERS_END => {
                let adjusted_address = address & 0b00100000_00000111;
                self.memory_write(adjusted_address, data);
            }

            0x8000..=0xFFFF => {
                panic!("Attempt to write to Cartridge ROM space")
            }

            _ => {
                println!("Ignoring memory write-access at {:02x}", address);
            }
        }
    }

    pub fn memory_read_u16(&mut self, address: u16) -> u16 {
        let low = self.memory_read(address) as u16;
        let high = self.memory_read(address + 1) as u16;

        (high << 8) | (low as u16)
    }

    pub fn memory_write_u16(&mut self, address: u16, value: u16) {
        let high = (value >> 8) as u8;
        let low = (value & 0xff) as u8;

        self.memory_write(address, low);
        self.memory_write(address + 1, high);
    }

    pub fn read_prg_rom(&self, mut address: u16) -> u8 {
        address -= 0x8000;

        if self.prg_rom.len() == 0x4000 && address >= 0x4000 {
            address %= 0x4000;
        }

        self.prg_rom[address as usize]
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as usize;

        let new_frame = self.ppu.tick(cycles * 3);

        if new_frame {
            (self.gameloop_callback)(&self.ppu, &mut self.joypad);
        }
    }

    pub fn pool_nmi_status(&mut self) -> Option<u8> {
        self.ppu.pool_nmi_status()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::cartridges::test::test_rom;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn memory_read_0x4016_returns_joypad_state() {
        let called = Rc::new(Cell::new(false));
        let c = called.clone();
        let mut bus = BUS::new(test_rom(), move |_ppu: &PPU, _joypad: &mut Joypad| {
            c.set(true);
        });
        bus.joypad.set_button_pressed_status(JoypadButton::A, true);
        bus.joypad.write(0);
        assert_eq!(bus.memory_read(0x4016), 1);
    }

    #[test]
    fn memory_write_0x4016_strobes_joypad() {
        let mut bus = BUS::new(test_rom(), |_ppu: &PPU, _joypad: &mut Joypad| {});
        bus.joypad.set_button_pressed_status(JoypadButton::A, true);
        bus.memory_write(0x4016, 0);
        assert_eq!(bus.memory_read(0x4016), 1);
    }

    #[test]
    fn tick_invokes_callback_on_frame() {
        let called = Rc::new(Cell::new(false));
        let c = called.clone();
        let mut bus = BUS::new(test_rom(), move |_ppu: &PPU, _joypad: &mut Joypad| {
            c.set(true);
        });
        for _ in 0..200000 {
            bus.tick(1);
            if called.get() {
                break;
            }
        }
        assert!(called.get());
    }
}
