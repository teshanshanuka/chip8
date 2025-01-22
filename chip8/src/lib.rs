pub const SCREEN_W: usize = 64;
pub const SCREEN_H: usize = 32;

const RAM_SZ: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SZ: usize = 16;
const NUM_KEYS: usize = 16;

pub struct Emu {
    pc: u16, // program counter
    ram: [u8; RAM_SZ],
    screen: [bool; SCREEN_W * SCREEN_H],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16, // stack pointer
    stack: [u16; STACK_SZ],
    keys: [bool; NUM_KEYS],
    dt: u8, // delay timer
    st: u8, // sound timer
}
