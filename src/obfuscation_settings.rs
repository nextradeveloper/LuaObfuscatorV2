#[derive(Clone)]
pub struct ObfuscationSettings {
    pub include_debug_line_info: bool,
    pub compress_bytecode: bool,
    pub encrypt_strings: bool,
}

impl ObfuscationSettings {
    pub fn new() -> Self {
        Self {
            include_debug_line_info: false,
            compress_bytecode: true,   // Re-enable compression 
            encrypt_strings: true,
        }
    }
}
