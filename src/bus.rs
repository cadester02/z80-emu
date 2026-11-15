pub const RAM_SIZE: usize = 0x10000;

pub trait Bus {
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, val: u8);
}

#[derive(Default)]
pub struct Ram(pub Vec<u8>);

impl Ram {
    pub fn new() -> Self {
        Ram(vec![0; RAM_SIZE])
    }

    pub fn load(&mut self, address: u16, data: &[u8]) {
        self.0[address as usize..(address as usize) + data.len()].copy_from_slice(data);
    }
}

impl Bus for Ram {
    fn read(&mut self, addr: u16) -> u8 {
        self.0[addr as usize]
    }

    fn write(&mut self, addr: u16, val: u8) {
        self.0[addr as usize] = val;
    }
}