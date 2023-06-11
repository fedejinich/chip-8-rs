use std::format;

use crate::opcodes::{fetch_op, match_opcode, OpcodeExec};

const PROGRAM_START_ADDRESS: u16 = 0x200;

pub const MEM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGTH: usize = 32;

pub struct Chip8 {
    v_reg: [u8; NUM_REGS],
    i: u16,
    pc: u16,
    memory: [u8; MEM_SIZE],
    stack: [u16; STACK_SIZE],
    stack_pointer: u8,
    display: [bool; SCREEN_WIDTH * SCREEN_HEIGTH],
    delay_timer: u8,
    sound_timer: u8,
}

impl Default for Chip8 {
    fn default() -> Self {
        Chip8 {
            v_reg: [0; NUM_REGS],
            i: 0,
            pc: PROGRAM_START_ADDRESS,
            memory: [0; MEM_SIZE],
            stack: [0; STACK_SIZE],
            stack_pointer: 0,
            display: [false; SCREEN_WIDTH * SCREEN_HEIGTH],
            delay_timer: 0,
            sound_timer: 0,
        }
    }
}

impl Chip8 {
    #[allow(dead_code)]
    pub fn load_program(&mut self, program: &[u8]) {
        let start = PROGRAM_START_ADDRESS as usize;
        let end = start + program.len();
        self.memory[start..end].copy_from_slice(program);
        println!("program loaded");
    }

    #[allow(dead_code)]
    // todo(fedejinich) add unit test for this
    fn tick(&mut self) {
        let op = fetch_op(&self.memory, &(self.pc as usize));
        self.execute_op(op);
        println!("tick")
    }

    // todo(fedejinich) add unit test for this
    // decodes the given opcode and executes it
    fn execute_op(&mut self, op: u16) {
        let result = match_opcode(op).execute_op(self);

        if (&result).is_err() {
            println!("Opcode execution error: {}", result.unwrap_err());
            return;
        }

        // advance program counter (memory is [u8], opcode is u16, that's why we advance 'pc' by two)
        self.pc_inc();
        self.pc_inc();

        println!("Opcode executed: {}", result.unwrap());
    }

    fn push(&mut self, elem: u16) {
        // todo(fedejinich) no error handling, what happens when we reach the limit?
        self.stack[self.stack_pointer as usize] = elem;
        self.stack_pointer += 1;
    }

    fn pop(&mut self) -> u16 {
        // todo(fedejinich) no error handling, what happens when there's nothing left?
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize]
    } 

    fn pc(&mut self, pc: &u16) {
        // todo(fedejinich) no error handling, should restrict pc to fit in memory range?
        self.pc = pc.clone();
    }

    fn pc_inc(&mut self) {
        self.pc += 1;
    }

    // OPCODES

    // Clear the display
    pub fn opcode_cls(&mut self) -> OpcodeExec {
        self.display = [false; SCREEN_WIDTH * SCREEN_HEIGTH];
        Ok(String::from("CLS"))
    }

    // Return from a subroutine.
    // The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
    pub fn opcode_ret(&mut self) -> OpcodeExec {
        let new_pc = self.pop();
        self.pc(&new_pc);
        Ok(String::from("RET"))
    }

    // Jump to a machine code routine at nnn.
    // This instruction is only used on the old computers on which Chip-8 was originally implemented. It is ignored by modern interpreters.
    pub fn opcode_sys(&self, _nnn: u16) -> OpcodeExec {
        Ok(String::from("SYS"))
    }

    // Jump to location nnn.
    // The interpreter sets the program counter to nnn.
    pub fn opcode_jmp(&mut self, nnn: u16) -> OpcodeExec {
        self.pc(&nnn);
        Ok(format!("JMP {}", nnn))
    }

    // Call subroutine at nnn.
    // The interpreter increments the stack pointer, then puts the current PC on the top of the stack (push). The PC is then set to nnn.
    pub fn opcode_call(&mut self, nnn: u16) -> OpcodeExec {
        self.push(self.pc);
        self.pc(&nnn);
        Ok(format!("CALL {}", nnn))
    }

    // Skip next instruction if Vx = kk.
    // The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.
    pub fn opcode_se(&mut self, x: usize, kk: u8) -> OpcodeExec {
        if self.v_reg[x] == kk {
            self.pc_inc();
            self.pc_inc();
        }
        Ok(format!("SE {}, {}", x, kk))
    }

    // Skip next instruction if Vx != kk.
    // The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
    pub fn opcode_sne(&mut self, x: usize, kk: u8) -> OpcodeExec {
        if self.v_reg[x] != kk {
            self.pc_inc();
            self.pc_inc();
        }
        Ok(format!("SNE {}, {}", x, kk))
    }

    // Skip next instruction if Vx = Vy.
    // The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
    pub fn opcode_se_vy(&mut self, x: usize, y: usize) -> OpcodeExec {
        if self.v_reg[x] == self.v_reg[y] {
            self.pc_inc();
            self.pc_inc();
        }
        Ok(format!("SE {} {}", x, y))
    }

    // Set Vx = kk.
    // The interpreter puts the value kk into register Vx.
    pub fn opcode_ld(&mut self, x: usize, kk: u8) -> OpcodeExec {
        self.v_reg[x] = kk;
        Ok(format!("LD {} {}", x, kk))
    }

    // Set Vx = Vx + kk.
    // Adds the value kk to the value of register Vx, then stores the result in Vx.
    pub fn opcode_add(&mut self, x: usize, kk: u8) -> OpcodeExec {
        self.v_reg[x] = self.v_reg[x] + kk;
        Ok(format!("ADD {} {}", x, kk))
    }

    // Set Vx = Vy.
    // Stores the value of register Vy in register Vx.
    pub fn opcode_ld_vy(&mut self, x: usize, y: usize) -> OpcodeExec {
        self.v_reg[x] = self.v_reg[y];
        Ok(format!("LD {}, {}", x, y))
    }

    // Set Vx = Vx OR Vy.
    // Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx.
    // A bitwise OR compares the corrseponding bits from two values, and if either bit is 1,
    // then the same bit in the result is also 1. Otherwise, it is 0.
    pub fn opcode_or(&mut self, x: usize, y: usize) -> OpcodeExec {
        self.v_reg[x] = self.v_reg[x] | self.v_reg[y];
        Ok(format!("OR {}, {}", x, y))
    }

    // Set Vx = Vx AND Vy.
    // Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx.
    // A bitwise AND compares the corrseponding bits from two values, and if both bits are 1,
    // then the same bit in the result is also 1. Otherwise, it is 0.
    pub fn opcode_and(&mut self, x: usize, y: usize) -> OpcodeExec {
        self.v_reg[x] = self.v_reg[x] & self.v_reg[y];
        Ok(format!("AND {}, {}", x, y))
    }

    // Set Vx = Vx XOR Vy.
    // Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx.
    // An exclusive OR compares the corrseponding bits from two values, and if the bits are not both the same,
    // then the corresponding bit in the result is set to 1. Otherwise, it is 0.
    pub fn opcode_xor(&mut self, x: usize, y: usize) -> OpcodeExec {
        self.v_reg[x] = self.v_reg[x] ^ self.v_reg[y];
        Ok(format!("XOR {}, {}", x, y))
    }

    // Set Vx = Vx + Vy, set VF = carry.
    // The values of Vx and Vy are added together. If the result is greater than 8 bits (i.e., > 255,) VF is set to 1,
    // otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vx.
    pub fn opcode_add_vy(&mut self, x: usize, y: usize) -> OpcodeExec {
        let (res, overflow) = self.v_reg[x].overflowing_add(self.v_reg[y]);
        let vf = if overflow { 1 } else { 0 };

        self.v_reg[x] = res;
        self.v_reg[0xF] = vf;

        Ok(format!("ADD {}, {}", x, y))
    }

    // Set Vx = Vx - Vy, set VF = NOT borrow.
    // If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.
    pub fn opcode_sub(&mut self, x: usize, y: usize) -> OpcodeExec {
        let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
        let new_vf = if borrow { 0 } else { 1 };

        self.v_reg[x] = new_vx;
        self.v_reg[0xF] = new_vf;

        Ok(format!("SUB {}, {}", x, y))
    }

    // Set Vx = Vx SHR 1.
    // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
    pub fn opcode_shr(&mut self, x: usize) -> OpcodeExec {
        let least = self.v_reg[x] & 1;

        self.v_reg[0xF] = least;
        self.v_reg[x] >>= 1; // this is equal to /2

        Ok(format!("SHR {}", x))
    }

    // Set Vx = Vy - Vx, set VF = NOT borrow.
    // If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results stored in Vx.
    pub fn opcode_subn(&mut self, x: usize, y: usize) -> OpcodeExec {
        let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
        let new_vf = if borrow { 0 } else { 1 };

        self.v_reg[x] = new_vx;
        self.v_reg[0xF] = new_vf;

        Ok(format!("SUBN {}, {}", x, y))
    }

    // Set Vx = Vx SHL 1.
    // If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
    pub fn opcode_shl(&mut self, x: usize) -> OpcodeExec {
        let most = (self.v_reg[x] >> 7) & 1;

        self.v_reg[0xF] = most;
        self.v_reg[x] <<= 1; // this is equal to *2

        Ok(format!("SHL {}", x))
    }

    // Skip next instruction if Vx != Vy.
    // The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2.
    pub fn opcode_sne_vy(&mut self, x: usize, y: usize) -> OpcodeExec {
        if self.v_reg[x] != self.v_reg[y] {
            self.pc_inc();
            self.pc_inc();
        }
        Ok(format!("SNE {}, {}", x, y))
    }


    // Set I = nnn.
    // The value of register I is set to nnn.
    pub fn opcode_ld_i(&mut self, nnn: u16) -> OpcodeExec {
        self.i = nnn;
        Ok(format!("LD I, {}", nnn))
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{assert_eq, println};

    #[test]
    fn test_load_program() {
        let program = vec![1, 2, 3, 4, 5, 6, 7];
        let mut chip_8 = Chip8::default();
        chip_8.load_program(&program);

        let start = PROGRAM_START_ADDRESS as usize;
        let end = start + program.len();
        assert_eq!(program, chip_8.memory[start..end]);
    }

    #[test]
    fn test_opcode_cls() {
        let mut chip_8 = Chip8::default();

        assert_eq!(chip_8.display, [false; SCREEN_WIDTH * SCREEN_HEIGTH]);

        chip_8.display = [true; SCREEN_WIDTH * SCREEN_HEIGTH];

        chip_8.opcode_cls().unwrap();

        assert_eq!(chip_8.display, [false; SCREEN_WIDTH * SCREEN_HEIGTH]);
    }

    #[test]
    fn test_opcode_ret() {
        let mut chip_8 = Chip8::default();

        assert_eq!(chip_8.pc, PROGRAM_START_ADDRESS);

        chip_8.push(PROGRAM_START_ADDRESS + 5);
        chip_8.opcode_ret().unwrap();

        assert_eq!(chip_8.pc, PROGRAM_START_ADDRESS + 5);
    }

    #[test]
    fn test_opcode_jmp() {
        let mut chip_8 = Chip8::default();

        assert_eq!(chip_8.pc, PROGRAM_START_ADDRESS);

        chip_8.opcode_jmp(PROGRAM_START_ADDRESS + 3).unwrap();

        assert_eq!(chip_8.pc, PROGRAM_START_ADDRESS + 3);
    }

    #[test]
    fn test_opcode_call() {
        let mut chip_8 = Chip8::default();

        assert_eq!(chip_8.pc, PROGRAM_START_ADDRESS);

        chip_8.opcode_call(PROGRAM_START_ADDRESS + 3).unwrap();

        assert_eq!(chip_8.pop(), PROGRAM_START_ADDRESS);
        assert_eq!(chip_8.pc, PROGRAM_START_ADDRESS + 3);
    }

    #[test]
    fn test_opcode_se() {
        let mut chip_8 = Chip8::default();
        let val = 0b01010101;
        let x = 0;

        chip_8.opcode_ld(x, val).unwrap();

        assert_eq!(chip_8.pc, PROGRAM_START_ADDRESS);

        chip_8.opcode_se(x, val).unwrap();

        assert_eq!(chip_8.pc, PROGRAM_START_ADDRESS + 2);
    }

    #[test]
    fn test_opcode_sne() {
        let mut chip_8 = Chip8::default();
        let val = 0b01010101;
        let x = 0;

        chip_8.opcode_ld(x, val).unwrap();

        assert_eq!(chip_8.pc, PROGRAM_START_ADDRESS);

        chip_8.opcode_sne(x, 0b11111111).unwrap();

        assert_eq!(chip_8.pc, PROGRAM_START_ADDRESS + 2);
    }

    #[test]
    fn test_opcode_se_vy() {
        let mut chip_8 = Chip8::default();
        let val = 0b01010101;
        let x = 0;
        let y = 1;

        chip_8.opcode_ld(x, val).unwrap();
        chip_8.opcode_ld(y, 0b01010101).unwrap();

        assert_eq!(chip_8.pc, PROGRAM_START_ADDRESS);

        chip_8.opcode_se_vy(x, y).unwrap();

        assert_eq!(chip_8.pc, PROGRAM_START_ADDRESS + 2);
    }

    #[test]
    fn test_opcode_ld() {
        let mut chip_8 = Chip8::default();
        chip_8.opcode_ld(0, 0b01010101).unwrap();

        assert_eq!(chip_8.v_reg[1], 0);
        assert_eq!(chip_8.v_reg[0], 0b01010101);
    }

    #[test]
    fn test_opcode_add() {
        let mut chip_8 = Chip8::default();
        let x = 0;

        chip_8.opcode_ld(x, 3).unwrap();

        assert_eq!(chip_8.v_reg[x], 3);

        chip_8.opcode_add(x, 1).unwrap();

        assert_eq!(chip_8.v_reg[x], 4);
    }

    #[test]
    fn test_opcode_ld_vy() {
        let mut chip_8 = Chip8::default();
        let x = 0;
        let y = 1;

        chip_8.opcode_ld(x, 0b01010101).unwrap();
        chip_8.opcode_ld(y, 0b11111111).unwrap();

        assert_eq!(chip_8.v_reg[x], 0b01010101);
        assert_eq!(chip_8.v_reg[y], 0b11111111);

        chip_8.opcode_ld_vy(x, y).unwrap();

        assert_eq!(chip_8.v_reg[x], 0b11111111);
        assert_eq!(chip_8.v_reg[y], 0b11111111);
    }

    #[test]
    fn test_or() {
        let x = 0;
        let y = 10;
        let mut chip_8 = Chip8::default();
        chip_8.opcode_ld(x, 0b01010101).unwrap();
        chip_8.opcode_ld(y, 0b11001001).unwrap();

        chip_8.opcode_or(x, y).unwrap();

        assert_eq!(chip_8.v_reg[x], 0b11011101);
    }

    #[test]
    fn test_and() {
        let x = 0;
        let y = 10;
        let mut chip_8 = Chip8::default();
        chip_8.opcode_ld(x, 0b01010101).unwrap();
        chip_8.opcode_ld(y, 0b11001001).unwrap();

        chip_8.opcode_and(x, y).unwrap();

        assert_eq!(chip_8.v_reg[x], 0b01000001);
    }

    #[test]
    fn test_xor() {
        let x = 0;
        let y = 10;
        let mut chip_8 = Chip8::default();
        chip_8.opcode_ld(x, 0b01010101).unwrap();
        chip_8.opcode_ld(y, 0b11001001).unwrap();

        chip_8.opcode_xor(x, y).unwrap();

        assert_eq!(chip_8.v_reg[x], 0b10011100);
    }

    #[test]
    fn test_opcode_add_vy() {
        let mut chip_8 = Chip8::default();
        let x = 0;
        let y = 10;

        chip_8.opcode_ld(x, 255).unwrap();
        chip_8.opcode_ld(y, 1).unwrap();

        assert_eq!(chip_8.v_reg[0xF], 0);

        chip_8.opcode_add_vy(x, y).unwrap();

        // overflows
        assert_eq!(chip_8.v_reg[x], 0);
        assert_eq!(chip_8.v_reg[0xF], 1);

        chip_8.opcode_ld(x, 254).unwrap();
        chip_8.opcode_ld(y, 1).unwrap();

        assert_eq!(chip_8.v_reg[0xF], 1);

        chip_8.opcode_add_vy(x, y).unwrap();

        // still fits in a u8
        assert_eq!(chip_8.v_reg[x], 255);
        assert_eq!(chip_8.v_reg[0xF], 0);
    }

    #[test]
    fn test_opcode_sub() {
        let mut chip_8 = Chip8::default();
        let x = 0;
        let y = 2;

        chip_8.opcode_ld(x, 3).unwrap();
        chip_8.opcode_ld(y, 1).unwrap();

        assert_eq!(chip_8.v_reg[0xF], 0);

        chip_8.opcode_sub(x, y).unwrap();

        // normal sub
        assert_eq!(chip_8.v_reg[0xF], 1);
        assert_eq!(chip_8.v_reg[x], 2);

        chip_8.opcode_ld(x, 1).unwrap();
        chip_8.opcode_ld(y, 3).unwrap();

        assert_eq!(chip_8.v_reg[0xF], 1);

        chip_8.opcode_sub(x, y).unwrap();

        // overflows
        assert_eq!(chip_8.v_reg[0xF], 0);
        assert_eq!(chip_8.v_reg[x], 254);
    }

    #[test]
    fn test_opcode_shr() {
        let mut chip_8 = Chip8::default();
        let x = 0;

        chip_8.opcode_ld(x, 7).unwrap();

        assert_eq!(chip_8.v_reg[0xF], 0);

        chip_8.opcode_shr(x).unwrap();

        assert_eq!(chip_8.v_reg[x], 3);
        assert_eq!(chip_8.v_reg[0xF], 1);
    }

    #[test]
    fn test_opcode_subn() {
        let mut chip_8 = Chip8::default();
        let x = 0;
        let y = 2;

        chip_8.opcode_ld(x, 1).unwrap();
        chip_8.opcode_ld(y, 3).unwrap();

        assert_eq!(chip_8.v_reg[0xF], 0);

        chip_8.opcode_subn(x, y).unwrap();

        // normal sub
        assert_eq!(chip_8.v_reg[0xF], 1);
        assert_eq!(chip_8.v_reg[x], 2);

        chip_8.opcode_ld(x, 3).unwrap();
        chip_8.opcode_ld(y, 1).unwrap();

        assert_eq!(chip_8.v_reg[0xF], 1);

        chip_8.opcode_subn(x, y).unwrap();

        // overflows
        assert_eq!(chip_8.v_reg[0xF], 0);
        assert_eq!(chip_8.v_reg[x], 254);
    }

    #[test]
    fn test_opcode_shl() {
        let mut chip_8 = Chip8::default();
        let x = 0;

        chip_8.opcode_ld(x, 250).unwrap();

        assert_eq!(chip_8.v_reg[0xF], 0);

        chip_8.opcode_shl(x).unwrap();

        assert_eq!(chip_8.v_reg[x], 244);
        assert_eq!(chip_8.v_reg[0xF], 1);
    }

    #[test]
    fn test_opcode_sne_vy() {
        let mut chip_8 = Chip8::default();
        let x = 0;
        let y = 10;

        chip_8.opcode_ld(x, 0b01010101).unwrap();
        chip_8.opcode_ld(y, 0b11111101).unwrap();

        assert_eq!(chip_8.pc, PROGRAM_START_ADDRESS);

        chip_8.opcode_sne_vy(x, y).unwrap();

        assert_eq!(chip_8.pc, PROGRAM_START_ADDRESS + 2);
    }

    #[test]
    fn test_opcode_ld_i() {
        let mut chip_8 = Chip8::default();

        assert_eq!(chip_8.i, 0);

        chip_8.opcode_ld_i(8).unwrap();

        assert_eq!(chip_8.i, 8);
    }

    #[test]
    fn test_opcode_jp_v0() {
        println!("should be implemented");
        assert!(false)
    }

    #[test]
    fn test_opcode_rnd() {
        println!("should be implemented");
        assert!(false)
    }

    #[test]
    fn test_opcode_drw() {
        println!("should be implemented");
        assert!(false)
    }

    #[test]
    fn test_opcode_skp() {
        println!("should be implemented");
        assert!(false)
    }

    #[test]
    fn test_opcode_sknp() {
        println!("should be implemented");
        assert!(false)
    }

    #[test]
    fn test_opcode_ld_vx_dt() {
        println!("should be implemented");
        assert!(false)
    }

    #[test]
    fn test_opcode_ld_vx_k() {
        println!("should be implemented");
        assert!(false)
    }

    #[test]
    fn test_opcode_ld_dt_vx() {
        println!("should be implemented");
        assert!(false)
    }

    #[test]
    fn test_opcode_ld_st_vx() {
        println!("should be implemented");
        assert!(false)
    }

    #[test]
    fn test_opcode_add_1_vx() {
        println!("should be implemented");
        assert!(false)
    }

    #[test]
    fn test_opcode_ld_f_vx() {
        println!("should be implemented");
        assert!(false)
    }

    #[test]
    fn test_opcode_ld_b_vx() {
        println!("should be implemented");
        assert!(false)
    }

    #[test]
    fn test_opcode_ld_1_vx() {
        println!("should be implemented");
        assert!(false)
    }

    #[test]
    fn test_opcode_ld_vx_1() {
        println!("should be implemented");
        assert!(false)
    }
}
