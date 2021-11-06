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
        (&mut self.0[addr..end]).copy_from_slice(data);
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
        let mut mem = Memory::<0xFF>::default();
        let some_bytes = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let addr = 10;
        let end = addr + some_bytes.len();
        mem.load_at(addr, &some_bytes);
        assert_eq!(&mem.0[addr..end], &some_bytes);
    }
}