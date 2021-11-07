mod cpu;
mod memory;

use memory::MemoryMap;

fn main() {
    // a simple test case to print a hexdump and suppress warnings
    let mut mem = MemoryMap::<0x100>::default();
    let some_bytes = [0xDE, 0xAD, 0xBE, 0xEF];
    let addr = 0x08;
    mem.load(addr, &some_bytes);
    println!("{:?}", mem);
    cpu::CPU::new().run();
}