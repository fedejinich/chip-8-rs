use crate::chip_8::Chip8;

pub type OpcodeExec = Result<String, String>;

pub enum Opcode {
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

pub fn match_opcode(op: u16) -> Opcode {
    let n1 = (op & 0xF000) >> 12;
    let n2 = ((op & 0x0F00) >> 8) as usize;
    let n3 = ((op & 0x00F0) >> 4) as usize;
    let n4 = op & 0x000F;

    let result: Opcode = match (n1, n2, n3, n4) {
        (0x0, 0x0, 0xE, 0x0) => Opcode::CLS,
        (0x0, 0x0, 0xE, 0xE) => Opcode::RET,
        (0x0, _, _, _) => Opcode::SYS(nnn_address(op)),
        (0x1, _, _, _) => Opcode::JP(nnn_address(op)),
        (0x2, _, _, _) => Opcode::CALL(nnn_address(op)),
        (0x3, _, _, _) => Opcode::SE(n2, kk(op)),
        (0x4, _, _, _) => Opcode::SNE(n2, kk(op)),
        (0x5, _, _, 0) => Opcode::SEVy(n2, n3),
        (0x6, _, _, _) => Opcode::LD(n2, kk(op)),
        (0x7, _, _, _) => Opcode::ADD(n2, kk(op)),
        (0x8, _, _, 0) => Opcode::LDVy(n2, n3),
        (0x8, _, _, 1) => Opcode::OR(n2, n3),

        (_, _, _, _) => Opcode::ERROR(format!("Unimplemented opcode: {}", op)),
    };

    result
}

fn nnn_address(op: u16) -> u16 {
    op & 0xFFF
}

fn kk(op: u16) -> u8 {
    (op & 0xFF) as u8
}

impl Opcode {
    pub fn execute_op(&self, chip_8: &mut Chip8) -> OpcodeExec {
        match self {
            Opcode::CLS => chip_8.opcode_cls(),
            Opcode::RET => chip_8.opcode_ret(),
            Opcode::SYS(nnn) => chip_8.opcode_sys(*nnn),
            Opcode::JP(nnn) => chip_8.opcode_jmp(*nnn),
            Opcode::CALL(nnn) => chip_8.opcode_call(*nnn),
            Opcode::SE(vx, kk) => chip_8.opcode_se(*vx, *kk),
            Opcode::SNE(vx, kk) => chip_8.opcode_sne(*vx, *kk),
            Opcode::SEVy(vx, vy) => chip_8.opcode_se_vy(*vx, *vy),
            Opcode::LD(vx, kk) => chip_8.opcode_ld(*vx, *kk),
            Opcode::ADD(vx, vy) => chip_8.opcode_add(*vx, *vy),
            Opcode::LDVy(vx, vy) => chip_8.opcode_ld_vy(*vx, *vy),
            Opcode::OR(vx, vy) => chip_8.opcode_or(*vx, *vy),
            Opcode::ERROR(e) => Err(e.clone()),
        }
    }
}
