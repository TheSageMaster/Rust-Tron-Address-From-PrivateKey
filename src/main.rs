use ethnum::U256;
use k256::ecdsa::SigningKey;
use sha3::{Digest, Keccak256};
use sha2::Sha256;
use hex;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use bs58;

#[macro_use]
extern crate arrayref;

fn main() -> io::Result<()> {
    let private_keys_hex = read_private_keys_from_file("private_keys.txt")?;

    for key_hex in private_keys_hex {
        let key_bytes = hex::decode(key_hex.trim()).expect("Invalid hex format");
        let private_key = U256::from_be_bytes(*array_ref!(key_bytes, 0, 32));
        let tron_address = generate_tron_address(private_key);
        println!("TRON Address: {}", tron_address);
    }

    Ok(())
}

fn generate_tron_address(private_key: U256) -> String {
    let private_key_bytes = u256_to_bytes_be(private_key);
    // Flip-flop the first and second 32 bits (16 bytes)
    let mut modified_private_key_bytes = [0u8; 32];
    modified_private_key_bytes[..16].copy_from_slice(&private_key_bytes[16..32]);
    modified_private_key_bytes[16..].copy_from_slice(&private_key_bytes[..16]);

    let signing_key = SigningKey::from_bytes(&modified_private_key_bytes.into()).expect("Invalid private key");
    let public_key = signing_key.verifying_key().to_encoded_point(false);

    // Steps 1 to 4: Get public key, apply Keccak-256, and take the last 40 characters
    let hash = Keccak256::digest(&public_key.as_bytes()[1..]);
    let address_bytes = &hash[12..];

    // Steps 5 to 9: Add '41' prefix, apply SHA-256 twice, and take first 8 characters for checksum
    let mut address_with_prefix = vec![0x41];
    address_with_prefix.extend_from_slice(address_bytes);
    let checksum = Sha256::digest(&Sha256::digest(&address_with_prefix));
    let checksum_slice = &checksum[..4];
    address_with_prefix.extend_from_slice(checksum_slice);

    // Step 10: Base58 encoding
    bs58::encode(address_with_prefix).into_string()
}

fn read_private_keys_from_file<P>(filename: P) -> io::Result<Vec<String>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    let buf_reader = io::BufReader::new(file);
    buf_reader.lines().collect()
}

// Convert U256 to a 32-byte array in big-endian format
fn u256_to_bytes_be(value: U256) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    let u128_parts = value.0; // Extract the [u128; 2] array

    // Process each u128 part
    for (i, &part) in u128_parts.iter().enumerate() {
        // Convert each u128 to big-endian bytes
        let part_bytes = part.to_be_bytes();

        // Determine the starting index for copying bytes into the final array
        let start_index = i * 16; // Each u128 has 16 bytes

        // Copy bytes into the appropriate position of the final array
        bytes[start_index..start_index + 16].copy_from_slice(&part_bytes);
    }

    bytes
}