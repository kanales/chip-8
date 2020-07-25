mod lib;

use lib::machine::Machine;
use lib::screen::{HEIGHT, WIDTH};
use std::env;
use std::fs::File;
use std::io::Read;

extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use std::time::Duration;

const SCALE: usize = 20;

fn machine() -> Machine {
    let args: Vec<String> = env::args().collect();

    if args.len() < 1 {
        panic!("Expected filename");
    }

    let filename = &args[1];
    let mut file = File::open(filename).unwrap();
    let mut rom = Vec::new();

    file.read_to_end(&mut rom).unwrap();

    Machine::new(&rom)
}

fn clear(canvas: &mut WindowCanvas) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
}

fn draw(canvas: &mut WindowCanvas, buffer: &[u8]) {
    let mut blacks = Vec::new();
    let mut whites = Vec::new();
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let b = buffer[y * WIDTH + x];
            let x = x * SCALE;
            let y = y * SCALE;
            let r = Rect::new(x as i32, y as i32, SCALE as u32, SCALE as u32);
            if b == 0 {
                blacks.push(r);
            } else {
                whites.push(r);
            }
        }
    }
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.fill_rects(&blacks).unwrap();
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.fill_rects(&whites).unwrap();
}

fn kc_as_u8(kc: Keycode) -> Option<u8> {
    match kc {
        Keycode::A => Some(0xA),
        Keycode::B => Some(0xB),
        Keycode::C => Some(0xC),
        Keycode::D => Some(0xD),
        Keycode::E => Some(0xE),
        Keycode::F => Some(0xF),
        Keycode::Num0 => Some(0x0),
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0x4),
        Keycode::Num5 => Some(0x5),
        Keycode::Num6 => Some(0x6),
        Keycode::Num7 => Some(0x7),
        Keycode::Num8 => Some(0x8),
        Keycode::Num9 => Some(0x9),
        _ => None,
    }
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("chip-8", (WIDTH * SCALE) as u32, (HEIGHT * SCALE) as u32)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut machine = machine();
    let mut presses = Vec::new();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(kc), ..
                } => {
                    if let Some(k) = kc_as_u8(kc) {
                        if !presses.contains(&k) {
                            presses.push(k);
                        }
                    }
                }
                Event::KeyUp {
                    keycode: Some(kc), ..
                } => {
                    if let Some(k) = kc_as_u8(kc) {
                        let index = presses.iter().position(|x| *x == k).unwrap();
                        presses.remove(index);
                    }
                }
                _ => {}
            }
        }
        machine.key_pressed(&presses);
        match machine.step() {
            Ok(Some(b)) => {
                clear(&mut canvas);
                draw(&mut canvas, b);
                canvas.present();
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("{:?}", e);
                break 'running;
            }
        }
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
