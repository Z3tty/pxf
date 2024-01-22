pub mod ext {
    use crate::pixel::pixel::Pixel;
    use std::io::Result;
    use std::fs::File;
    use std::net::TcpStream;
    use std::io::prelude::*;
    use std::sync::{Arc, Mutex};
    pub const HOST: &str    = "pixelflut.uwu.industries:1234";
    pub const HEIGHT: u16   = 720;
    pub const WIDTH: u16    = 1280;

    pub fn dyn_size_get() -> Result<(u16, u16)> {
        println!("Getting canvas size");
        let mut stream: TcpStream = TcpStream::connect(HOST)?;
        let mut size: String = String::new();
        stream.write(b"SIZE\n")?;
        stream.read_to_string(&mut size)?;
        let mut split = size.split_whitespace();
        split.next();
        let width: u16 = split
                            .next()
                            .unwrap()
                            .parse::<u16>()
                            .unwrap();
        let height: u16 = split
                            .next()
                            .unwrap()
                            .parse::<u16>()
                            .unwrap();
        println!("Canvas size: {}x{}", width, height);
        Ok((width, height))
    }
    
    pub fn serialize_pixelmap_to_file(pixelmap: Vec<Pixel>) -> Result<()> {
        let mut serialized: String = String::new();
        for pixel in pixelmap {
            serialized.push_str(&pixel.to_string());
        }
        let filename: String = format!("img{}.txt", rand::random::<i64>());
        let mut file: File = File::create(filename.clone())?;
        file.write_all(serialized.as_bytes())?;
        println!("Serialized pixelmap to {}", filename);
        Ok(())
    }
    
    pub fn capture_canvas() -> Result<()> {
        println!("Capturing canvas");
        let w: u16;
        let h: u16;
        (w, h) = dyn_size_get()?;
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
        let _ = serialize_pixelmap_to_file(pixelmap);
        Ok(())
    }
    
    pub fn deserialize_to_pixelmap(filename: &str) -> Vec<Pixel> {
        let mut file: File = File::open(filename).unwrap();
        let mut contents: String = String::new();
        file.read_to_string(&mut contents).unwrap();
        let mut pixelmap: Vec<Pixel> = Vec::new();
        for line in contents.lines() {
            pixelmap.push(Pixel::from_str(line));
        }
        pixelmap
    }
    
    pub fn draw_from_file(filename: &str) -> Result<()> {
        println!("Drawing from file {}", filename);
        let pixelmap: Vec<Pixel> = deserialize_to_pixelmap(filename);
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
        Ok(())
    }
}