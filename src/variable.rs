#[allow(dead_code)]
pub mod token {
    pub const SECTION: &str = "SECTION";

    pub const EQU: &str = "EQU";
    pub const SET: &str = "SET";
    pub const RSSET: &str = "RSSET";
    pub const RSRESET: &str = "RSRESET";
    pub const RB: &str = "RB";
    pub const RW: &str = "RW";
    pub const EQUS: &str = "EQUS";
    pub const MACRO: &str = "MACRO";
    pub const ENDM: &str = "ENDM";
    pub const MACRO_TOKEN: [&str; 8] = [
        "IF", "ELIF", "ELSE", "ENDC", "PRINTT", "PRINTI", "PRINTV", "PRINTF",
    ];

    pub const REPT: &str = "REPT";
    pub const ENDR: &str = "ENDR";

    pub const INCLUDE: &str = "INCLUDE";
    pub const INCBIN: &str = "INCBIN";
}

pub mod opcode {
    pub const ADC: &str = "ADC";
    pub const ADD: &str = "ADD";
    pub const AND: &str = "AND";
    pub const CP: &str = "CP";
    pub const DEC: &str = "DEC";
    pub const INC: &str = "INC";
    pub const OR: &str = "OR";
    pub const SBC: &str = "SBC";
    pub const SUB: &str = "SUB";
    pub const XOR: &str = "XOR";

    pub const BIT: &str = "BIT";
    pub const RES: &str = "RES";
    pub const SET: &str = "SET";
    pub const SWAP: &str = "SWAP";

    pub const RL: &str = "RL";
    pub const RLA: &str = "RLA";
    pub const RLC: &str = "RLC";
    pub const RLCA: &str = "RLCA";
    pub const RR: &str = "RR";
    pub const RRA: &str = "RRA";
    pub const RRC: &str = "RRC";
    pub const RRCA: &str = "RRCA";
    pub const SLA: &str = "SLA";
    pub const SRA: &str = "SRA";
    pub const SRL: &str = "SRL";

    pub const LD: &str = "LD";
    pub const LDH: &str = "LDH";

    pub const CALL: &str = "CALL";
    pub const JP: &str = "JP";
    pub const JR: &str = "JR";
    pub const RET: &str = "RET";
    pub const RETI: &str = "RETI";
    pub const RST: &str = "RST";

    pub const POP: &str = "POP";
    pub const PUSH: &str = "PUSH";

    pub const CCF: &str = "CCF";
    pub const CPL: &str = "CPL";
    pub const DAA: &str = "DAA";
    pub const DI: &str = "DI";
    pub const EI: &str = "EI";
    pub const HALT: &str = "HALT";
    pub const NOP: &str = "NOP";
    pub const SCF: &str = "SCF";
    pub const STOP: &str = "STOP";

    #[allow(dead_code)]
    pub const COMMENT: &str = ";";

    pub const OPCODE_LIST: [&str; 46] = [
        ADC, ADD, AND, CP, DEC, INC, OR, SBC, SUB, XOR, BIT, RES, SET, SWAP, RL, RLA, RLC, RLCA,
        RR, RRA, RRC, RRCA, SLA, SRA, SRL, LD, LDH, CALL, JP, JR, RET, RETI, RST, POP, PUSH, CCF,
        CPL, DAA, DI, EI, HALT, NOP, SCF, STOP, "JPBA", "CALLBA",
    ];

    #[allow(dead_code)]
    pub const JUMP_LIST: [&str; 3] = [JP, JR, CALL];

    #[allow(dead_code)]
    pub const DEFINE_LIST: [&str; 3] = ["DB", "DW", "DL"];
}
