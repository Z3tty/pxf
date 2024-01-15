use std::net::TcpStream;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::io::Result;
use std::env;
use evalexpr::*;

const HOST: &str = "pixelflut.uwu.industries:1234";
const HEIGHT: u16 = 720;
const WIDTH: u16 = 1280;
const DYN_SIZE: bool = true;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }
    fn from_str(s: &str) -> Color {
        let mut split = s.split_whitespace();
        let r = split.next().unwrap().parse::<u8>().unwrap();
        let g = split.next().unwrap().parse::<u8>().unwrap();
        let b = split.next().unwrap().parse::<u8>().unwrap();
        let a = split.next().unwrap().parse::<u8>().unwrap();
        Color::new(r, g, b, a)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Pixel {
    pub x: u16,
    pub y: u16,
    pub color: Color,
}

impl Pixel {
    fn new(x: u16, y: u16, color: Color) -> Pixel {
        Pixel { x, y, color }
    }
    fn to_string(&self) -> String {
        format!("PX {} {} {:02X}{:02X}{:02X}{:02X}\n", self.x, self.y, self.color.r, self.color.g, self.color.b, self.color.a)
    }
    fn from_str(s: &str) -> Pixel {
        let mut split = s.split_whitespace();
        let x = split.next().unwrap().parse::<u16>().unwrap();
        let y = split.next().unwrap().parse::<u16>().unwrap();
        let r = split.next().unwrap().parse::<u8>().unwrap();
        let g = split.next().unwrap().parse::<u8>().unwrap();
        let b = split.next().unwrap().parse::<u8>().unwrap();
        let a = split.next().unwrap().parse::<u8>().unwrap();
        Pixel::new(x, y, Color::new(r, g, b, a))
    }
}

fn random_color(r_alph: bool) -> Color {
    Color::new(
        rand::random::<u8>(),
        rand::random::<u8>(), 
        rand::random::<u8>(), 
        match r_alph {
            true => rand::random::<u8>(),
            false => 255,
        }
    )
}

fn combine_colors(c1: Color, c2: Color) -> Color {
    Color::new(
        (c1.r + c2.r) / 2,
        (c1.g + c2.g) / 2,
        (c1.b + c2.b) / 2,
        (c1.a + c2.a) / 2,
    )
}

fn dyn_size_get() -> Result<(u16, u16)> {
    println!("Getting canvas size");
    let mut stream: TcpStream = TcpStream::connect(HOST)?;
    let mut size = String::new();
    stream.write(b"SIZE\n")?;
    stream.read_to_string(&mut size)?;
    let mut split = size.split_whitespace();
    split.next();
    let width = split.next().unwrap().parse::<u16>().unwrap();
    let height = split.next().unwrap().parse::<u16>().unwrap();
    println!("Canvas size: {}x{}", width, height);
    Ok((width, height))
}

fn main() {
    let _default_col: Color = Color::from_str("0 0 0 255");
    let args: Vec<String> = env::args().collect();
    let mut generate_noise: bool = false;
    let mut fill_back: bool = false;
    let mut patternize: bool = false;
    let mut pattern_eval: bool = false;
    let mut r_formula: &str = "";
    let mut g_formula: &str = "";
    let mut b_formula: &str = "";
    let mut a_formula: &str = "";
    match args.len() {
        1 => println!("No arguments given, using default values"),
        2 => {
            match args[1].as_str() {
                "-noise" => generate_noise = true,
                "-fill" => fill_back = true,
                "-help" => {
                    println!("Usage: pixelflut-rs [OPTIONS]\n\nOPTIONS:\n\t-noise\t\t\tDraw random pixels\n\t-fill\t\t\tFill background with random color\n\t-pattern\t\tDraw a pattern (formulas currently broken)\n\t\t[\"RF\", \"GF\", \"BF\", \"AF\"]\n\t-help\t\t\tShow this help message");
                    return;
                },
                "-pattern" => patternize = true,
                _ => println!("Invalid argument, using default values"),
            }
        },
        3 => {
            match args[1].as_str() {
                "-noise" => generate_noise = true,
                "-fill" => fill_back = true,
                _ => println!("Invalid arguments, using default values"),
            }
            match args[2].as_str() {
                "-noise" => generate_noise = true,
                "-fill" => fill_back = true,
                _ => println!("Invalid arguments, using default values"),
            }
        },
        6 => {
            match args[1].as_str() {
                "-pattern" => {
                    patternize =  true;
                    pattern_eval = true;
                    r_formula = &args[2];
                    g_formula = &args[3];
                    b_formula = &args[4];
                    a_formula = &args[5];
                },
                _ => println!("Invalid arguments, using default values"),
            }
        }
        _ => println!("Invalid argument count, using default values"),
    }
    let w: u16;
    let h: u16;
    match DYN_SIZE {
        true => (w, h) = dyn_size_get().unwrap(),
        false => (w, h) = (WIDTH, HEIGHT),
    };
    if patternize {
        let mut iteration: u16 = 0;
        loop { println!("Iteration {}", iteration);
            let bgc = random_color(false);
            let mut pixelmap: Vec<Pixel> = Vec::new();
            for x in 0..w {
                for y in 0..h {
                    let pixel: Pixel = match fill_back && generate_noise {
                        true => Pixel::new(x, y, combine_colors(random_color(true), bgc)),
                        false => match fill_back || generate_noise {
                            true => match generate_noise {
                                true => Pixel::new(x, y, random_color(true)),
                                false => Pixel::new(x, y, bgc),
                            },
                            false => match pattern_eval {
                                true => Pixel::new(x, y, Color::new(
                                    (eval_int(r_formula)).unwrap() as u8 % 255,
                                    (eval_int(g_formula)).unwrap() as u8 % 255,
                                    (eval_int(b_formula)).unwrap() as u8 % 255, 
                                    (eval_int(a_formula)).unwrap() as u8 % 255,
                                )),
                                false => Pixel::new(x, y, Color::new(
                                    ((x * x) + (y * y) + 2 * iteration) as u8 % 255,
                                    ((x * x) + (y * y) + 2 * iteration) as u8 % 255,
                                    ((x * x) + (y * y) + 2 * iteration) as u8 % 255, 
                                    ((x * x) + (y * y))                 as u8 % 255,
                                )),
                            },
                        },
                    };
                    pixelmap.push(pixel);
                }
            }
            let stream: Arc<Mutex<TcpStream>> = Arc::new(Mutex::new(TcpStream::connect(HOST).unwrap()));
            for pixel in pixelmap {
                stream.lock().unwrap().write(pixel.to_string().as_bytes()).unwrap();
            }
            iteration += 1;
        }
    } else {    
        let stream: Arc<Mutex<TcpStream>> = Arc::new(Mutex::new(TcpStream::connect(HOST).unwrap()));
        let mut iteration: u32 = 0;
        loop { println!("Iteration {}", iteration);
            let bgc = random_color(false);
            for x in 0..w {
                for y in 0..h {
                    let bgpixel = Pixel::from_str(
                        format!(
                            "{} {} {} {} {} {}",
                            x, y, bgc.r, bgc.g, bgc.b, bgc.a
                        ).as_str());
                    if fill_back        { stream.lock().unwrap().write(bgpixel.to_string().as_bytes()).unwrap(); }
                    if generate_noise   { stream.lock().unwrap().write(Pixel::new(x, y, random_color(true)).to_string().as_bytes()).unwrap(); }
                }
            }     
            iteration += 1;
        }
    }
}
