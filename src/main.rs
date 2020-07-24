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

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

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
