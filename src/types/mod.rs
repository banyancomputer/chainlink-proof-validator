use cid::Cid;
use ethers::prelude::Address;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sled::IVec;
use std::ops::{Add, Mul, Sub};

pub fn serialize_cid<S: Serializer>(cid: &Cid, s: S) -> Result<S::Ok, S::Error> {
    let cid_bytes = cid.to_bytes();
    s.serialize_bytes(&cid_bytes)
}

// fn<'de, D>(D) -> Result<T, D::Error> where D: Deserializer<'de>
pub fn deserialize_cid<'de, D>(deserializer: D) -> Result<Cid, D::Error>
where
    D: Deserializer<'de>,
{
    let cid_bytes = <&[u8]>::deserialize(deserializer)?;
    Cid::read_bytes(cid_bytes).map_err(|e| Error::custom(e.to_string()))
}

pub fn serialize_hash<S: Serializer>(hash: &bao::Hash, s: S) -> Result<S::Ok, S::Error> {
    let hash_bytes = hash.as_bytes();
    s.serialize_bytes(hash_bytes)
}

pub fn deserialize_hash<'de, D>(deserializer: D) -> Result<bao::Hash, D::Error>
where
    D: Deserializer<'de>,
{
    let hash_bytes = <[u8; 32]>::deserialize(deserializer)?;
    Ok(bao::Hash::from(hash_bytes))
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct DealID(pub u64);

#[allow(clippy::from_over_into)]
impl Into<IVec> for DealID {
    fn into(self) -> IVec {
        IVec::from(&self.0.to_le_bytes())
    }
}

impl From<IVec> for DealID {
    fn from(iv: IVec) -> Self {
        let bytes = iv.as_ref();
        let mut deal_id_bytes = [0u8; 8];
        deal_id_bytes.copy_from_slice(&bytes[..8]);
        DealID(u64::from_le_bytes(deal_id_bytes))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct BlockNum(pub u64);

impl Add for BlockNum {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        BlockNum(self.0 + other.0)
    }
}

impl Sub for BlockNum {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        BlockNum(self.0 - other.0)
    }
}

impl Mul<u64> for BlockNum {
    type Output = Self;
    fn mul(self, other: u64) -> Self {
        BlockNum(self.0 * other)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct TokenAmount(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token(pub Address);

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct OnChainDealInfo {
    pub deal_id: DealID,
    pub deal_start_block: BlockNum,
    pub deal_length_in_blocks: BlockNum,
    pub proof_frequency_in_blocks: BlockNum,
    pub price: TokenAmount,
    pub collateral: TokenAmount,
    pub erc20_token_denomination: Token,
    #[serde(serialize_with = "serialize_cid", deserialize_with = "deserialize_cid")]
    pub ipfs_file_cid: Cid,
    pub file_size: u64,
    #[serde(
        serialize_with = "serialize_hash",
        deserialize_with = "deserialize_hash"
    )]
    pub blake3_checksum: bao::Hash,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum DealStatus {
    Future,
    Active,
    CompleteAwaitingFinalization,
    Cancelled,
    Done,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocalDealInfo {
    pub onchain: OnChainDealInfo,
    #[serde(serialize_with = "serialize_cid", deserialize_with = "deserialize_cid")]
    pub obao_cid: Cid,
    pub last_submission: BlockNum,
    pub status: DealStatus,
}

pub struct Proof {
    pub block_number: BlockNum,
    pub bao_proof_data: Vec<u8>,
}

/// TODO lmao this is so lame, please improve it
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ProofBuddyError {
    FatalPanic(String),
    InformWebserver(String),
    NonFatal(String),
}