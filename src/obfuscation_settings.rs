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
            compress_bytecode: false,  // Disable compression to avoid decompression infinite loops
            encrypt_strings: true,
        }
    }
}
