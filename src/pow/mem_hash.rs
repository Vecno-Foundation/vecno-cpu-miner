use crate::pow::hasher::VecnoHash;
use crate::Hash;
use blake3::Hasher as Blake3;

pub struct MemHash {
    sbox: Vec<u8>,
    rounds: usize,
    result: [u32; 8],
    nonce: u64,
}

impl MemHash {
    #[inline(always)]
    pub fn new(input_hash: Hash, timestamp: u64, nonce: u64) -> Self {
        let input_bytes = input_hash.as_bytes();
        
        let sbox = Self::generate_sbox(&input_bytes);
        let rounds = Self::calculate_rounds(&input_bytes, timestamp);
        let result = Self::initialize_result(&input_bytes);

        Self {
            sbox,
            rounds,
            result,
            nonce,
        }
    }

    #[inline(always)]
    fn generate_sbox(input_bytes: &[u8]) -> Vec<u8> {
        let mut sbox = vec![0u8; 64]; 
        let mut hasher = Blake3::new();
        hasher.update(input_bytes);
        let mut seed = hasher.finalize();
        
        // Fill S-box in 32-byte chunks using BLAKE3
        for i in (0..64).step_by(2) {
            let chunk = &mut sbox[i..(i + 2).min(64)];
            chunk.copy_from_slice(&seed.as_bytes()[..chunk.len()]);
            hasher = Blake3::new();
            hasher.update(seed.as_bytes());
            seed = hasher.finalize();
        }
        sbox
    }

    #[inline(always)]
    fn calculate_rounds(input_bytes: &[u8], timestamp: u64) -> usize {
        let mut hasher = Blake3::new();
        hasher.update(input_bytes);
        hasher.update(&timestamp.to_le_bytes());
        let hash = hasher.finalize();
        (u32::from_le_bytes(hash.as_bytes()[0..4].try_into().unwrap()) % 8 + 16) as usize
    }

    #[inline(always)]
    fn initialize_result(input_bytes: &[u8]) -> [u32; 8] {
        let mut result = [0u32; 8];
        for i in 0..8 {
            result[i] = u32::from_le_bytes(input_bytes[i * 4..i * 4 + 4].try_into().unwrap());
        }
        result
    }

    #[inline(always)]
    fn u32_array_to_u8_array(result: [u32; 8]) -> [u8; 32] {
        let mut output = [0u8; 32];
        for (i, &value) in result.iter().enumerate() {
            output[i * 4..i * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        output
    }

    #[inline(always)]
    fn bit_manipulations(data: &mut [u8; 32]) {
        for i in (0..32).step_by(4) {
            data[i] ^= data[i + 1];
            data[i + 2] ^= data[i + 3];
        }
    }

    #[inline(always)]
    fn byte_mixing(b3_hash1: &[u8; 32], b3_hash2: &[u8; 32]) -> [u8; 32] {
        let mut temp_buf = [0u8; 32];
        for i in 0..32 {
            temp_buf[i] = b3_hash1[i] ^ b3_hash2[i];
        }
        temp_buf
    }

    #[inline(always)]
    pub fn compute_hash(&mut self) -> Hash {
        let operations: [fn(u32, u32) -> u32; 4] = [
            |a, b| a.wrapping_add(b),
            |a, b| a.wrapping_sub(b),
            |a, b| a.rotate_left(b & 0x1F),
            |a, b| a ^ b,
        ];

        // Initialize hash_bytes from result
        let mut hash_bytes = Self::u32_array_to_u8_array(self.result);

        // First loop: BLAKE3 hashing with bit manipulations
        for _ in 0..self.rounds {
            let mut hasher = Blake3::new();
            hasher.update(&hash_bytes);
            hash_bytes = *hasher.finalize().as_bytes();
            Self::bit_manipulations(&mut hash_bytes);
        }

        // Second loop: BLAKE3 hashing with bit manipulations
        for _ in 0..self.rounds {
            let mut hasher = Blake3::new();
            hasher.update(&hash_bytes);
            hash_bytes = *hasher.finalize().as_bytes();
            Self::bit_manipulations(&mut hash_bytes);
        }

        // Update result from hash_bytes
        for i in 0..8 {
            self.result[i] = u32::from_le_bytes(hash_bytes[i * 4..i * 4 + 4].try_into().unwrap());
        }

        // Continue with original S-box and operation logic
        for round in 0..self.rounds {
            for i in 0..8 {
                let mut state_hasher = Blake3::new();
                state_hasher.update(&self.result[i].to_le_bytes());
                state_hasher.update(&round.to_le_bytes());
                state_hasher.update(&self.nonce.to_le_bytes());
                let state_hash = state_hasher.finalize();
                let state_bytes = state_hash.as_bytes();

                let result_bytes = Self::u32_array_to_u8_array(self.result);
                let mixed_bytes = Self::byte_mixing(state_bytes, &result_bytes);
                let v = u32::from_le_bytes(mixed_bytes[0..4].try_into().unwrap());

                let mut v = v ^ self.result[i];

                let branch = (v & 0xFF) as usize % 4;
                v = operations[branch](v, self.result[(i + 1) % 8]);

                let b = v.to_le_bytes();
                // Use input-dependent indices for S-box lookups
                let idx_base = (v as usize) % 64;
                v = u32::from_le_bytes([
                    self.sbox[(idx_base + (b[0] as usize)) % 64],
                    self.sbox[(idx_base + (b[1] as usize)) % 64],
                    self.sbox[(idx_base + (b[2] as usize)) % 64],
                    self.sbox[(idx_base + (b[3] as usize)) % 64],
                ]);

                self.result[i] = v;
            }
        }

        let mut output = Self::u32_array_to_u8_array(self.result);
        Self::bit_manipulations(&mut output);
        VecnoHash::hash(Hash::from_le_bytes(output))
    }
}

#[inline]
pub fn mem_hash(input_hash: Hash, timestamp: u64, nonce: u64) -> Hash {
    let mut mem_hash = MemHash::new(input_hash, timestamp, nonce);
    mem_hash.compute_hash()
}