use crate::opcodes::{match_opcode, OpcodeExec};

const PROGRAM_START_ADDRESS: u16 = 0x200;

const MEM_SIZE: usize = 4096;
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
    pub fn tick(&mut self) {
        let op = self.fetch_op();
        self.execute_op(op);
        println!("tick")
    }

    // todo(fedejinich) add unit test for this
    // fetches the 16-bit opcode stored in memory at the program counter
    fn fetch_op(&self) -> u16 {
        let high = self.memory[self.pc as usize] as u16;
        let low = self.memory[(self.pc as usize) + 1] as u16;
        let op = (high >> 8) | low;
        op
    }

    // todo(fedejinich) add unit test for this
    // decodes the given opcode and executes it
    fn execute_op(&mut self, op: u16) {
        let opcode = match_opcode(op);

        let result = opcode.execute_op(self);

        if (&result).is_err() {
            println!("Opcode execution error: {}", result.unwrap_err());
            return;
        }

        // advance program counter (memory is [u8], opcode is u16, that's why we advance 'pc' by two)
        self.increase_program_counter();
        self.increase_program_counter();

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

    fn set_pc(&mut self, pc: &u16) {
        // todo(fedejinich) no error handling, should restrict pc to fit in memory range?
        self.pc = pc.clone();
    }

    fn increase_program_counter(&mut self) {
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
        self.set_pc(&new_pc);
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
        self.set_pc(&nnn);
        Ok(format!("JMP {}", nnn))
    }

    // Call subroutine at nnn.
    // The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to nnn.
    pub fn opcode_call(&mut self, nnn: u16) -> OpcodeExec {
        self.push(self.pc);
        self.set_pc(&nnn);
        Ok(format!("CALL {}", nnn))
    }

    // Skip next instruction if Vx = kk.
    // The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.
    pub fn opcode_se(&mut self, x: usize, kk: u8) -> OpcodeExec {
        if self.v_reg[x] == kk {
            self.increase_program_counter();
        }
        Ok(format!("SE v{}, {}", x, kk))
    }

    // Skip next instruction if Vx != kk.
    // The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
    pub fn opcode_sne(&mut self, x: usize, kk: u8) -> OpcodeExec {
        if self.v_reg[x] != kk {
            self.increase_program_counter();
        }
        Ok(format!("SNE v{}, {}", x, kk))
    }

    // Skip next instruction if Vx = Vy.
    // The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
    pub fn opcode_se_vy(&mut self, x: usize, y: usize) -> OpcodeExec {
        if self.v_reg[x] == self.v_reg[y] {
            self.increase_program_counter();
        }
        Ok(format!("SE vx{} vy{}", x, y))
    }

    // Set Vx = kk.
    // The interpreter puts the value kk into register Vx.
    pub fn opcode_ld(&mut self, x: usize, kk: u8) -> OpcodeExec {
        self.v_reg[x] = kk;
        Ok(format!("LD vx{} {}", x, kk))
    }

    // Set Vx = Vx + kk.
    // Adds the value kk to the value of register Vx, then stores the result in Vx.
    pub fn opcode_add(&mut self, x: usize, kk: u8) -> OpcodeExec {
        self.v_reg[x] = self.v_reg[x] + kk;
        Ok(format!("ADD vx{} {}", x, kk))
    }

    // Set Vx = Vy.
    // Stores the value of register Vy in register Vx.
    pub fn opcode_ld_vy(&mut self, x: usize, y: usize) -> OpcodeExec {
        self.v_reg[x] = self.v_reg[y];
        Ok(format!("LD vx{}, vy{}", x, y))
    }

    // Set Vx = Vx OR Vy.
    // Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx.
    // A bitwise OR compares the corrseponding bits from two values, and if either bit is 1,
    // then the same bit in the result is also 1. Otherwise, it is 0.
    pub fn opcode_or(&mut self, x: usize, y: usize) -> OpcodeExec {
        self.v_reg[x] = self.v_reg[x] | self.v_reg[y];
        Ok(format!("OR vx{}, vy{}", x, y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_program() {
        let program = vec![1, 2, 3, 4, 5, 6, 7];
        let mut chip_8 = Chip8::default();
        chip_8.load_program(&program);

        let start = PROGRAM_START_ADDRESS as usize;
        let end = start + program.len();
        assert_eq!(program, chip_8.memory[start..end]);
    }
}
