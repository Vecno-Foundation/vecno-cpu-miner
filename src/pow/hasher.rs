use crate::Hash;
use blake3::Hasher as Blake3State;

const BLOCK_HASH_DOMAIN: &[u8; 32] = b"BlockHash\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";

#[derive(Clone)]
pub(super) struct PowHash(Blake3State);

#[derive(Clone)]
pub(super) struct VecnoHash;

#[derive(Clone)]
pub struct HeaderHasher(Blake3State);

impl PowHash {
    #[inline]
    pub(super) fn new(pre_pow_hash: Hash, timestamp: u64) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&pre_pow_hash.to_le_bytes());
        hasher.update(&timestamp.to_le_bytes());
        hasher.update(&[0u8; 32]);
        Self(hasher)
    }

    #[inline(always)]
    pub(super) fn finalize_with_nonce(&mut self, nonce: u64) -> Hash {
        let mut hasher = self.0.clone(); // Clone the internal blake3::Hasher
        hasher.update(&nonce.to_le_bytes());
        let mut hash_bytes = [0u8; 32];
        hasher.finalize_xof().fill(&mut hash_bytes);
        let mut words = [0u64; 4];
        for (i, chunk) in hash_bytes.chunks(8).enumerate() {
            words[i] = u64::from_le_bytes(chunk.try_into().unwrap());
        }
        Hash::new(words)
    }
}

impl VecnoHash {
    #[inline(always)]
    pub(super) fn hash(in_hash: Hash) -> Hash {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&in_hash.to_le_bytes());
        let mut hash_bytes = [0u8; 32];
        hasher.finalize_xof().fill(&mut hash_bytes);
        let mut words = [0u64; 4];
        for (i, chunk) in hash_bytes.chunks(8).enumerate() {
            words[i] = u64::from_le_bytes(chunk.try_into().unwrap());
        }
        Hash::new(words)
    }
}

impl HeaderHasher {
    #[inline(always)]
    pub fn new() -> Self {
        Self(blake3::Hasher::new_keyed(BLOCK_HASH_DOMAIN))
    }

    pub fn write<A: AsRef<[u8]>>(&mut self, data: A) {
        self.0.update(data.as_ref());
    }

    #[inline(always)]
    pub fn finalize(self) -> Hash {
        let hash = self.0.finalize();
        let bytes = hash.as_bytes();
        let mut words = [0u64; 4];
        for (i, chunk) in bytes.chunks(8).enumerate() {
            words[i] = u64::from_le_bytes(chunk.try_into().unwrap());
        }
        Hash::new(words)
    }
}

pub trait Hasher {
    fn update<A: AsRef<[u8]>>(&mut self, data: A) -> &mut Self;
}

impl Hasher for HeaderHasher {
    fn update<A: AsRef<[u8]>>(&mut self, data: A) -> &mut Self {
        self.write(data);
        self
    }
}
