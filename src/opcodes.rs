use crate::chip_8::Chip8;

type OpCodeExec = Result<String, String>;

pub enum OpCode {
    ERROR(String),
    CLS,                // CLS
    RET,                // RET
    SYS(u16),           // SYS addr
    JP(u16),            // JP addr
    CALL(u16),          // CALL addr
    SE(usize, u8),      // SE Vx, byte
    SNE(usize, u8),     // SNE Vx, byte
    SEVy(usize, usize), // SE Vx Vy
    LD(usize, u8),      // LD Vx, byte
    ADD(usize, u8),     // ADD Vx, byte
    LDVy(usize, usize), // LD Vx, Vy
    OR(usize, usize),   // OR Vx, Vy
}

pub fn match_opcode(op: u16) -> OpCode {
    let n1 = (op & 0xF000) >> 12;
    let n2 = ((op & 0x0F00) >> 8) as usize;
    let n3 = ((op & 0x00F0) >> 4) as usize;
    let n4 = op & 0x000F;

    let result: OpCode = match (n1, n2, n3, n4) {
        (0x0, 0x0, 0xE, 0x0) => OpCode::CLS,
        (0x0, 0x0, 0xE, 0xE) => OpCode::RET,
        (0x0, _, _, _) => OpCode::SYS(nnn_address(op)),
        (0x1, _, _, _) => OpCode::JP(nnn_address(op)),
        (0x2, _, _, _) => OpCode::CALL(nnn_address(op)),
        (0x3, _, _, _) => OpCode::SE(n2, kk(op)),
        (0x4, _, _, _) => OpCode::SNE(n2, kk(op)),
        (0x5, _, _, 0) => OpCode::SEVy(n2, n3),
        (0x6, _, _, _) => OpCode::LD(n2, kk(op)),
        (0x7, _, _, _) => OpCode::ADD(n2, kk(op)),
        (0x8, _, _, 0) => OpCode::LDVy(n2, n3),
        (0x8, _, _, 1) => OpCode::OR(n2, n3),

        (_, _, _, _) => OpCode::ERROR(format!("Unimplemented opcode: {}", op)),
    };

    result
}

fn nnn_address(op: u16) -> u16 {
    op & 0xFFF
}

fn kk(op: u16) -> u8 {
    (op & 0xFF) as u8
}

impl OpCode {
    pub fn execute_op(&self, chip_8: &mut Chip8) -> OpCodeExec {
        match self {
            OpCode::CLS => chip_8.opcode_cls(),
            OpCode::RET => chip_8.opcode_ret(),
            OpCode::SYS(nnn) => chip_8.opcode_sys(*nnn),
            OpCode::JP(nnn) => chip_8.opcode_jmp(*nnn),
            OpCode::CALL(nnn) => chip_8.opcode_call(*nnn),
            OpCode::SE(vx, kk) => chip_8.opcode_se(*vx, *kk),
            OpCode::SNE(vx, kk) => chip_8.opcode_sne(*vx, *kk),
            OpCode::SEVy(vx, vy) => chip_8.opcode_se_vy(*vx, *vy),
            OpCode::LD(vx, kk) => chip_8.opcode_ld(*vx, *kk),
            OpCode::ADD(vx, vy) => chip_8.opcode_add(*vx, *vy),
            OpCode::LDVy(vx, vy) => chip_8.opcode_ld_vy(*vx, *vy),
            OpCode::OR(vx, vy) => chip_8.opcode_or(*vx, *vy),
            OpCode::ERROR(e) => Err(e.clone()),
        }
    }
}
