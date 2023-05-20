type OpcodeExec = Result<String, String>;

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
        let n1 = (op & 0xF000) >> 12;
        let n2 = (op & 0x0F00) >> 8;
        let n3 = (op & 0x00F0) >> 4;
        let n4 = op & 0x000F;

        // decode and execute opcode
        let result: OpcodeExec = match (n1, n2, n3, n4) {
            (0x0, 0x0, 0xE, 0x0) => self.opcode_cls(), // CLS
            (0x0, 0x0, 0xE, 0xE) => self.opcode_ret(), // RET
            (0x0, _, _, _) => self.opcode_sys(self.nnn_address(op)), // SYS addr
            (0x1, _, _, _) => self.opcode_jmp(self.nnn_address(op)), // JP addr
            (0x2, _, _, _) => self.opcode_call(self.nnn_address(op)), // CALL addr
            (0x3, _, _, _) => self.opcode_se(n2 as usize, self.kk(op)), // SE Vx, byte
            (0x4, _, _, _) => self.opcode_sne(n2 as usize, self.kk(op)), // SNE Vx, byte
            (0x5, _, _, 0) => self.opcode_sey(n2 as usize, n3 as usize), // SE Vx Vy
            (0x6, _, _, _) => self.opcode_ld(n2 as usize, self.kk(op)), // LD Vx, byte
            (0x7, _, _, _) => self.opcode_add(n2 as usize, self.kk(op)), // ADD Vx, byte
            (0x8, _, _, 0) => self.opcode_ldy(n2 as usize, n3 as usize), // LD Vx, Vy
            (_, _, _, _) => Err(format!("Unimplemented opcode: {}", op)),
        };

        if (&result).is_err() {
            println!("Opcode execution error: {}", result.unwrap_err());
            return;
        }

        // advance program counter (memory is [u8], opcode is u16, that's why we advance 'pc' by two)
        self.increase_program_counter();
        self.increase_program_counter();

        println!("Opcode executed: {}", result.unwrap());
    }

    fn nnn_address(&self, op: u16) -> u16 {
        op & 0xFFF
    }

    fn kk(&self, op: u16) -> u8 {
        (op & 0xFF) as u8
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

    fn set_pc(&mut self, pc: u16) {
        // todo(fedejinich) no error handling, should restrict pc to fit in memory range?
        self.pc = pc;
    }

    fn increase_program_counter(&mut self) {
        self.pc += 1;
    }

    // OPCODES

    // Jump to a machine code routine at nnn.
    // This instruction is only used on the old computers on which Chip-8 was originally implemented. It is ignored by modern interpreters.
    fn opcode_sys(&self, _nnn: u16) -> OpcodeExec {
        Ok(String::from("SYS"))
    }

    // Clear the display
    fn opcode_cls(&mut self) -> OpcodeExec {
        self.display = [false; SCREEN_WIDTH * SCREEN_HEIGTH];
        Ok(String::from("CLS"))
    }

    // Return from a subroutine.
    // The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
    fn opcode_ret(&mut self) -> OpcodeExec {
        let new_pc = self.pop();
        self.set_pc(new_pc);
        Ok(String::from("RET"))
    }

    // Jump to location nnn.
    // The interpreter sets the program counter to nnn.
    fn opcode_jmp(&mut self, nnn: u16) -> OpcodeExec {
        self.set_pc(nnn);
        Ok(format!("JMP {}", nnn))
    }

    // Call subroutine at nnn.
    // The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to nnn.
    fn opcode_call(&mut self, nnn: u16) -> OpcodeExec {
        self.push(self.pc);
        self.set_pc(nnn);
        Ok(format!("CALL {}", nnn))
    }

    // Skip next instruction if Vx = kk.
    // The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.
    fn opcode_se(&mut self, x: usize, kk: u8) -> OpcodeExec {
        if self.v_reg[x] == kk {
            self.increase_program_counter();
        }
        Ok(format!("SE v{}, {}", x, kk))
    }

    // Skip next instruction if Vx != kk.
    // The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
    fn opcode_sne(&mut self, x: usize, kk: u8) -> OpcodeExec {
        if self.v_reg[x] != kk {
            self.increase_program_counter();
        }
        Ok(format!("SNE v{}, {}", x, kk))
    }

    // Skip next instruction if Vx = Vy.
    // The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
    fn opcode_sey(&mut self, x: usize, y: usize) -> OpcodeExec {
        if self.v_reg[x] == self.v_reg[y] {
            self.increase_program_counter();
        }
        Ok(format!("SE vx{} vy{}", x, y))
    }

    // Set Vx = kk.
    // The interpreter puts the value kk into register Vx.
    fn opcode_ld(&mut self, x: usize, kk: u8) -> OpcodeExec {
        self.v_reg[x] = kk;
        Ok(format!("LD vx{} {}", x, kk))
    }

    // Set Vx = Vx + kk.
    // Adds the value kk to the value of register Vx, then stores the result in Vx.
    fn opcode_add(&mut self, x: usize, kk: u8) -> OpcodeExec {
        self.v_reg[x] = self.v_reg[x] + kk;
        Ok(format!("ADD vx{} {}", x, kk))
    }

    // Set Vx = Vy.
    // Stores the value of register Vy in register Vx.
    fn opcode_ldy(&mut self, x: usize, y: usize) -> OpcodeExec {
        self.v_reg[x] = self.v_reg[y];
        Ok(format!("LD vx{}, vy{}", x, y))
    }
}

#[test]
fn test_load_program() {
    let program = vec![1, 2, 3, 4, 5, 6, 7];
    let mut chip_8 = Chip8::default();
    chip_8.load_program(&program);

    let start = PROGRAM_START_ADDRESS as usize;
    let end = start + program.len();
    assert_eq!(program, chip_8.memory[start..end]);
}
