use core::str;
use std::net::TcpStream;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::io::Result;
use std::env;
use evalexpr::*;
mod pixel;
mod ext;
use pixel::pixel::*;
use ext::ext::*;

fn print_help() -> () {
    println!(
        "Usage: pxf [OPTIONS]\n\n\
        OPTIONS:\n\t\
        -noise\t\t\tDraw random pixels\
        \n\t-fill\t\t\tFill background with random color\
        \n\t-pattern\t\tDraw a pattern (formulas \
         currently broken)\n\t\t [\"RF\", \"GF\", \"BF\", \"AF\"]\n\t\
         -help\t\t\tShow this help message\n\
         \t-dyn\t\t\tGet canvas size from server instead of using hardcoded aspect\n\
         \t-capture\t\tCapture canvas and save to file [UNFINISHED]\n\
         \t-from [FILENAME]\tDraw from file [UNFINISHED]\n\
         \t-slice\t\t\tDraw slices across the canvas\n"
    );
}

fn main() -> Result<()> {
    let col: String = Color::new(255, 255, 255, 255).to_str();
    let default_col: Color = Color::from_str(&col);
    let args: Vec<String> = env::args().collect();

    let mut generate_noise: bool    = false;
    let mut fill_back: bool         = false;
    let mut patternize: bool        = false;
    let mut slice: bool             = false;
    let mut pattern_eval: bool      = false;
    let mut dyn_size: bool          = false;
    let mut capture: bool           = false;
    let mut file: &str              =    "";
    let mut r_formula: &str         =    "";
    let mut g_formula: &str         =    "";
    let mut b_formula: &str         =    "";
    let mut a_formula: &str         =    "";

    match args.len() {
        1 => println!("No arguments given, using default values"),
        2 => {
            match args[1].as_str() {
                "-help"     => { print_help(); return Ok(()); },
                "-noise"    => generate_noise   = true,
                "-fill"     => fill_back        = true,
                "-dyn"      => dyn_size         = true,
                "-pattern"  => patternize       = true,
                "-slice"    => slice            = true,
                "-capture"  => capture          = true,
                _ => println!("Invalid argument, using default values"),
            }
        },
        3 => {
            match args[1].as_str() {
                "-noise"    => generate_noise   = true,
                "-fill"     => fill_back        = true,
                "-dyn"      => dyn_size         = true,
                "-slice"    => slice            = true,
                "-from"     => file             = &args[2],
                _ => println!("Invalid arguments, using default values"),
            }
            match args[2].as_str() {
                "-noise"    => generate_noise   = true,
                "-fill"     => fill_back        = true,
                "-dyn"      => dyn_size         = true,
                "-slice"    => slice            = true,
                _ => println!("Invalid arguments, using default values"),
            }
        },
        6 => {
            match args[1].as_str() {
                "-pattern" => {
                    patternize              = true;
                    pattern_eval            = true;
                    r_formula               = &args[2];
                    g_formula               = &args[3];
                    b_formula               = &args[4];
                    a_formula               = &args[5];
                },
                _ => println!("Invalid arguments, using default values"),
            }
        }
        _ => println!("Invalid argument count, using default values"),
    }

    let w: u16;
    let h: u16;
    
    match dyn_size {
        true    => (w, h) = dyn_size_get()?,
        false   => (w, h) = (WIDTH, HEIGHT),
    };

    if capture {
        capture_canvas()?;
        return Ok(());
    }

    if file != "" {
        draw_from_file(file)?;
        return Ok(());
    }

    if slice {
        let mut pixelmap: Vec<Pixel> = Vec::new();
        for x in 0..w {
            for y in 0..h {
                let pixel = Pixel::new(x, y, Color::new(
                        (x+y) as u8, (x+y) as u8, (x+y) as u8, (x+y) as u8
                ));
                pixelmap.push(pixel);
            }
        }
        let stream: Arc<Mutex<TcpStream>> = Arc::new(
            Mutex::new(
                TcpStream::connect(HOST)?
            )
        );
        for pixel in pixelmap {
            stream
            .lock()
            .unwrap()
            .write(
                pixel
                .to_string()
                .as_bytes()
            )?;
        }
        return Ok(());
    }

    let mut iteration: u16 = 0;
    loop { println!("Iteration {}", iteration);
        let bgc = random_color(false);
        let mut pixelmap: Vec<Pixel> = Vec::new();
        for x in 0..w {
            for y in 0..h {
                let pixel: Pixel = match fill_back && generate_noise {
                    true    => Pixel::new(x, y, combine_colors(random_color(true), bgc)),
                    false   => 
                        match fill_back || generate_noise {
                            true    => 
                                match generate_noise {
                                    true    => Pixel::new(x, y, random_color(true)),
                                    false   => Pixel::new(x, y, bgc),
                                },
                            false   => 
                                match patternize {
                                    true    => 
                                        match pattern_eval {
                                            true    => Pixel::new(x, y, Color::new(
                                                (eval_int(r_formula)).unwrap() as u8 % 255,
                                                (eval_int(g_formula)).unwrap() as u8 % 255,
                                                (eval_int(b_formula)).unwrap() as u8 % 255, 
                                                (eval_int(a_formula)).unwrap() as u8 % 255,
                                            )),
                                            false   => Pixel::new(x, y, Color::new(
                                                ((x * x) + (y * y) + 2 * iteration) as u8 % 255,
                                                ((x * x) + (y * y) + iteration)     as u8 % 255,
                                                ((x * x) + (y * y))                 as u8 % 255, 
                                                ((x * x) + (y * y))                 as u8 % 255,
                                            )),
                                        },
                                    false => Pixel::new(x, y, default_col),
                            },
                        },
                };
                pixelmap.push(pixel);
            }
        }
        let stream: Arc<Mutex<TcpStream>> = Arc::new(
            Mutex::new(
                TcpStream::connect(HOST)?
            )
        );
        for pixel in pixelmap {
            stream
            .lock()
            .unwrap()
            .write(
                pixel
                .to_string()
                .as_bytes()
            )?;
        }
        iteration += 1;
    }
}