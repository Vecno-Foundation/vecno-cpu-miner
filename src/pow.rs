pub use crate::pow::hasher::HeaderHasher;
use crate::{
    pow::{
        hasher::{Hasher, PowHash},
        mem_hash::mem_hash,
    },
    proto::{RpcBlock, RpcBlockHeader},
    target::{self, Uint256},
    Error,
};
use std::error::Error as StdError;
use std::fmt;

mod hasher;
mod mem_hash;

#[derive(Clone)]
pub struct State {
    #[allow(dead_code)]
    pub id: usize,
    pub nonce: u64,
    target: Uint256,
    block: RpcBlock,
    // PRE_POW_HASH || TIME || 32 zero byte padding; without NONCE
    hasher: PowHash,
    timestamp: u64,
}

impl State {
    #[inline]
    pub fn new(id: usize, block: RpcBlock) -> Result<Self, Error> {
        let header = block.header.as_ref().ok_or("Header is missing")?;
        let target = target::u256_from_compact_target(header.bits);
        let timestamp = header.timestamp as u64;
        let mut hasher = HeaderHasher::new();
        serialize_header(&mut hasher, header, true);
        let pre_pow_hash = hasher.finalize();
        let hasher = PowHash::new(pre_pow_hash, timestamp);

        Ok(Self {
            id,
            nonce: 0,
            target,
            block,
            hasher,
            timestamp,
        })
    }

    #[inline(always)]
    /// PRE_POW_HASH || TIME || 32 zero byte padding || NONCE
    pub fn calculate_pow(&self, nonce: u64) -> Uint256 {
        // Hasher already contains PRE_POW_HASH || TIME || 32 zero byte padding; only NONCE is missing
        let block_hash = self.hasher.clone().finalize_with_nonce(nonce);
        let hash = mem_hash(block_hash, self.timestamp, nonce);
        Uint256::from_le_bytes(hash.as_bytes())
    }

    #[inline(always)]
    pub fn check_pow(&self, nonce: u64) -> bool {
        let pow = self.calculate_pow(nonce);
        pow <= self.target
    }

    #[inline(always)]
    pub fn generate_block_if_pow(&mut self) -> Option<RpcBlock> {
        self.check_pow(self.nonce).then(|| {
            let mut block = self.block.clone();
            let header = block.header.as_mut().expect("Header exists on creation");
            header.nonce = self.nonce;
            block
        })
    }
}

#[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
compile_error!("Supporting only 32/64 bits");

#[inline(always)]
pub fn serialize_header<H: Hasher>(hasher: &mut H, header: &RpcBlockHeader, for_pre_pow: bool) {
    let (nonce, timestamp) = if for_pre_pow { (0, 0) } else { (header.nonce, header.timestamp) };
    let num_parents = header.parents.len();
    let version: u16 = header.version.try_into().unwrap();
    hasher.update(version.to_le_bytes()).update((num_parents as u64).to_le_bytes());

    let mut hash = [0u8; 32];
    for parent in &header.parents {
        hasher.update((parent.parent_hashes.len() as u64).to_le_bytes());
        for hash_string in &parent.parent_hashes {
            decode_to_slice(hash_string, &mut hash).unwrap();
            hasher.update(hash);
        }
    }
    decode_to_slice(&header.hash_merkle_root, &mut hash).unwrap();
    hasher.update(hash);

    decode_to_slice(&header.accepted_id_merkle_root, &mut hash).unwrap();
    hasher.update(hash);
    decode_to_slice(&header.utxo_commitment, &mut hash).unwrap();
    hasher.update(hash);

    hasher
        .update(timestamp.to_le_bytes())
        .update(header.bits.to_le_bytes())
        .update(nonce.to_le_bytes())
        .update(header.daa_score.to_le_bytes())
        .update(header.blue_score.to_le_bytes());

    let blue_work_len = header.blue_work.len().div_ceil(2);
    if header.blue_work.len() % 2 == 0 {
        decode_to_slice(&header.blue_work, &mut hash[..blue_work_len]).unwrap();
    } else {
        let mut blue_work = String::with_capacity(header.blue_work.len() + 1);
        blue_work.push('0');
        blue_work.push_str(&header.blue_work);
        decode_to_slice(&blue_work, &mut hash[..blue_work_len]).unwrap();
    }

    hasher.update((blue_work_len as u64).to_le_bytes()).update(&hash[..blue_work_len]);

    decode_to_slice(&header.pruning_point, &mut hash).unwrap();
    hasher.update(hash);
}

#[derive(Debug)]
enum FromHexError {
    OddLength,
    InvalidStringLength,
    InvalidHexCharacter { c: char, index: usize },
}

impl fmt::Display for FromHexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FromHexError::OddLength => write!(f, "hex string has odd length"),
            FromHexError::InvalidStringLength => write!(f, "hex string length does not match output buffer"),
            FromHexError::InvalidHexCharacter { c, index } => {
                write!(f, "invalid hex character '{}' at index {}", c, index)
            }
        }
    }
}

impl StdError for FromHexError {}

#[inline(always)]
fn decode_to_slice<T: AsRef<[u8]>>(data: T, out: &mut [u8]) -> Result<(), FromHexError> {
    let data = data.as_ref();
    if data.len() % 2 != 0 {
        return Err(FromHexError::OddLength);
    }
    if data.len() / 2 != out.len() {
        return Err(FromHexError::InvalidStringLength);
    }

    for (i, byte) in out.iter_mut().enumerate() {
        *byte = val(data[2 * i], 2 * i)? << 4 | val(data[2 * i + 1], 2 * i + 1)?;
    }

    #[inline(always)]
    fn val(c: u8, idx: usize) -> Result<u8, FromHexError> {
        match c {
            b'A'..=b'F' => Ok(c - b'A' + 10),
            b'a'..=b'f' => Ok(c - b'a' + 10),
            b'0'..=b'9' => Ok(c - b'0'),
            _ => Err(FromHexError::InvalidHexCharacter { c: c as char, index: idx }),
        }
    }

    Ok(())
}