use std::fmt::Display;


#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub leds: [u8; 25],
}

impl Frame {

    pub fn new() -> Frame {
        Frame { leds: [255; 25] }
    }

    pub fn with_values(leds: [u8; 25]) -> Frame {
        Frame { leds }
    }



    pub fn invert(&mut self) {
        for l in &mut self.leds {
            *l = 255 - *l;
        }
    }

    pub fn set_row(&mut self, row: usize, val: u8) {
        for (i, l) in self.leds.iter_mut().enumerate() {
            if i % 5 == row {
                *l = val;
            }
        }
    }

    pub fn set_col(&mut self, col: usize, val: u8) {
        for (i, l) in self.leds.iter_mut().enumerate() {
            if i / 5 == col {
                *l = val;
            }
        }
    }

    pub fn set_all(&mut self, val: u8) {
        self.leds = [val; 25];
    }

}

impl Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let mut out = String::new();
        out.push_str(".byte ");
        for (i, b) in self.leds.iter().enumerate() {
            if i != 0 && 1 != 24 {
                out.push(',');
            }
            out.push_str(&format!("{}", b));
        }

        write!(f, "{}", out)
    }
}