//! Encryption and Decryption using Blowfish with ECB mode.

use std::ffi::OsString;

use crypto::blowfish::Blowfish;
use crypto::symmetriccipher::{BlockEncryptor, BlockDecryptor};

const PADDING_BYTE: u8 = 2;

/// Returns the encrypted input using the given key.
///
/// The returned string is encoded in hexadecimal notation,
/// which is a UTF-8 string, so it's fine to return it using
/// the `String` type.
pub fn encrypt(key: &str, input: &str) -> String {
    let cipherbytes = cipher_with(key.as_bytes(), input.as_bytes(), |blowfish, from, mut to| {
        blowfish.encrypt_block(from, to);
    });

    // Generate hexadecimal representation of `cipherbytes`.
    let mut output = String::with_capacity(cipherbytes.len() * 2);
    for b in cipherbytes {
        output.push_str(&format!("{:02x}", b));
    }
    output
}

/// Returns the decrypted input using the given key.
///
/// Because Strings must be UTF-8 compilant, and decrypting
/// doesn't guarantees an UTF-8 string, we return
/// a OsString which doesn't have to be UTF-8 compilant.
pub fn decrypt(key: &str, hex_input: &str) -> OsString {
    use std::u8;
    use std::str;
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    // Gets bytes from hexadecimal representation.
    let mut input = Vec::with_capacity(hex_input.len());
    for chunk in hex_input.as_bytes().chunks(2) {
        // `chunk` is utf-8 since it is comming from &str.
        let fragment = unsafe { str::from_utf8_unchecked(chunk) };
        let byte = u8::from_str_radix(fragment, 16).unwrap_or(0);
        input.push(byte);
    }

    let mut cipherbytes = cipher_with(key.as_bytes(), &input, |blowfish, from, mut to| {
        blowfish.decrypt_block(from, to);
    });

    // Ignore up to `PADDING_BYTE`.
    if let Some(index) = cipherbytes.iter().position(|&b| b == PADDING_BYTE) {
        cipherbytes.truncate(index);
    }

    OsStr::from_bytes(&cipherbytes).to_owned()
}

/// Divides the input in blocks and ciphers it using the given closure.
fn cipher_with<F>(key: &[u8], input: &[u8], func: F) -> Vec<u8>
    where F: Fn(&Blowfish, &[u8], &mut [u8]) {

    let blowfish = Blowfish::new(key);
    let block_size = <Blowfish as BlockEncryptor>::block_size(&blowfish);

    // Input and output bytes
    let input_len = round_len(input.len(), block_size);
    let mut input = input.to_vec();
    input.resize(input_len, PADDING_BYTE);

    let mut output : Vec<u8> = Vec::with_capacity(input_len);
    unsafe { output.set_len(input_len); }

    // Encrypts input and into output
    for (ichunk, mut ochunk) in input.chunks(block_size).zip(output.chunks_mut(block_size)) {
        func(&blowfish, ichunk, ochunk);
    }

    output
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

#[cfg(test)]
mod tests {
    use super::{encrypt, decrypt};
    use std::ffi::OsString;

    struct Test {
        key: String,
        plain_text: String,
        cipher_text: String,
    }

    fn get_test_vector() -> Vec<Test> {
        vec![
            Test {
                key: "R=U!LH$O2B#".to_owned(),
                plain_text: "è.<Ú1477631903".to_owned(),
                cipher_text: "4a6b45612b018614c92c50dc73462bbd".to_owned(),
            },
        ]
    }

    #[test]
    fn encrypt_test_vector() {
        for test in get_test_vector() {
            let cipher_text = encrypt(&test.key, &test.plain_text);
            assert_eq!(test.cipher_text, cipher_text);
        }
    }
}
