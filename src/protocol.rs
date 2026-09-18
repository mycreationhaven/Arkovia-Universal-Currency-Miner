use scrypt::{scrypt, Params};

/// Exact Arkovia/Nxt Monetary System work bytes: five unsigned 64-bit fields in little endian.
pub fn work_bytes(nonce: u64, currency_id: u64, units: u64, counter: u64, account_id: u64) -> [u8; 40] {
    let mut bytes = [0u8; 40];
    for (offset, value) in [(0, nonce), (8, currency_id), (16, units), (24, counter), (32, account_id)] { bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes()); }
    bytes
}

/// Arkovia's Scrypt algorithm ID is 5. It is scrypt N=1024, r=1, p=1, 32-byte output.
pub fn scrypt_hash(input: &[u8; 40]) -> [u8; 32] {
    let params = Params::new(10, 1, 1, 32).expect("static scrypt parameters are valid");
    let mut output = [0u8; 32];
    scrypt(input, input, &params, &mut output).expect("output size matches params");
    output
}

/// Target bytes are interpreted in the little-endian order used by CurrencyMinting.meetsTarget.
pub fn meets_target(hash: &[u8; 32], target: &[u8; 32]) -> bool {
    for index in (0..32).rev() { if hash[index] > target[index] { return false; } if hash[index] < target[index] { return true; } }
    true
}

#[cfg(test)]
mod tests { use super::*;
    #[test] fn work_is_little_endian() { let work = work_bytes(0x0102, 2, 3, 4, 5); assert_eq!(&work[..2], &[2, 1]); assert_eq!(u64::from_le_bytes(work[8..16].try_into().unwrap()), 2); }
    #[test] fn target_comparison_matches_reference_order() { let mut hash=[0;32]; let target=[0;32]; assert!(meets_target(&hash,&target)); hash[31]=1; assert!(!meets_target(&hash,&target)); }
}
