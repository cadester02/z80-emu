use crate::bus::Bus;

pub const FLAG_SIGN: u8 = 0x80;
pub const FLAG_ZERO: u8 = 0x40;
pub const FLAG_BIT_5: u8 = 0x20;
pub const FLAG_HALF_CARRY: u8 = 0x10;
pub const FLAG_BIT_3: u8 = 0x08;
pub const FLAG_PARITY_OVERFLOW: u8 = 0x04;
pub const FLAG_SUBTRACT: u8 = 0x02;
pub const FLAG_CARRY: u8 = 0x01;

pub const NOP_CYCLES: u64 = 4;
pub const ADD_CYCLES: u64 = 4;
pub const READ_CYCLES: u64 = 3;

#[derive(Default)]
pub struct Cpu {
    // 8-bit registers
    pub a: u8, pub f: u8,
    pub b: u8, pub c: u8,
    pub d: u8, pub e: u8,
    pub h: u8, pub l: u8,
    
    // Shadow registers 8-bit
    pub a_shadow: u8, pub f_shadow: u8,
    pub b_shadow: u8, pub c_shadow: u8,
    pub d_shadow: u8, pub e_shadow: u8,
    pub h_shadow: u8, pub l_shadow: u8,

    // 16-bit registers
    pub sp: u16,
    pub ix: u16, pub iy: u16,


    // Interrupt and refresh registers
    pub i: u8, pub r: u8,

    // Program counter
    pub pc: u16,

    // CPU state
    pub halted: bool,
    pub cycles: u64,
}

impl Cpu {
    // Fetch the next instruction from memory
    pub fn fetch(&mut self, bus: &mut impl Bus) -> u8 {
        let instruction = bus.read(self.pc);
        self.increment_pc();
        instruction
    }

    // Execute a single instruction
    pub fn step(&mut self, bus: &mut impl Bus) {
        // Fetch the next instruction
        let instruction = self.fetch(bus);

        // Decode and execute the instruction here
        match instruction {
            // Example instruction decoding and execution
            0x00 => {
                // NOP instruction
                self.cycles += NOP_CYCLES;
            }

            // ADD A, B
            0x80 => { self.add_a_r(self.b); }

            // ADD A, C
            0x81 => { self.add_a_r(self.c); }

            // ADD A, D
            0x82 => { self.add_a_r(self.d); }

            // ADD A, E
            0x83 => { self.add_a_r(self.e); }

            // ADD A, H
            0x84 => { self.add_a_r(self.h); }

            // ADD A, L
            0x85 => { self.add_a_r(self.l); }

            // ADD A, (HL)
            0x86 => {
                let value = bus.read(self.hl());
                self.add_a_r(value);

                self.cycles += READ_CYCLES;
            }

            // ADD A, A
            0x87 => { self.add_a_r(self.a); }

            _ => {
                unimplemented!("Unknown instruction: {:02X} at {:04X}", instruction, self.pc);
            }
        }
    }

    // Add a register to the accumulator
    fn add_a_r(&mut self, rhs: u8) {
        let half_carry = self.check_half_carry(self.a, rhs);
        let parity_overflow = self.check_parity_overflow(self.a, rhs);
        let (result, carry) = self.a.overflowing_add(rhs);

        self.a = result;
        self.set_result_flags(result, half_carry, parity_overflow, false, carry);

        self.cycles += ADD_CYCLES;
    }

    fn set_result_flags(&mut self, value: u8, half_carry: bool, parity_overflow: bool, subtract: bool, carry: bool) {
        self.f = if value & FLAG_SIGN != 0 {self.f | FLAG_SIGN} else {self.f & !FLAG_SIGN}; // Copies bit 7 of the value to the sign flag
        self.f = if value == 0 {self.f | FLAG_ZERO} else {self.f & !FLAG_ZERO}; // Sets the zero flag if the value is zero
        self.f = if value & FLAG_BIT_5 != 0 {self.f | FLAG_BIT_5} else {self.f & !FLAG_BIT_5}; // Copies bit 5 of the value to the bit 5 flag
        self.f = if value & FLAG_BIT_3 != 0 {self.f | FLAG_BIT_3} else {self.f & !FLAG_BIT_3}; // Copies bit 3 of the value to the bit 3 flag

        self.f = if half_carry {self.f | FLAG_HALF_CARRY} else {self.f & !FLAG_HALF_CARRY}; // Sets the half-carry flag if half_carry is true
        self.f = if parity_overflow {self.f | FLAG_PARITY_OVERFLOW} else {self.f & !FLAG_PARITY_OVERFLOW}; // Sets the parity/overflow flag if parity_overflow is true
        self.f = if subtract {self.f | FLAG_SUBTRACT} else {self.f & !FLAG_SUBTRACT}; // Sets the subtract flag if subtract is true
        self.f = if carry {self.f | FLAG_CARRY} else {self.f & !FLAG_CARRY}; // Sets the carry flag if carry is true
    } 

    fn check_half_carry(&self, lhs: u8, rhs: u8) -> bool {
        ((lhs & 0x0F) + (rhs & 0x0F)) > 0x0F
    }

    fn check_parity_overflow(&self, lhs: u8, rhs: u8) -> bool {
        let result = lhs.wrapping_add(rhs);
        ((lhs ^ rhs) & 0x80) == 0 && ((lhs ^ result) & 0x80) != 0
    }

    // Get the value of the HL register pair
    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | self.l as u16
    }

    // Increment the program counter
    fn increment_pc(&mut self) {
        self.pc = self.pc.wrapping_add(1);
    }
}

