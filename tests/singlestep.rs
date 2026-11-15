use serde::Deserialize;
use std::fs;
use z80_emu::{Bus, Cpu, Ram};

const DATA_DIR: &str = "tests/data/z80/v1";

// Only the fields your CPU has so far. serde skips everything else in the JSON.
#[derive(Deserialize)]
struct State {
    pc: u16,
    sp: u16,
    a: u8, b: u8, c: u8, d: u8, e: u8, f: u8, h: u8, l: u8,
    ram: Vec<(u16, u8)>, // [[address, value], ...]
}

#[derive(Deserialize)]
struct TestCase {
    name: String,
    initial: State,
    #[serde(rename = "final")]
    expected: State,
    cycles: Vec<serde_json::Value>, // one entry per T-state (verify in the README)
}

fn load_state(cpu: &mut Cpu, ram: &mut Ram, s: &State) {
    cpu.pc = s.pc;
    cpu.sp = s.sp;
    cpu.a = s.a; cpu.f = s.f;
    cpu.b = s.b; cpu.c = s.c;
    cpu.d = s.d; cpu.e = s.e;
    cpu.h = s.h; cpu.l = s.l;
    for &(addr, val) in &s.ram {
        ram.write(addr, val);
    }
}

/// Returns a list of human-readable differences (empty = pass).
fn diff(cpu: &Cpu, ram: &mut Ram, case: &TestCase) -> Vec<String> {
    let want = &case.expected;
    let mut out = Vec::new();

    let mut reg = |name: &str, got: u16, exp: u16| {
        if got != exp {
            out.push(format!("{name}: got {got:#06X}, want {exp:#06X}"));
        }
    };
    reg("pc", cpu.pc, want.pc);
    reg("sp", cpu.sp, want.sp);
    reg("a", cpu.a as u16, want.a as u16);
    reg("b", cpu.b as u16, want.b as u16);
    reg("c", cpu.c as u16, want.c as u16);
    reg("d", cpu.d as u16, want.d as u16);
    reg("e", cpu.e as u16, want.e as u16);
    reg("h", cpu.h as u16, want.h as u16);
    reg("l", cpu.l as u16, want.l as u16);

    if cpu.f != want.f {
        out.push(format!("f: got {:08b}, want {:08b}  (SZYHXPNC)", cpu.f, want.f));
    }
    for &(addr, exp) in &want.ram {
        let got = ram.read(addr);
        if got != exp {
            out.push(format!("ram[{addr:#06X}]: got {got:#04X}, want {exp:#04X}"));
        }
    }
    let t_states = case.cycles.len() as u64;
    if cpu.cycles != t_states {
        out.push(format!("cycles: got {}, want {t_states}", cpu.cycles));
    }
    out
}

fn run_file(file: &str) {
    let path = format!("{DATA_DIR}/{file}.json");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let cases: Vec<TestCase> =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path}: {e}"));

    let mut failures = 0;

    assert!(!cases.is_empty(), "{path} contained no test cases");
    eprintln!("{file}: running {} cases", cases.len());

    for case in &cases {
        let mut cpu = Cpu::default();
        let mut ram = Ram::new();
        load_state(&mut cpu, &mut ram, &case.initial);
        cpu.step(&mut ram);

        let problems = diff(&cpu, &mut ram, case);
        if !problems.is_empty() {
            failures += 1;
            if failures <= 3 {
                // Only print the first few so one bug doesn't flood the screen.
                eprintln!("FAIL {} ({file})\n  {}", case.name, problems.join("\n  "));
            }
        }
    }
    assert_eq!(failures, 0, "{failures} of {} cases failed in {file}.json", cases.len());
}

// One #[test] per opcode, so `cargo test` shows pass/fail per instruction.
macro_rules! opcode_tests {
    ($($name:ident => $file:literal),* $(,)?) => {
        $( #[test] fn $name() { run_file($file); } )*
    };
}

opcode_tests! {
    nop       => "00",
    ld_bc_nn  => "01",
    ld_bc_a   => "02",
    inc_bc    => "03",
    inc_b     => "04",
    dec_b     => "05",
    add_a_b   => "80",
    add_a_c   => "81",
    add_a_d   => "82",
    add_a_e   => "83",
    add_a_h   => "84",
    add_a_l   => "85",
    add_a_hl  => "86",
    add_a_a   => "87",
}