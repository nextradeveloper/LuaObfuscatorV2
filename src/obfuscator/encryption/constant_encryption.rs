use rand::{Rng, thread_rng};
use regex::Regex;

fn encrypt_strings(input: &mut String) {
    let mut rng = thread_rng();
    
    // Find all string literals (both single and double quoted)
    let string_regex = Regex::new(r#"("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*')"#).unwrap();
    
    // Generate a random key for XOR encryption (between 1 and 255)
    let xor_key: u8 = rng.gen_range(1..=255);
    
    // Create encrypted version with XOR + Base64-like encoding
    let encrypted_content = string_regex.replace_all(input, |caps: &regex::Captures| {
        let original = &caps[1];
        
        // Skip very short strings that might be operators or special characters
        if original.len() <= 3 {
            return original.to_string();
        }
        
        // Remove quotes to get the actual string content
        let quote_char = original.chars().nth(0).unwrap();
        let content = &original[1..original.len()-1];
        
        // Apply XOR encryption to each byte
        let encrypted_bytes: Vec<u8> = content.bytes()
            .map(|b| b ^ xor_key)
            .collect();
        
        // Convert to a custom base32-like encoding to avoid issues with special characters
        let encoded = base32_encode(&encrypted_bytes);
        
        // Generate a decryption function for this key
        let func_name = format!("_decrypt_{}", xor_key);
        
        // Return a function call that will decrypt the string at runtime
        format!("{}({}{}{})", func_name, quote_char, encoded, quote_char)
    });
    
    // Add the decryption functions at the beginning of the file
    let decryption_code = format!(
        r#"-- String decryption functions
local function base32_decode(input)
    local alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567"
    local result = {{}}
    local buffer = 0
    local bitsLeft = 0
    
    for i = 1, #input do
        local char = string.sub(input, i, i)
        local pos = string.find(alphabet, char)
        if pos then
            local value = pos - 1
            buffer = (buffer * 32) + value
            bitsLeft = bitsLeft + 5
            
            if bitsLeft >= 8 then
                result[#result + 1] = string.char(math.floor(buffer / (2 ^ (bitsLeft - 8))))
                buffer = buffer % (2 ^ (bitsLeft - 8))
                bitsLeft = bitsLeft - 8
            end
        end
    end
    
    return table.concat(result)
end

-- Simple XOR function for Lua 5.1 compatibility
local function xor_byte(a, b)
    local result = 0
    local bitval = 1
    while a > 0 or b > 0 do
        local aa, bb = a % 2, b % 2
        if aa ~= bb then result = result + bitval end
        bitval = bitval * 2
        a = math.floor(a / 2)
        b = math.floor(b / 2)
    end
    return result
end

local function _decrypt_{}(encoded)
    local decoded = base32_decode(encoded)
    local result = {{}}
    for i = 1, #decoded do
        result[i] = string.char(xor_byte(string.byte(decoded, i), {}))
    end
    return table.concat(result)
end

"#, xor_key, xor_key);
    
    *input = decryption_code + &encrypted_content;
}

// Custom base32 encoding function
fn base32_encode(data: &[u8]) -> String {
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut result = String::new();
    let mut buffer = 0u64;
    let mut bits_left = 0;
    
    for &byte in data {
        buffer = (buffer << 8) | (byte as u64);
        bits_left += 8;
        
        while bits_left >= 5 {
            let index = ((buffer >> (bits_left - 5)) & 0x1F) as usize;
            result.push(alphabet.chars().nth(index).unwrap());
            bits_left -= 5;
        }
    }
    
    if bits_left > 0 {
        let index = ((buffer << (5 - bits_left)) & 0x1F) as usize;
        result.push(alphabet.chars().nth(index).unwrap());
    }
    
    result
}

pub fn encrypt(input: &mut String) {
    encrypt_strings(input);
}
