pub (crate) struct CPU {
    pub register_a: u8, // Acumulador: Para operaciones aritmeticas
    pub status: u8, // Registro de 8 bits donde cada bit es una flag
    pub program_counter: u16, // Para poder direccionar las 65535 direcciones de memoria
    pub register_x: u8, // Index Register X: Para loops o indices
}

impl CPU {
    pub fn new() -> Self {
        Self {
            register_a: 0,
            status: 0,
            program_counter: 0,
            register_x: 0,
        }
    }

    fn lda(&mut self, value: u8) {
        self.register_a = value; // Guardar el parametro en el acumulador
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        // Zero Flag: El valor obtenido incluye cero
        if result == 0 {
            self.status = self.status | 0b0000_0010; // Activa Zero Flag
        } else {
            self.status = self.status & 0b1111_1101; // Apaga ZF sin tocar el resto de flags
        }

        // Flag N (Negative): El valor obtenido tiene el bit mas significativo prendido
        if result & 0b1000_0000 != 0 { // Ver si el primer bit esta prendido
            self.status = self.status | 0b1000_0000; // Activar Flag N
        } else {
            self.status = self.status & 0b0111_1111; // Apaga Flag N sin tocar el resto de flags
        }
    }

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.program_counter = 0; // En este caso usamos el PC como contador de indices

        loop {
            let opscode = program[self.program_counter as usize];
            self.program_counter += 1;

            match opscode {
                // LDA (Load Accumulator): Carga un valor en el acumulador
                0xA9 => { 
                    // Tomar el primer argumento
                    let param = program[self.program_counter as usize];
                    self.program_counter += 1;

                    self.lda(param) // Guardar el parametro en el acumulador
                }
                
                // TAX: Transfer A to X
                0xAA => self.tax(),

                // INX: Increase X register + 1
                0xe8 => self.inx(),
                
                // Break
                0x00 => return,

                _ => todo!()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9,0x05,0x00]);
        assert_eq!(cpu.register_a, 0x05); // Se carga el valor correctamente
        assert!(cpu.status & 0b0000_0010 == 0b00); // No hay flag Z
        assert!(cpu.status & 0b1000_0000 == 0); // No hay Flag N
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9,0x00,0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10); // Si se activa la Z
    }

    #[test]
    fn test_0xa9_lda_flag_negative() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9,0x80,0x00]);
        assert!(cpu.status & 0b1000_0000 == 0b1000_0000); // Se activa N
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.register_a = 10;
        cpu.interpret(vec![0xaa,0x00]);

        assert_eq!(cpu.register_x, 10);
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9,0xc0,0xaa,0xe8,0x00]);

        assert_eq!(cpu.register_x, 0xc1);
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.register_x = 0xff;
        cpu.interpret(vec![0xe8,0xe8,0x00]);

        assert_eq!(cpu.register_x, 1);
    }
}
