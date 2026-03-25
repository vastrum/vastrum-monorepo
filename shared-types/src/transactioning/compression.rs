#[derive(BorshSerialize, BorshDeserialize)]
struct CompressedCalldata {
    decompressed_size: u32,
    compressed: Vec<u8>,
}

pub fn compress_calldata(data: &[u8]) -> Vec<u8> {
    let envelope = CompressedCalldata {
        decompressed_size: data.len() as u32,
        compressed: lz4_flex::compress(data),
    };
    let compressed = envelope.encode();
    return compressed;
}

pub fn decompress_calldata(data: &[u8]) -> Result<Vec<u8>, String> {
    let Ok(envelope) = borsh::from_slice::<CompressedCalldata>(data) else {
        return Err("invalid compressed envelope".into());
    };
    let size = envelope.decompressed_size as usize;
    if size > MAX_DECOMPRESSED_CALLDATA_SIZE {
        return Err(format!(
            "decompressed size {} exceeds limit {}",
            size, MAX_DECOMPRESSED_CALLDATA_SIZE
        ));
    }
    let Ok(decompressed) = lz4_flex::decompress(&envelope.compressed, size) else {
        return Err("lz4 decompress failed".into());
    };
    return Ok(decompressed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let original = b"hello world hello world hello world";
        let compressed = compress_calldata(original);
        let decompressed = decompress_calldata(&compressed).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn large_calldata_round_trip() {
        // 1 MB of repetitive data - compresses well
        let original = vec![0xABu8; 1024 * 1024];
        let compressed = compress_calldata(&original);
        assert!(compressed.len() < original.len(), "should actually compress");
        let decompressed = decompress_calldata(&compressed).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn large_incompressible_round_trip() {
        // 10 MB of pseudo-random data - barely compresses
        let mut original = vec![0u8; 10 * 1024 * 1024];
        for (i, b) in original.iter_mut().enumerate() {
            *b = (i.wrapping_mul(2654435761)) as u8;
        }
        let compressed = compress_calldata(&original);
        let decompressed = decompress_calldata(&compressed).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn rejects_over_limit() {
        // 50 MB repetitive data - compresses small but decompressed_size exceeds 16 MB limit
        let original = vec![0xABu8; 50 * 1024 * 1024];
        let compressed = compress_calldata(&original);
        assert!(decompress_calldata(&compressed).is_err());
    }

    #[test]
    fn rejects_oversized() {
        let fake = CompressedCalldata {
            decompressed_size: (MAX_DECOMPRESSED_CALLDATA_SIZE as u32) + 1,
            compressed: vec![0; 8],
        };
        let encoded = fake.encode();
        assert!(decompress_calldata(&encoded).is_err());
    }

    #[test]
    fn rejects_invalid_envelope() {
        assert!(decompress_calldata(&[1, 2, 3]).is_err());
    }

    #[test]
    fn empty_calldata_round_trip() {
        let compressed = compress_calldata(&[]);
        let decompressed = decompress_calldata(&compressed).unwrap();
        assert!(decompressed.is_empty());
    }
}

use crate::borsh::BorshExt;
use crate::limits::MAX_DECOMPRESSED_CALLDATA_SIZE;
use borsh::{BorshDeserialize, BorshSerialize};
