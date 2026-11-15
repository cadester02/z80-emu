const fn build_cycle_table() -> [u64; 256] {
    let mut table = [0; 256];

    table[0x00] = 4; table[0x01] = 10; table[0x02] = 7; table[0x03] = 6; table[0x04] = 4; table[0x05] = 4;
    
    table[0x80] = 4; table[0x81] = 4; table[0x82] = 4; table[0x83] = 4; table[0x84] = 4; table[0x85] = 4; table[0x86] = 7; table[0x87] = 4;

    table
}

pub const CYCLE_TABLE: [u64; 256] = build_cycle_table();