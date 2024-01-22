pub mod pixel {
    #[derive(Debug, Clone, Copy)]
    pub struct Color {
        pub r: u8,
        pub g: u8,
        pub b: u8,
        pub a: u8,
    }

    impl Color {
        pub fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
            Color { r, g, b, a }
        }
        pub fn from_str(s: &str) -> Color {
            let r: u8 = u8::from_str_radix(&s[0..2], 16).unwrap();
            let g: u8 = u8::from_str_radix(&s[2..4], 16).unwrap();
            let b: u8 = u8::from_str_radix(&s[4..6], 16).unwrap();
            let a: u8 = match s.len() {
                8 => u8::from_str_radix(&s[6..8], 16).unwrap(),
                _ => 255,
            };
            Color::new(r, g, b, a)
        }
        pub fn to_str(&self) -> String {
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
        pub fn new(x: u16, y: u16, color: Color) -> Pixel {
            Pixel { x, y, color }
        }
        pub fn to_string(&self) -> String {
            format!(
                "PX {} {} {:02X}{:02X}{:02X}{:02X}\n",
                self.x, self.y, 
                self.color.r, self.color.g, self.color.b, self.color.a
            )
        }
        pub fn from_str(s: &str) -> Pixel {
            let mut split = s.split_whitespace();
            split.next(); // Skip PX
            let x: u16 = split
                            .next()
                            .unwrap()
                            .parse::<u16>()
                            .unwrap();
            let y: u16 = split
                            .next()
                            .unwrap()
                            .parse::<u16>()
                            .unwrap();
            let color_str: &str = split
                                    .next()
                                    .unwrap();
            let color: Color = Color::from_str(color_str);
            Pixel::new(x, y, color)
        }
    }

    pub fn random_color(r_alph: bool) -> Color {
        Color::new(
            rand::random::<u8>(),
            rand::random::<u8>(), 
            rand::random::<u8>(), 
            match r_alph {
                true    => rand::random::<u8>(),
                false   => 255,
            }
        )
    }

    pub fn combine_colors(c1: Color, c2: Color) -> Color {
        Color::new(
            (c1.r + c2.r) / 2,
            (c1.g + c2.g) / 2,
            (c1.b + c2.b) / 2,
            (c1.a + c2.a) / 2,
        )
    }
}