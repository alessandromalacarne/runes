use bitflags::bitflags;

bitflags! {
    pub struct JoypadButton: u8 {
        const A      = 0b0000_0001;
        const B      = 0b0000_0010;
        const SELECT = 0b0000_0100;
        const START  = 0b0000_1000;
        const UP     = 0b0001_0000;
        const DOWN   = 0b0010_0000;
        const LEFT   = 0b0100_0000;
        const RIGHT  = 0b1000_0000;
    }
}

pub struct Joypad {
    strobe: bool,
    button_index: u8,
    button_status: JoypadButton,
}

impl Default for Joypad {
    fn default() -> Self {
        Self::new()
    }
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            strobe: false,
            button_index: 0,
            button_status: JoypadButton::empty(),
        }
    }

    pub fn write(&mut self, value: u8) {
        self.strobe = value & 1 == 1;
        if self.strobe {
            self.button_index = 0;
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.button_index > 7 {
            return 1;
        }

        let value = (self.button_status.bits() & (1 << self.button_index)) >> self.button_index;

        if !self.strobe && self.button_index <= 7 {
            self.button_index += 1;
        }

        value
    }

    pub fn set_button_pressed_status(&mut self, button: JoypadButton, pressed: bool) {
        self.button_status.set(button, pressed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strobe_resets_index() {
        let mut joypad = Joypad::new();
        joypad.set_button_pressed_status(JoypadButton::A, true);
        joypad.write(1);
        for _ in 0..10 {
            assert_eq!(joypad.read(), 1);
        }
    }

    #[test]
    fn serial_read_returns_buttons_in_order() {
        let mut joypad = Joypad::new();
        joypad.set_button_pressed_status(JoypadButton::A, true);
        joypad.set_button_pressed_status(JoypadButton::RIGHT, true);
        joypad.write(0);

        // A=1, B=0, Select=0, Start=0, Up=0, Down=0, Left=0, Right=1
        assert_eq!(joypad.read(), 1); // A
        assert_eq!(joypad.read(), 0); // B
        assert_eq!(joypad.read(), 0); // Select
        assert_eq!(joypad.read(), 0); // Start
        assert_eq!(joypad.read(), 0); // Up
        assert_eq!(joypad.read(), 0); // Down
        assert_eq!(joypad.read(), 0); // Left
        assert_eq!(joypad.read(), 1); // Right
                                      // 9th read returns 1
        assert_eq!(joypad.read(), 1);
    }

    #[test]
    fn seventh_button_released_returns_zero() {
        // Kills the `>` -> `==`/`>=` mutation: at index 7 the real
        // implementation must return the button state, not the "past end" value.
        let mut joypad = Joypad::new();
        // only A pressed, so index 7 (Right) is released
        joypad.set_button_pressed_status(JoypadButton::A, true);
        joypad.write(0);
        for _ in 0..7 {
            let _ = joypad.read();
        }
        assert_eq!(joypad.read(), 0);
    }

    #[test]
    fn release_clears_button() {
        let mut joypad = Joypad::new();
        joypad.set_button_pressed_status(JoypadButton::A, true);
        joypad.write(0);
        assert_eq!(joypad.read(), 1);
        joypad.set_button_pressed_status(JoypadButton::A, false);
        // index advanced; reset via strobe
        joypad.write(1);
        joypad.write(0);
        assert_eq!(joypad.read(), 0);
    }
}
