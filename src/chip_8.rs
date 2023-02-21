enum Opcode {
    SYS(u16),
}

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
    pub fn load_program(&mut self, program: &[u8]) {
        let start = PROGRAM_START_ADDRESS as usize;
        let end = start + program.len();
        self.memory[start..end].copy_from_slice(program);
        println!("program loaded");
    }

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
        let d1 = (op & 0xF000) >> 12;
        let d2 = (op & 0x0F00) >> 8;
        let d3 = (op & 0x00F0) >> 4;
        let d4 = op & 0x000F;

        // decode and execute opcode
        let result: OpcodeExec = match (d1, d2, d3, d4) {
            (0x0, d2, d3, d4) => self.opcode_sys(d2, d3, d4), // SYS addr
            (0x0, 0x0, 0xE, 0x0) => self.opcode_cls(),        // CLS
            (_, _, _, _) => Err(format!("Unimplemented opcode: {}", op)),
        };

        if (&result).is_err() {
            println!("Opcode execution error: {}", result.unwrap_err());
            return;
        }

        // advance program counter
        self.pc += 2;

        println!("Opcode executed: {}", result.unwrap());
    }

    fn opcode_sys(&self, d2: u16, d3: u16, d4: u16) -> OpcodeExec {
        let addr = (d2 << 8) | (d3 << 4) | d4;
        todo!()
    }

    fn opcode_cls(&mut self) -> OpcodeExec {
        self.display = [false; SCREEN_WIDTH * SCREEN_HEIGTH];
        Ok(String::from("CLS"))
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
