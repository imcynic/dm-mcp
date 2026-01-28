//! RUNSUB Encryption Algorithm
//!
//! Implementation of BYOND's RUNSUB encryption/decryption algorithm.
//! Based on reverse-engineering work from BYONDProxyServer.

/// Decrypt data in-place using the RUNSUB algorithm.
///
/// The algorithm uses a running checksum combined with key bit rotation
/// to generate round keys for each byte.
///
/// # Arguments
/// * `data` - The data buffer to decrypt in-place
/// * `key` - The 32-bit encryption key (0 = no encryption)
pub fn runsub_decrypt(data: &mut [u8], key: i32) {
    if key == 0 || data.is_empty() {
        return;
    }

    let mut checksum: i8 = 0;

    // Process all bytes except the last (which is the checksum)
    for i in 0..data.len().saturating_sub(1) {
        // Calculate round key from checksum and shifted key
        let shift_amount = (checksum as u8) % 32;
        let round_key = checksum.wrapping_add((key >> shift_amount) as i8);

        // Decrypt: subtract round key
        data[i] = data[i].wrapping_sub(round_key as u8);

        // Update checksum with decrypted value
        checksum = checksum.wrapping_add(data[i] as i8);
    }
}

/// Encrypt data in-place using the RUNSUB algorithm.
///
/// The final byte of the buffer will be overwritten with the checksum.
///
/// # Arguments
/// * `data` - The data buffer to encrypt in-place (must have space for checksum)
/// * `key` - The 32-bit encryption key
pub fn runsub_encrypt(data: &mut [u8], key: i32) {
    if data.is_empty() {
        return;
    }

    let mut checksum: u8 = 0;

    // Process all bytes except the last (which will be the checksum)
    for i in 0..data.len().saturating_sub(1) {
        // Calculate round key from checksum and shifted key
        let shift_amount = checksum % 32;
        let round_key = (checksum as i8).wrapping_add((key >> shift_amount) as i8) as u8;

        // Update checksum with plaintext value BEFORE encryption
        checksum = checksum.wrapping_add(data[i]);

        // Encrypt: add round key
        data[i] = data[i].wrapping_add(round_key);
    }

    // Write checksum to final byte
    if !data.is_empty() {
        data[data.len() - 1] = checksum;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key: i32 = 0x12345678;
        let original = b"Hello, BYOND!";

        // Create buffer with extra byte for checksum
        let mut data = Vec::with_capacity(original.len() + 1);
        data.extend_from_slice(original);
        data.push(0); // Space for checksum

        // Encrypt
        runsub_encrypt(&mut data, key);

        // Verify data changed
        assert_ne!(&data[..original.len()], original);

        // Decrypt
        runsub_decrypt(&mut data, key);

        // Verify roundtrip (excluding checksum byte)
        assert_eq!(&data[..original.len()], original);
    }

    #[test]
    fn test_zero_key_noop() {
        let mut data = b"test data".to_vec();
        let original = data.clone();

        runsub_decrypt(&mut data, 0);
        assert_eq!(data, original);

        runsub_encrypt(&mut data, 0);
        // Note: encrypt with key=0 still writes checksum
    }

    #[test]
    fn test_empty_data() {
        let mut data: Vec<u8> = vec![];
        runsub_decrypt(&mut data, 12345);
        runsub_encrypt(&mut data, 12345);
        assert!(data.is_empty());
    }
}
