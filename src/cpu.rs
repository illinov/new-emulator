pub (crate) struct CPU {
    pub register_a: u8, // Acumulador: Para operaciones aritmeticas
    pub status: u8, // Registro de 8 bits donde cada bit es una flag
    pub program_counter: u16, // 2 bytes para poder direccionar las 65535 direcciones de memoria
    pub register_x: u8, // Index Register X: Para loops o indices
    pub register_y: u8,
    memory: [u8; 0xFFFF], // Memoria del juego. von Neumann.  
}

// Como cada direccion puede guardar un byte. Aqui se usa little endian, donde el byte menos significativo(la parte baja), se almacena primero.
// Si quisieramos por ejemplo leer la direccion de memoria del punto de entrada de la direccion 0xFFFC, 
// Sabiendo que las direcciones de memoria ocupan 2 bytes. Suponiendo que la direccion que buscamos es: 0x1234
// Debemos tomar la parte 0x34 de la direccion 0xFFFC y la parte 0x12 de 0xFFFD, para despues unirlas.
// Hacemos esto leyendo ambas partes en un espacio de 2 bytes, para poder desplazar la parte alta 0x12 a la izquierda y con un 
// simple 'or', quedarnos con los bytes prendidos de 0x34 en la parte derecha. Regresando asi el numero completo.
//
// Dirección real: 0x1234
// En memoria (little endian):
// [0x34] [0x12]
//   ↑      ↑
//  bajo   alto
//
// mem_read_u16
//
// hi = 0x12 = 0b00000000_00010010
//
// hi << 8   = 0b00010010_00000000  = 0x1200
//                        ^^^^^^^^
//                      8 posiciones vacías
//
// lo = 0x34 = 0b00000000_00110100
//
// (hi << 8) | lo = 0b00010010_00000000
//                | 0b00000000_00110100
//                = 0b00010010_00110100 = 0x1234
trait Mem {
    fn mem_read(&self, addr: u16) -> u8;

    fn mem_write(&mut self, addr: u16, data: u8);

    fn mem_read_u16(&self, pos:u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }
}

impl Mem for CPU {
    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }
}

impl CPU {
    pub fn new() -> Self {
        Self {
            register_a: 0,
            status: 0,
            program_counter: 0,
            register_x: 0,
            register_y: 0,
            memory: [0; 0xFFFF],
        }
    }
    
    // "Reinicia la consola" e inicializa el PC en el punto de inicio del codigo del juego
    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.status = 0;
        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    // Toma el codigo del juego, lo mete en la memoria RGP en la seccion ROM y guarda en la
    // direccion 0xFFFC la direccion del punto de entrada del codigo del juego
    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000 .. (0x8000 + program.len())] // Reservar la seccion de memoria que ocupa el codigo del juego
            .copy_from_slice(&program[..]); // Copiar el codigo juego en la memoria del juego reservada (es von Neumann)
        // 
        self.mem_write_u16(0xFFFC,0x8000); // Guardar la referencia a la direccion del entry point del codigo del juego
    }

    // ASM INSTRUCTIONS
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

    pub fn run(&mut self) {
        loop {
            let opscode = self.mem_read(self.program_counter);
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
        cpu.load_and_run(vec![0xa9,0x05,0x00]);
        assert_eq!(cpu.register_a, 0x05); // Se carga el valor correctamente
        assert!(cpu.status & 0b0000_0010 == 0b00); // No hay flag Z
        assert!(cpu.status & 0b1000_0000 == 0); // No hay Flag N
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9,0x00,0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10); // Si se activa la Z
    }

    #[test]
    fn test_0xa9_lda_flag_negative() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9,0x80,0x00]);
        assert!(cpu.status & 0b1000_0000 == 0b1000_0000); // Se activa N
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.register_a = 10;
        cpu.load_and_run(vec![0xaa,0x00]);

        assert_eq!(cpu.register_x, 10);
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9,0xc0,0xaa,0xe8,0x00]);

        assert_eq!(cpu.register_x, 0xc1);
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.register_x = 0xff;
        cpu.load_and_run(vec![0xe8,0xe8,0x00]);

        assert_eq!(cpu.register_x, 1);
    }
}
