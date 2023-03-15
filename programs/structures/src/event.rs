use anchor_lang::prelude::*;


#[event]
pub struct MeteorMined {
    pub instance: u64, 
    pub tile: u64,
    pub meteor: u64,
    pub player: u64,
}

#[event]
pub struct PortalUsed{
    pub instance: u64,
    pub from: u64,
    pub to: u64,
    pub unit: u64
}

#[event]
pub struct LootableLooted{
    pub instance: u64,
    pub tile: u64,
    pub lootable: u64,
    pub player: u64,
}

#[event]
pub struct GameFinished{
    pub instance:u64,
    pub winning_player_id:u64,
    pub winning_player_key:Pubkey,
    pub high_score:u64,
}