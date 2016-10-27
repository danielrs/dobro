//! Encryption and Decryption using Blowfish with ECB mode.

use crypto::blowfish::Blowfish;
use crypto::symmetriccipher::{BlockEncryptor, BlockDecryptor};

pub fn encrypt(key: &String, input: &String) -> String {
    let blowfish = Blowfish::new(key.as_bytes());
    let block_size = <Blowfish as BlockEncryptor>::block_size(&blowfish);

    // Input and output bytes
    let input_len = round_len(input.len(), block_size);
    let mut input = input.as_bytes().to_vec();
    input.resize(input_len, 0);

    let mut output : Vec<u8> = Vec::with_capacity(input_len);
    unsafe { output.set_len(input_len); }

    // Encrypts input and saves it into output
    for (ichunk, mut ochunk) in input.chunks(block_size).zip(output.chunks_mut(block_size)) {
        blowfish.encrypt_block(&ichunk, &mut ochunk);
    }

    // Generates hex representation of output
    let mut hex_output = String::with_capacity(output.len() * 2);
    for i in 0..output.len() {
        hex_output.push_str(&format!("{:02x}", output[i]));
    }

    hex_output
}

/// Rounds the given len so that it contains blocks
/// of the same size.
fn round_len(len: usize, block_size: usize) -> usize {
    let remainder = len % block_size;
    if remainder == 0 {
        len
    }
    else {
        len + block_size - remainder
    }
}
