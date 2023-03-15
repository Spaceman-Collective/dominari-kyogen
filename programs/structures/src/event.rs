use anchor_lang::prelude::*;


#[event]
pub struct MeteorMined { 
    pub tile: u64,
    pub meteor: u64,
    pub player: u64,
}

#[event]
pub struct PortalUsed{
    pub from: u64,
    pub to: u64,
    pub unit: u64
}