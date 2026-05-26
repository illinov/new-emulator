use crate::cpu::AddressingMode;
use std::sync::LazyLock;


pub (crate) struct OpCode {
    pub code: u8,
    pub mnemonic: &'static str,
    pub len: u8,
    pub cycles: u8, // Ciclos de reloj que tarda en ejecutarse la instruccion
    pub mode: AddressingMode,
}

macro_rules! opcode_table {
    ( $( $code:literal => $mn:literal, $len:literal, $cycles:literal, $mode:expr );* $(;)? ) => {{
        let mut table: [Option<OpCode>; 256] = [const { None }; 256];
        $(
            table[$code] = Some(OpCode {
                code: $code,
                mnemonic: $mn,
                len: $len,
                cycles: $cycles,
                mode: $mode,
            });
        )*
        table
    }};
}

pub(crate) static OPCODES: LazyLock<[Option<OpCode>; 256]> = LazyLock::new(|| opcode_table![
    0x00 => "BRK", 1, 7, AddressingMode::NoneAddressing;
    0xaa => "TAX", 1, 2, AddressingMode::NoneAddressing;
    0xe8 => "INX", 1, 2, AddressingMode::NoneAddressing;

    0xa9 => "LDA", 2, 2, AddressingMode::Immediate;
    0xa5 => "LDA", 2, 3, AddressingMode::ZeroPage;
    0xb5 => "LDA", 2, 4, AddressingMode::ZeroPage_X;
    0xad => "LDA", 3, 4, AddressingMode::Absolute;
    0xbd => "LDA", 3, 4 /*+1 if page crossed*/, AddressingMode::Absolute_X;
    0xb9 => "LDA", 3, 4 /*+1 if page crossed*/, AddressingMode::Absolute_Y;
    0xa1 => "LDA", 2, 6, AddressingMode::Indirect_X;
    0xb1 => "LDA", 2, 5 /*+1 if page crossed*/, AddressingMode::Indirect_Y;

    0x85 => "STA", 2, 3, AddressingMode::ZeroPage;
    0x95 => "STA", 2, 4, AddressingMode::ZeroPage_X;
    0x8d => "STA", 3, 4, AddressingMode::Absolute;
    0x9d => "STA", 3, 5, AddressingMode::Absolute_X;
    0x99 => "STA", 3, 5, AddressingMode::Absolute_Y;
    0x81 => "STA", 2, 6, AddressingMode::Indirect_X;
    0x91 => "STA", 2, 6, AddressingMode::Indirect_Y;
]);

