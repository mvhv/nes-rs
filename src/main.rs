mod cpu;
mod memory;

use memory::Memory;

fn main() {
    let mut mem = Memory::<0x100>::default();
    let some_bytes = [0xDE, 0xAD, 0xBE, 0xEF];
    let addr = 0x08;
    let end = addr + some_bytes.len();
    mem.load_at(addr, &some_bytes);
    println!("{:?}", mem);
}