use nes_rs::{prog, CPU};

use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;

use rand::Rng;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl2::init()?;
    let video_subsystem =
        sdl_context
            .video()?;
    let window =
        video_subsystem
            .window("Snake game", (32.0 * 10.0) as u32, (32.0 * 10.0) as u32)
            .position_centered()
            .build()?;

    let mut canvas =
        window
            .into_canvas()
            .present_vsync()
            .build()?;
    canvas.set_scale(10.0, 10.0)?;
    
    let mut event_pump =
        sdl_context
            .event_pump()?;

    let creator =
        canvas
            .texture_creator();
    let mut texture =
        creator
            .create_texture_target(
                PixelFormatEnum::RGB24, 32, 32
            )?;

    let mut cpu = CPU::new();
    cpu.load_program(
        prog::SNAKE
    );
    cpu.interrupt_reset();

    let mut screen_state = [0 as u8; 32 * 3 * 32];
    let mut rng = rand::thread_rng();
    
    cpu.run_with_callback(move |cpu| {
        handle_user_input(cpu, &mut event_pump);
        cpu.load(0xfe, &[rng.gen_range(1..16)]);

        if read_screen_state(cpu, &mut screen_state) {
            texture.update(None, &screen_state, 32 * 3)?;
            canvas.copy(&texture, None, None)?;
            canvas.present();
        }

        ::std::thread::sleep(std::time::Duration::new(0, 70_0000));

        Ok(())
    });

    Ok(())
}

fn handle_user_input(cpu: &mut CPU, event_pump: &mut EventPump) {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => std::process::exit(0),
            Event::KeyDown { keycode: Some(Keycode::W), .. } => cpu.load(MMAP_DPAD_UP as u16, &[0xFF]),
            Event::KeyDown { keycode: Some(Keycode::S), .. } => cpu.load(MMAP_DPAD_DOWN as u16, &[0xFF]),
            Event::KeyDown { keycode: Some(Keycode::A), .. } => cpu.load(MMAP_DPAD_LEFT as u16, &[0xFF]),
            Event::KeyDown { keycode: Some(Keycode::D), .. } => cpu.load(MMAP_DPAD_RIGHT as u16, &[0xFF]),
            _ => {}
        }
    }
}

fn read_screen_state(cpu: &CPU, frame: &mut [u8; 32 * 3 * 32]) -> bool {
    let mut frame_idx = 0;
    let mut update = false;
    for i in 0x0200..0x0600 {
        let color_idx = cpu.read(i);
        let (b1, b2, b3) = color(color_idx).rgb();
        if frame[frame_idx] != b1 || frame[frame_idx + 1] != b2 || frame[frame_idx + 2] != b3 {
            frame[frame_idx] = b1;
            frame[frame_idx + 1] = b2;
            frame[frame_idx + 2] = b3;
            update = true;
        }
        frame_idx += 3
    }
    update
}

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

pub const MMAP_DPAD_UP: u8 = 0x77;
pub const MMAP_DPAD_DOWN: u8 = 0x73;
pub const MMAP_DPAD_LEFT: u8 = 0x61;
pub const MMAP_DPAD_RIGHT: u8 = 0x64;