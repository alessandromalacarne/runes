use components::bus::BUS;
use components::cartridges::ROM;
use components::cpu::CPU;
use components::joypads::JoypadButton;
use components::ppu::render;
use components::ppu::Frame;
use components::ppu::PPU;
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
};

pub mod components;
pub mod tile_viewer;
pub mod trace;

fn color(byte: u8) -> Color {
    match byte {
        0 => sdl2::pixels::Color::BLACK,
        1 => sdl2::pixels::Color::WHITE,
        2 | 9 => sdl2::pixels::Color::GREY,
        3 | 10 => sdl2::pixels::Color::RED,
        4 | 11 => sdl2::pixels::Color::GREEN,
        5 | 12 => sdl2::pixels::Color::BLUE,
        6 | 13 => sdl2::pixels::Color::MAGENTA,
        7 | 14 => sdl2::pixels::Color::YELLOW,
        _ => sdl2::pixels::Color::CYAN,
    }
}

fn read_screen_state(cpu: &mut CPU, frame: &mut [u8; 32 * 3 * 32]) -> bool {
    let mut frame_index = 0;
    let mut update = false;
    for i in 0x0200..0x600 {
        let color_index = cpu.memory_read(i as u16);
        let (r, g, b) = color(color_index).rgb();
        if frame[frame_index] != r || frame[frame_index + 1] != g || frame[frame_index + 2] != b {
            frame[frame_index] = r;
            frame[frame_index + 1] = g;
            frame[frame_index + 2] = b;
            update = true;
        }
        frame_index += 3;
    }
    update
}

fn key_to_button(key: Keycode) -> Option<JoypadButton> {
    match key {
        Keycode::Z => Some(JoypadButton::A),
        Keycode::X => Some(JoypadButton::B),
        Keycode::Return => Some(JoypadButton::START),
        Keycode::Backspace => Some(JoypadButton::SELECT),
        Keycode::Up | Keycode::W => Some(JoypadButton::UP),
        Keycode::Down | Keycode::S => Some(JoypadButton::DOWN),
        Keycode::Left | Keycode::A => Some(JoypadButton::LEFT),
        Keycode::Right | Keycode::D => Some(JoypadButton::RIGHT),
        _ => None,
    }
}

pub fn start(game: &str) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Tile viewer", (256.0 * 3.0) as u32, (240.0 * 3.0) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(3.0, 3.0).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();
    let path_to_game = format!("games/{}.nes", game);
    let bytes: Vec<u8> = std::fs::read(path_to_game).unwrap();
    let rom = ROM::new(&bytes).unwrap();

    let mut frame = Frame::new();

    let bus = BUS::new(
        rom,
        move |ppu: &PPU, joypad: &mut components::joypads::Joypad| {
            render(ppu, &mut frame);

            texture.update(None, frame.get_data(), 256 * 3).unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => {
                        std::process::exit(0);
                    }

                    Event::KeyDown {
                        keycode: Some(key), ..
                    } => {
                        if let Some(button) = key_to_button(key) {
                            joypad.set_button_pressed_status(button, true);
                        }
                    }

                    Event::KeyUp {
                        keycode: Some(key), ..
                    } => {
                        if let Some(button) = key_to_button(key) {
                            joypad.set_button_pressed_status(button, false);
                        }
                    }

                    _ => {}
                }
            }
        },
    );

    let mut cpu = CPU::new(bus);
    cpu.reset();
    cpu.run();
}
