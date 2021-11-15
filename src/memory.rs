use std::fmt;

pub trait MemoryMap {
    fn read_u8(&self, addr: u16) -> u8;
    
    fn write_u8(&mut self, addr: u16, val: u8);
    
    fn load(&mut self, addr: u16, data: &[u8]);

    fn read_u16(&self, addr: u16) -> u16 {
        let lo = self.read_u8(addr);
        let hi = self.read_u8(addr + 1);
        u16::from_le_bytes([lo, hi])
    }

    fn write_u16(&mut self, addr: u16, val: u16) {
        let [lo, hi] = val.to_le_bytes();
        self.write_u8(addr, lo);
        self.write_u8(addr + 1, hi);
    }
}

/// A MemoryMap with only a flat address space and no shared regions
pub struct SimpleMap<const S: usize>([u8; S]);

impl<const S: usize> Default for SimpleMap<S> {
    fn default() -> SimpleMap<S> {
        SimpleMap([0u8; S])
    }
}

impl<const S: usize> MemoryMap for SimpleMap<S> {
    fn read_u8(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        self.0[addr]
    }

    fn write_u8(&mut self, addr: u16, val: u8) {
        let addr = addr as usize;
        self.0[addr] = val;
    }

    fn load(&mut self, addr: u16, data: &[u8]) {
        let addr = addr as usize;
        let end = addr + data.len();
        self.0[addr..end].copy_from_slice(data);
    }
}

/// Formats up to 16 bytes into a readable hexdump line
fn fmt_hexdump_line(line_no: Option<u16>, data: &[u8]) -> String {
    let hex_body = data
        .iter()
        .enumerate()
        .map(|(i, &x)| match i {
            0x3 | 0x7 | 0x0b => format!("{:02X} ", x),
            _ => format!("{:02X}", x),
        })
        .collect::<Vec<String>>()
        .join(" ");

    let ascii_body = data
        .iter()
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

impl<const S: usize> fmt::Debug for SimpleMap<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let header = fmt_hexdump_line(None, &(0x00..0x10).collect::<Vec<_>>());

        let divider = "-".repeat(header.len());

        let body = self
            .0
            .chunks(16)
            .enumerate()
            .map(|(i, chunk)| fmt_hexdump_line(Some(i as u16 * 16), chunk))
            .collect::<Vec<String>>()
            .join("\n");

        write!(f, "\n{}\n{}\n{}", header, divider, body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deadbeef is a recognisable 32-bit value for testing
    const DEADBEEF: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];

    #[test]
    fn test_memory_write_read() {
        let mut mem = SimpleMap::<4096>::default();
        mem.write_u8(10, 5u8);
        assert_eq!(mem.read_u8(10), 5u8);
        assert_eq!(mem.read_u8(100), 0u8);
    }

    #[test]
    fn test_memory_load() {
        let mut mem = SimpleMap::<0x100>::default();
        let addr = 0x08;
        let end = addr + DEADBEEF.len();
        mem.load(addr as u16, &DEADBEEF);
        assert_eq!(&mem.0[addr..end], &DEADBEEF);
    }

    #[test]
    fn test_u16_le_write_read() {
        let mut mem = SimpleMap::<0x100>::default();
        mem.load(0x00, &DEADBEEF);

        // write 4096 at 0x10 in little endian
        let addr = 0x10;
        let val = 4096_u16;
        let [lo, hi] = val.to_le_bytes();
        mem.write_u16(addr, val);

        // check simple read
        assert_eq!(mem.read_u16(addr), val);
        // check byte order
        assert_eq!(mem.read_u8(addr), lo);
        assert_eq!(mem.read_u8(addr + 1), hi);

        // read deadbeef as 16-bit words (should be flipped)
        assert_eq!(mem.read_u16(0x00), 0xADDE);
        assert_eq!(mem.read_u16(0x02), 0xEFBE);
    }
}
