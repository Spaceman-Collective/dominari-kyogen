use anchor_lang::prelude::*;
use core_ds::account::MaxSize;

#[cfg_attr(feature = "sdk", derive(serde::Serialize, serde::Deserialize))]
#[derive(AnchorDeserialize, AnchorSerialize, Debug, Clone)]
pub struct StructuresComponentKeys {
    pub structure: Pubkey
}

impl MaxSize for StructuresComponentKeys {
    fn get_max_size() -> u64 {
        return 32*1;
    }
}

