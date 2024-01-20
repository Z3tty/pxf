use core::str;
use std::net::TcpStream;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::io::Result;
use std::env;
use evalexpr::*;
use std::fs::File;

const HOST: &str    = "pixelflut.uwu.industries:1234";
const HEIGHT: u16   = 720;
const WIDTH: u16    = 1280;

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
        let r: u8 = u8::from_str_radix(&s[0..2], 16).unwrap();
        let g: u8 = u8::from_str_radix(&s[2..4], 16).unwrap();
        let b: u8 = u8::from_str_radix(&s[4..6], 16).unwrap();
        let a: u8 = match s.len() {
            8 => u8::from_str_radix(&s[6..8], 16).unwrap(),
            _ => 255,
        };
        Color::new(r, g, b, a)
    }
    fn to_str(&self) -> String {
        format!("{}{}{}{}", self.r, self.g, self.b, self.a)
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
        split.next(); // Skip PX
        let x: u16 = split.next().unwrap().parse::<u16>().unwrap();
        let y: u16 = split.next().unwrap().parse::<u16>().unwrap();
        let color_str: &str = split.next().unwrap();
        let color: Color = Color::from_str(color_str);
        Pixel::new(x, y, color)
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
    let mut size: String = String::new();
    stream.write(b"SIZE\n")?;
    stream.read_to_string(&mut size)?;
    let mut split = size.split_whitespace();
    split.next();
    let width: u16 = split.next().unwrap().parse::<u16>().unwrap();
    let height: u16 = split.next().unwrap().parse::<u16>().unwrap();
    println!("Canvas size: {}x{}", width, height);
    Ok((width, height))
}

fn serialize_pixelmap_to_file(pixelmap: Vec<Pixel>) -> () {
    let mut serialized: String = String::new();
    for pixel in pixelmap {
        serialized.push_str(&pixel.to_string());
    }
    let filename = format!("img{}.txt", rand::random::<i64>());
    let mut file = File::create(filename.clone()).unwrap();
    file.write_all(serialized.as_bytes()).unwrap();
    println!("Serialized pixelmap to {}", filename);
}

fn capture_canvas() -> Result<()> {
    println!("Capturing canvas");
    let w: u16;
    let h: u16;
    (w, h) = dyn_size_get().unwrap();
    let mut pixelmap: Vec<Pixel> = Vec::new();
    let mut stream: TcpStream = TcpStream::connect(HOST)?;
    for x in 0..w {
        for y in 0..h {
            stream.write(format!("PX {} {}\n", x, y).as_bytes())?;
            let mut pixel: String = String::new();
            stream.read_to_string(&mut pixel)?;
            pixelmap.push(Pixel::from_str(&pixel));
        }
    }
    serialize_pixelmap_to_file(pixelmap);
    Ok(())
}

fn deserialize_to_pixelmap(filename: &str) -> Vec<Pixel> {
    let mut file: File = File::open(filename).unwrap();
    let mut contents: String = String::new();
    file.read_to_string(&mut contents).unwrap();
    let mut pixelmap: Vec<Pixel> = Vec::new();
    for line in contents.lines() {
        pixelmap.push(Pixel::from_str(line));
    }
    pixelmap
}

fn draw_from_file(filename: &str) -> Result<()> {
    println!("Drawing from file {}", filename);
    let pixelmap: Vec<Pixel> = deserialize_to_pixelmap(filename);
    let stream: Arc<Mutex<TcpStream>> = Arc::new(Mutex::new(TcpStream::connect(HOST)?));
    for pixel in pixelmap {
        stream.lock().unwrap().write(pixel.to_string().as_bytes()).unwrap();
    }
    Ok(())
}

fn print_help() -> () {
    println!("Usage: pxf [OPTIONS]\n\nOPTIONS:\n\t-noise\t\t\tDraw random pixels\
    \n\t-fill\t\t\tFill background with random color\n\t-pattern\t\tDraw a pattern (formulas \
         currently broken)\n\t\t [\"RF\", \"GF\", \"BF\", \"AF\"]\n\t-help\t\t\tShow this help message\n\
         \t-dyn\t\t\tGet canvas size from server instead of using hardcoded aspect\n\
         \t-capture\t\tCapture canvas and save to file [UNFINISHED]\n\
         \t-from [FILENAME]\tDraw from file [UNFINISHED]\n\
         \t-slice\t\t\tDraw slices across the canvas\n");
}

fn main() -> () {
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
                "-help" => { print_help(); return; },
                "-noise" => generate_noise  = true,
                "-fill" => fill_back        = true,
                "-dyn" => dyn_size          = true,
                "-pattern" => patternize    = true,
                "-slice" => slice           = true,
                "-capture" => capture       = true,
                _ => println!("Invalid argument, using default values"),
            }
        },
        3 => {
            match args[1].as_str() {
                "-noise" => generate_noise  = true,
                "-fill" => fill_back        = true,
                "-dyn" => dyn_size          = true,
                "-slice" => slice           = true,
                "-from" => file             = &args[2],
                _ => println!("Invalid arguments, using default values"),
            }
            match args[2].as_str() {
                "-noise" => generate_noise  = true,
                "-fill" => fill_back        = true,
                "-dyn" => dyn_size          = true,
                "-slice" => slice           = true,
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
        true => (w, h) = dyn_size_get().unwrap(),
        false => (w, h) = (WIDTH, HEIGHT),
    };

    if capture {
        capture_canvas().unwrap();
        return;
    }

    if file != "" {
        draw_from_file(file).unwrap();
        return;
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
        let stream: Arc<Mutex<TcpStream>> = Arc::new(Mutex::new(TcpStream::connect(HOST).unwrap()));
        for pixel in pixelmap {
            stream.lock().unwrap().write(pixel.to_string().as_bytes()).unwrap();
        }
        return;
    }

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
                        false => match patternize {
                            true => match pattern_eval {
                                true => Pixel::new(x, y, Color::new(
                                    (eval_int(r_formula)).unwrap() as u8 % 255,
                                    (eval_int(g_formula)).unwrap() as u8 % 255,
                                    (eval_int(b_formula)).unwrap() as u8 % 255, 
                                    (eval_int(a_formula)).unwrap() as u8 % 255,
                                )),
                                false => Pixel::new(x, y, Color::new(
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
        let stream: Arc<Mutex<TcpStream>> = Arc::new(Mutex::new(TcpStream::connect(HOST).unwrap()));
        for pixel in pixelmap {
            stream.lock().unwrap().write(pixel.to_string().as_bytes()).unwrap();
        }
        iteration += 1;
    }
}
