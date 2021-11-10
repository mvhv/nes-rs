use nes_rs::{CPU, prog};

fn main() {
    let mut cpu = CPU::new();
    cpu.load_program(prog::SNAKE);
    cpu.interrupt_reset();
    cpu.run();
}