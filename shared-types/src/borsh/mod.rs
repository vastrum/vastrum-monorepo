use borsh::{BorshDeserialize, BorshSerialize};

pub trait BorshExt: BorshSerialize + BorshDeserialize {
    fn encode(&self) -> Vec<u8> {
        return borsh::to_vec(self).unwrap();
    }

    fn decode(bytes: &[u8]) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        return borsh::from_slice(bytes);
    }
}
impl<T: BorshSerialize + BorshDeserialize> BorshExt for T {}
