use crate::bus::Bus;
use crate::cycles::CYCLE_TABLE;

pub const FLAG_SIGN: u8 = 0x80;
pub const FLAG_ZERO: u8 = 0x40;
pub const FLAG_BIT_5: u8 = 0x20;
pub const FLAG_HALF_CARRY: u8 = 0x10;
pub const FLAG_BIT_3: u8 = 0x08;
pub const FLAG_PARITY_OVERFLOW: u8 = 0x04;
pub const FLAG_SUBTRACT: u8 = 0x02;
pub const FLAG_CARRY: u8 = 0x01;

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
        self.cycles += CYCLE_TABLE[instruction as usize];

        // Decode and execute the instruction here
        match instruction {
            // Example instruction decoding and execution
            0x00 => { }

            // LOAD BC, nn
            0x01 => { 
                let value = self.read_word(self.pc, bus);
                self.set_bc(value);
             }

            // LOAD (BC), A
            0x02 => { bus.write(self.bc(), self.a); }

            // INC BC
            0x03 => { self.set_bc(self.bc().wrapping_add(1)); }

            // INC B
            0x04 => { self.b = self.inc_r(self.b); }

            // DEC B
            0x05 => { self.b = self.dec_r(self.b); }

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
            0x86 => { self.add_a_r(bus.read(self.hl())); }

            // ADD A, A
            0x87 => { self.add_a_r(self.a); }

            _ => { unimplemented!("Unknown instruction: {:02X} at {:04X}", instruction, self.pc); }
        }
    }

    // Add a register to the accumulator
    fn add_a_r(&mut self, rhs: u8) {
        let (result, carry) = self.a.overflowing_add(rhs);
        let half_carry: bool = self.check_half_carry(self.a, rhs, result);
        let overflow: bool = self.check_overflow(self.a, rhs, result, false);

        self.a = result;
        self.set_add_flags(result, half_carry, overflow, carry);
    }

    fn inc_r(&mut self, value: u8) -> u8 {
        let result: u8 = value.wrapping_add(1);
        let half_carry: bool = self.check_half_carry(value, 1, result);
        let overflow: bool = self.check_overflow(value, 1, result, false);

        self.set_inc_flags(result, half_carry, overflow);
        result
    }

    fn dec_r(&mut self, value: u8) -> u8 {
        let result: u8 = value.wrapping_sub(1);
        let half_carry: bool = self.check_half_carry(value, 1, result);
        let overflow: bool = self.check_overflow(value, 1, result, true);

        self.set_dec_flags(result, half_carry, overflow);
        result
    }

    fn set_add_flags(&mut self, value: u8, half_carry: bool, parity_overflow: bool, carry: bool) {
        self.set_sign_flag(value);
        self.set_zero_flag(value);
        self.set_bit_5_flag(value);
        self.set_bit_3_flag(value);

        self.set_half_carry_flag(half_carry);
        self.set_overflow_flag(parity_overflow);
        self.set_subtract_flag(false);
        self.set_carry_flag(carry);
    }

    fn set_inc_flags(&mut self, value: u8, half_carry: bool, overflow: bool) {
        self.set_sign_flag(value);
        self.set_zero_flag(value);
        self.set_bit_5_flag(value);
        self.set_bit_3_flag(value);

        self.set_half_carry_flag(half_carry);
        self.set_overflow_flag(overflow);
        self.set_subtract_flag(false);
    }

    fn set_dec_flags(&mut self, value: u8, half_carry: bool, overflow: bool) {
        self.set_sign_flag(value);
        self.set_zero_flag(value);
        self.set_bit_5_flag(value);
        self.set_bit_3_flag(value);

        self.set_half_carry_flag(half_carry);
        self.set_overflow_flag(overflow);
        self.set_subtract_flag(true);
    }

    fn set_sign_flag(&mut self, value: u8) {
        self.f = if value & FLAG_SIGN != 0 {self.f | FLAG_SIGN} else {self.f & !FLAG_SIGN}; // Copies bit 7 of the value to the sign flag
    }

    fn set_zero_flag(&mut self, value: u8) {
        self.f = if value == 0 {self.f | FLAG_ZERO} else {self.f & !FLAG_ZERO}; // Sets the zero flag if the value is zero
    }

    fn set_bit_5_flag(&mut self, value: u8) {
        self.f = if value & FLAG_BIT_5 != 0 {self.f | FLAG_BIT_5} else {self.f & !FLAG_BIT_5}; // Copies bit 5 of the value to the bit 5 flag
    }

    fn set_bit_3_flag(&mut self, value: u8) {
        self.f = if value & FLAG_BIT_3 != 0 {self.f | FLAG_BIT_3} else {self.f & !FLAG_BIT_3}; // Copies bit 3 of the value to the bit 3 flag
    }

    fn set_half_carry_flag(&mut self, half_carry: bool) {
        self.f = if half_carry {self.f | FLAG_HALF_CARRY} else {self.f & !FLAG_HALF_CARRY}; // Sets the half-carry flag if half_carry is true
    }

    fn set_overflow_flag(&mut self, parity_overflow: bool) {
        self.f = if parity_overflow {self.f | FLAG_PARITY_OVERFLOW} else {self.f & !FLAG_PARITY_OVERFLOW}; // Sets the parity/overflow flag if parity_overflow is true
    }

    fn set_subtract_flag(&mut self, subtract: bool) {
        self.f = if subtract {self.f | FLAG_SUBTRACT} else {self.f & !FLAG_SUBTRACT}; // Sets the subtract flag if subtract is true
    }

    fn set_carry_flag(&mut self, carry: bool) {
        self.f = if carry {self.f | FLAG_CARRY} else {self.f & !FLAG_CARRY}; // Sets the carry flag if carry is true
    }

    fn check_half_carry(&self, lhs: u8, rhs: u8, result: u8) -> bool {
        ((lhs ^ rhs ^ result) & 0x10) != 0
    }

    fn check_overflow(&self, lhs: u8, rhs: u8, result: u8, subtract: bool) -> bool {
    let rhs = if subtract { !rhs } else { rhs };
    ((lhs ^ result) & (rhs ^ result) & 0x80) != 0
}

    // Increment the program counter
    fn increment_pc(&mut self) {
        self.pc = self.pc.wrapping_add(1);
    }

    // Read 2 bytes from memory and return them as a u16
    fn read_word(&mut self, address: u16, bus: &mut impl Bus) -> u16 {
        let low = bus.read(address) as u16;
        let high = bus.read(address.wrapping_add(1)) as u16;
        self.pc = address.wrapping_add(2); // Increment the program counter past the word

        (high << 8) | low
    }

    // Register Pairs
    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | self.f as u16
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = value as u8;
    }

    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | self.c as u16
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = value as u8;
    }

    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | self.e as u16
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = value as u8;
    }

    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | self.l as u16
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = value as u8;
    }

    pub fn af_shadow(&self) -> u16 {
        ((self.a_shadow as u16) << 8) | self.f_shadow as u16
    }

    pub fn set_af_shadow(&mut self, value: u16) {
        self.a_shadow = (value >> 8) as u8;
        self.f_shadow = value as u8;
    }

    pub fn bc_shadow(&self) -> u16 {
        ((self.b_shadow as u16) << 8) | self.c_shadow as u16
    }

    pub fn set_bc_shadow(&mut self, value: u16) {
        self.b_shadow = (value >> 8) as u8;
        self.c_shadow = value as u8;
    }

    pub fn de_shadow(&self) -> u16 {
        ((self.d_shadow as u16) << 8) | self.e_shadow as u16
    }

    pub fn set_de_shadow(&mut self, value: u16) {
        self.d_shadow = (value >> 8) as u8;
        self.e_shadow = value as u8;
    }

    pub fn hl_shadow(&self) -> u16 {
        ((self.h_shadow as u16) << 8) | self.l_shadow as u16
    }

    pub fn set_hl_shadow(&mut self, value: u16) {
        self.h_shadow = (value >> 8) as u8;
        self.l_shadow = value as u8;
    }
}