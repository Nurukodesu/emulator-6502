pub struct ClockPPU {
    cycles: u64, // total PPU cycles
}

impl ClockPPU {
    pub fn new() -> Self {
        Self { cycles: 0 }
    }

    pub fn step_cpu(&mut self, cpu_cycles: u32) {
        self.cycles += cpu_cycles as u64 * 3;
    }

    pub fn cyc(&self) -> u32 {
        (self.cycles % 341) as u32
    }
}
