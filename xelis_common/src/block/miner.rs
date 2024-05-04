use std::borrow::Cow;

use log::debug;

use crate::{
    crypto::{
        elgamal::RISTRETTO_COMPRESSED_SIZE,
        pow_hash_with_scratch_pad,
        Hash,
        Hashable,
        Input,
        PublicKey,
        ScratchPad,
        XelisHashError
    },
    serializer::{Reader, ReaderError, Serializer, Writer},
    time::TimestampMillis,
};

use super::{BlockHeader, BLOCK_WORK_SIZE, EXTRA_NONCE_SIZE};

// This structure is used by xelis-miner which allow to compute a valid block POW hash
#[derive(Clone, Debug)]
pub struct MinerWork<'a> {
    header_work_hash: Hash, // include merkle tree of tips, txs, and height (immutable)
    timestamp: TimestampMillis, // miners can update timestamp to keep it up-to-date
    nonce: u64,
    miner: Option<Cow<'a, PublicKey>>,
    // Extra nonce so miner can write anything
    // Can also be used to spread more the work job and increase its work capacity
    extra_nonce: [u8; EXTRA_NONCE_SIZE],
    // Cache in case of hashing
    cache: Option<Input>
}

impl<'a> MinerWork<'a> {
    pub fn new(header_work_hash: Hash, timestamp: TimestampMillis) -> Self {
        Self {
            header_work_hash,
            timestamp,
            nonce: 0,
            miner: None,
            extra_nonce: [0u8; EXTRA_NONCE_SIZE],
            cache: None
        }
    }

    pub fn print_values(&self) {
        println!("header_work_hash: {:?}", self.header_work_hash);
        println!("timestamp: {:?}", self.timestamp);
        println!("nonce: {:?}", self.nonce);
        println!("miner: {:?}", self.miner);
        println!("extra_nonce: {:?}", self.extra_nonce);
    }
    
    pub fn from_block(header: BlockHeader) -> Self {
        Self {
            header_work_hash: header.get_work_hash(),
            timestamp: header.get_timestamp(),
            nonce: 0,
            miner: Some(Cow::Owned(header.miner)),
            extra_nonce: header.extra_nonce,
            cache: None
        }
    }

    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    pub fn get_header_work_hash(&self) -> &Hash {
        &self.header_work_hash
    }

    pub fn get_miner(&self) -> Option<&PublicKey> {
        self.miner.as_ref().map(|m| m.as_ref())
    }

    #[inline(always)]
    pub fn get_pow_hash(&mut self, scratch_pad: &mut ScratchPad) -> Result<Hash, XelisHashError> {
        if self.cache.is_none() {
            let mut input = Input::default();
            input.as_mut_slice()?[0..BLOCK_WORK_SIZE].copy_from_slice(&self.to_bytes());
            self.cache = Some(input);
        }

        let mut bytes = self.cache.as_mut().unwrap().as_mut_slice()?.clone();
        pow_hash_with_scratch_pad(&mut bytes, scratch_pad)
    }

    pub fn get_extra_nonce(&mut self) -> &mut [u8; EXTRA_NONCE_SIZE] {
        &mut self.extra_nonce
    }

    #[inline(always)]
    pub fn set_timestamp(&mut self, timestamp: TimestampMillis) -> Result<(), XelisHashError> {
        self.timestamp = timestamp;
        if let Some(cache) = &mut self.cache {
            cache.as_mut_slice()?[32..40].copy_from_slice(&self.timestamp.to_be_bytes());
        }

        Ok(())
    }

    #[inline(always)]
    pub fn increase_nonce(&mut self) -> Result<(), XelisHashError> {
        self.nonce += 1;
        if let Some(cache) = &mut self.cache {
            cache.as_mut_slice()?[40..48].copy_from_slice(&self.nonce.to_be_bytes());
        }
        Ok(())
    }

    #[inline(always)]
    pub fn set_miner(&mut self, miner: Cow<'a, PublicKey>) {
        self.miner = Some(miner);
    }

    #[inline(always)]
    pub fn set_thread_id(&mut self, id: u8) {
        self.extra_nonce[EXTRA_NONCE_SIZE - 1] = id;
    }

    #[inline(always)]
    pub fn set_thread_id_u16(&mut self, id: u16) {
        self.extra_nonce[EXTRA_NONCE_SIZE - 2..].copy_from_slice(id.to_be_bytes().as_ref());
    }

    #[inline(always)]
    pub fn take(self) -> (Hash, TimestampMillis, u64, Option<Cow<'a, PublicKey>>, [u8; EXTRA_NONCE_SIZE]) {
        (self.header_work_hash, self.timestamp, self.nonce, self.miner, self.extra_nonce)
    }
}

impl<'a> Serializer for MinerWork<'a> {
    fn write(&self, writer: &mut Writer) {
        writer.write_hash(&self.header_work_hash); // 32
        writer.write_u64(&self.timestamp); // 32 + 8 = 40
        writer.write_u64(&self.nonce); // 40 + 8 = 48
        writer.write_bytes(&self.extra_nonce); // 48 + 32 = 80

        // 80 + 32 = 112
        if let Some(miner) = &self.miner {
            miner.write(writer);
        } else {
            // We set a 32 bytes empty public key as we don't have any miner
            writer.write_bytes(&[0u8; RISTRETTO_COMPRESSED_SIZE]);
        }

        debug_assert!(writer.total_write() == BLOCK_WORK_SIZE, "invalid block work size, expected {}, got {}", BLOCK_WORK_SIZE, writer.total_write());
    }

    fn read(reader: &mut Reader) -> Result<MinerWork<'a>, ReaderError> {
        if reader.total_size() != BLOCK_WORK_SIZE {
            debug!("invalid block work size, expected {}, got {}", BLOCK_WORK_SIZE, reader.total_size());
            return Err(ReaderError::InvalidSize)
        }

        let header_work_hash = reader.read_hash()?;
        let timestamp = reader.read_u64()?;
        let nonce = reader.read_u64()?;
        let extra_nonce = reader.read_bytes_32()?;
        let miner = Some(Cow::Owned(PublicKey::read(reader)?));

        Ok(MinerWork {
            header_work_hash,
            timestamp,
            nonce,
            extra_nonce,
            miner,
            cache: None
        })
    }

    fn size(&self) -> usize {
        BLOCK_WORK_SIZE
    }
}

// no need to override hash() as its already serialized in good format
impl Hashable for MinerWork<'_> {}

// #[cfg(test)]
// mod tests {
//     use indexmap::IndexSet;
//     use crate::{crypto::{Hash, Hashable, KeyPair}, serializer::Serializer};
//     use super::MinerWork;
//     #[test]
//     fn test_block_template_from_hex() {
//         let serialized = "00000000000000eab80000018f3662cc0300000000000000002163911437e8860308689873ad09ef32ea1e679d7dfa34ad49fd03bf0597636f01822b17bd3aae2766e83f603458ce155b103d4d766e1fb35c5c349aa0cfe00c530001cb58c67d2aaf2ad04cec0cb8d3296dfaae828c3d0620b9437856e1e4f9bc206d7e40899c7bcc885fad6dd3bdc68fa73141c1d8b917a1f399afeb1fb191376b16".to_owned();
//         let header = MinerWork::from_hex(serialized.clone()).unwrap();
//         print!("Miner {:?}", header.miner);
//         print!("Tips Count {:?}", header.get_tips().len());
//         print!("Tx Count: {:?}", header.get_txs_count());
//         assert!(header.to_hex() == serialized);
//     }
// }