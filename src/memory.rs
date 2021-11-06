use std::fmt;

pub struct Memory<const S: usize> ([u8; S]);

impl<const S: usize> Default for Memory<S> {
    fn default() -> Memory<S> {
        Memory([0u8; S])
    }
}

impl<const S: usize> Memory<S> {
    pub fn get(&self, n: usize) -> u8 {
        self.0[n]
    }
    
    pub fn set(&mut self, n: usize, val: u8) {
        self.0[n] = val;
    }

    pub fn load_at(&mut self, addr: usize, data: &[u8]) {
        let end = addr + data.len();
        self.0[addr..end].copy_from_slice(data);
    }
}

/// Formats up to 16 bytes into a readable hexdump line
fn fmt_hexdump_line(line_no: Option<u16>, data:  &[u8]) -> String {
    let hex_body = data.iter()
        .enumerate()
        .map(|(i, &x)| {
            match i {
                0x3 | 0x7 | 0x0b => format!("{:02X} ", x), 
                _ => format!("{:02X}", x),
            }
        })
        .collect::<Vec<String>>()
        .join(" ");
    
    let ascii_body = data.iter()
        .map(|&x| {
            let c = x as char;
            if c.is_ascii() && !c.is_control() {
                c
            } else {
                '.'
            }
        })
        .collect::<String>();

    match line_no {
        Some(no) => format!("{:04X} | {} | {:16} |", no, hex_body, ascii_body),
        None => format!("     | {} | {} |", hex_body, ascii_body),
    }
}

impl<const S: usize> fmt::Debug for Memory<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let header = fmt_hexdump_line(None, &(0x00..0x10).collect::<Vec<_>>());
        
        let divider = "-".repeat(header.len());

        let body = self.0
            .chunks(16)
            .enumerate()
            .map(|(i, chunk)| fmt_hexdump_line(Some(i as u16 * 16), chunk))
            .collect::<Vec<String>>()
            .join("\n");
        
        write!(f, "{}\n{}\n{}", header, divider, body)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory() {
        let mut mem = Memory::<4096>::default();
        mem.set(10, 5u8);
        assert_eq!(mem.get(10), 5u8);
        assert_eq!(mem.get(100), 0u8);
    }

    #[test]
    fn test_load_at() {
        let mut mem = Memory::<0x100>::default();
        let some_bytes = [0xDE, 0xAD, 0xBE, 0xEF];
        let addr = 0x08;
        let end = addr + some_bytes.len();
        mem.load_at(addr, &some_bytes);
        assert_eq!(&mem.0[addr..end], &some_bytes);
    }
}