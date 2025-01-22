pub const SCREEN_W: usize = 64;
pub const SCREEN_H: usize = 32;

const RAM_SZ: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SZ: usize = 16;
const NUM_KEYS: usize = 16;

const START_ADDR: u16 = 0x200; // ROM data is loaded after a 512 byte offset into the RAM

const FONTSET_SZ: usize = 80;
const FONTSET: [u8; FONTSET_SZ] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

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

impl Emu {
    pub fn new() -> Self {
        let mut emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SZ],
            screen: [false; SCREEN_W * SCREEN_H],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SZ],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };
        emu.ram[0..FONTSET_SZ].copy_from_slice(&FONTSET);

        emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SZ];
        self.screen = [false; SCREEN_W * SCREEN_H];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SZ];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SZ].copy_from_slice(&FONTSET);
    }

    pub fn tick(&mut self) {
        let op = self.fetch();
        self.execute(op)
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            if self.st == 1 {
                self.beep();
            }
            self.st -= 1;
        }
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            // 0000: NOP
            (0, 0, 0, 0) => return,
            // 00E0: CLS
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_W * SCREEN_H];
            }
            // 00EE: RET
            (0, 0, 0xE, 0xE) => {
                self.pc = self.pop();
            }
            // 1NNN: JMP NNN
            (1, _, _, _) => {
                self.pc = op & 0x0FFF;
            }
            // 2NNN: CALL NNN
            (2, _, _, _) => {
                self.push(self.pc);
                self.pc = op & 0x0FFF;
            }
            // 3XNN: SKIP VX == NN
            (3, x, _, _) => {
                let nn = (op & 0x00FF) as u8;
                if self.v_reg[x as usize] == nn {
                    self.pc += 2;
                }
            }
            // 4XNN: SKIP VX != NN
            (4, x, _, _) => {
                let nn = (op & 0x00FF) as u8;
                if self.v_reg[x as usize] != nn {
                    self.pc += 2;
                }
            }
            // 5XY0: SKIP VX == VY
            (5, x, y, 0) => {
                if self.v_reg[x as usize] == self.v_reg[y as usize] {
                    self.pc += 2;
                }
            }
            // 6XNN: VX = NN
            (6, x, _, _) => {
                let nn = (op & 0x00FF) as u8;
                self.v_reg[x as usize] = nn;
            }
            // 7XNN: VX += NN
            (7, x, _, _) => {
                let nn = (op & 0x00FF) as u8;
                self.v_reg[x as usize] = self.v_reg[x as usize].wrapping_add(nn);
            }
            // 8XY0: VX = VY
            (8, x, y, 0) => {
                self.v_reg[x as usize] = self.v_reg[y as usize];
            }
            // 8XY1: VX |= VY
            (8, x, y, 1) => {
                self.v_reg[x as usize] |= self.v_reg[y as usize];
            }
            // 8XY2: VX &= VY
            (8, x, y, 2) => {
                self.v_reg[x as usize] &= self.v_reg[y as usize];
            }
            // 8XY3: VX ^= VY
            (8, x, y, 3) => {
                self.v_reg[x as usize] ^= self.v_reg[y as usize];
            }
            // 8XY4: VX += VY
            (8, x, y, 4) => {
                let (vx, carry) = self.v_reg[x as usize].overflowing_add(self.v_reg[y as usize]);
                let vf = if carry { 1 } else { 0 };
                self.v_reg[x as usize] = vx;
                self.v_reg[0xF] = vf;
            }
            // 8XY5: VX -= VY
            (8, x, y, 5) => {
                let (vx, borrow) = self.v_reg[x as usize].overflowing_sub(self.v_reg[y as usize]);
                let vf = if borrow { 0 } else { 1 };
                self.v_reg[x as usize] = vx;
                self.v_reg[0xF] = vf;
            }
            // 8XY6: VX >>= 1
            (8, x, _, 6) => {
                let lsb = self.v_reg[x as usize] & 1;
                self.v_reg[x as usize] >>= 1;
                self.v_reg[0xF] = lsb;
            }
            // 8XY7: VX = VY - VX
            (8, x, y, 7) => {
                let (vx, borrow) = self.v_reg[y as usize].overflowing_sub(self.v_reg[x as usize]);
                let vf = if borrow { 0 } else { 1 };
                self.v_reg[x as usize] = vx;
                self.v_reg[0xF] = vf;
            }
            // 8XYE: VX << 1
            (8, x, _, 0xE) => {
                let msb = (self.v_reg[x as usize] >> 7) & 1;
                self.v_reg[x as usize] <<= 1;
                self.v_reg[0xF] = msb;
            }
            // 9XY0: SKIP VX != VY
            (9, x, y, 0) => {
                if self.v_reg[x as usize] != self.v_reg[y as usize] {
                    self.pc += 2;
                }
            }
            // ANNN: I = NNN
            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            }
            // BNNN: JMP V0 + NNN
            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            }
            // CXNN: VX = rand() & NN
            (0xC, x, _, _) => {
                let nn = (op & 0x00FF) as u8;
                let rng: u8 = rand::random();
                self.v_reg[x as usize] = rng & nn;
            }
            // DXYN: Draw sprite
            (0xD, x, y, num_rows) => {
                let x_coord = self.v_reg[x as usize] as u16;
                let y_coord = self.v_reg[y as usize] as u16;

                // keep track of flipped pixels
                let mut flipped = false;

                // sprite can be n_rows tall
                for y_line in 0..num_rows {
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    // sprites are always 8 pixels wide
                    for x_line in 0..8 {
                        // Use a mask to fetch current pixel's bit. Only flip if a 1
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            // wrap around screen
                            let x = (x_coord + x_line) as usize % SCREEN_W;
                            let y = (y_coord + y_line) as usize % SCREEN_H;

                            //pixel idx for 1D array
                            let idx = x + (y * SCREEN_W);

                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

                // set VF to 1 if any pixels were flipped from 1 to 0
                self.v_reg[0xF] = if flipped { 1 } else { 0 };
            }
            // EX9E: SKIP if key pressed
            (0xE, x, 9, 0xE) => {
                let vx = self.v_reg[x as usize];
                if self.keys[vx as usize] {
                    self.pc += 2;
                }
            }
            // EXA1: SKIP if key not pressed
            (0xE, x, 0xA, 1) => {
                let vx = self.v_reg[x as usize];
                if !self.keys[vx as usize] {
                    self.pc += 2;
                }
            }
            // FX07: VX = DT
            (0xF, x, 0, 0x7) => {
                self.v_reg[x as usize] = self.dt;
            }
            // FX0A: Wait for key press
            (0xF, x, 0, 0xA) => {
                let mut key_pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x as usize] = i as u8;
                        key_pressed = true;
                        break;
                    }
                }
                if !key_pressed {
                    // redo opcode
                    self.pc -= 2;
                }
            }
            // FX15: DT = VX
            (0xF, x, 1, 0x5) => {
                self.dt = self.v_reg[x as usize];
            }
            // FX18: ST = VX
            (0xF, x, 1, 0x8) => {
                self.st = self.v_reg[x as usize];
            }
            // FX1E: I += VX
            (0xF, x, 1, 0xE) => {
                let vx = self.v_reg[x as usize] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            }
            // FX29: I to font addr
            (0xF, x, 2, 9) => {
                let c = self.v_reg[x as usize] as u16;
                // each font sprite is 5 bytes
                self.i_reg = c * 5;
            }
            // FX33: I = BCD of VX
            (0xF, x, 3, 3) => {
                let vx = self.v_reg[x as usize] as f32;

                let hundreds = (vx / 100.0).floor() as u8;
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                let ones = (vx % 10.0) as u8;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            }
            // FX55: store V0 - VX in I
            (0xF, x, 5, 5) => {
                for idx in 0..=x as usize {
                    self.ram[self.i_reg as usize + idx] = self.v_reg[idx];
                }
            }
            // FX65: load I to V0 - VX
            (0xF, x, 6, 5) => {
                for idx in 0..=x as usize {
                    self.v_reg[idx] = self.ram[self.i_reg as usize + idx];
                }
            }
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
        }
    }

    fn fetch(&mut self) -> u16 {
        // chip8 opcodes are 2 bytes
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op: u16 = (higher_byte << 8) | lower_byte; // combine as big-endian
        self.pc += 2;
        op
    }

    fn beep(&mut self) {}

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }
}
