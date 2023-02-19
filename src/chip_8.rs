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
