use anchor_lang::prelude::*;
use crate::state::*;
use core_ds::account::MaxSize;

#[account]
pub struct Config {
    pub authority: Pubkey,
    pub components: StructuresComponentKeys,
}

impl MaxSize for Config {
    fn get_max_size() -> u64 {
        return 32+StructuresComponentKeys::get_max_size();
    }
}

#[account]
pub struct StructureIndex {
    pub instance: u64,
    pub portal: Vec<u64>,
    pub healer: Vec<u64>,
    pub lootable: Vec<u64>,
    pub meteor: Vec<u64>
}

impl MaxSize for StructureIndex {
    fn get_max_size() -> u64 {
        return 8+4+4+4+4;
    }
}