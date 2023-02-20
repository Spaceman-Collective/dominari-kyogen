use anchor_lang::prelude::*;

pub const SEEDS_KYOGENSIGNER:&[u8;13] = b"kyogen_signer";
pub const SEEDS_PACK:&[u8;4] = b"pack";
pub const SEEDS_BLUEPRINT:&[u8;9] = b"blueprint";
pub const SEEDS_INSTANCEINDEX:&[u8;14] = b"instance_index";

pub const ENTITY_ID_SIZE: usize = 8; //u64
pub const STRING_MAX_SIZE: u64 = 128;
pub const PLAYER_MAX_CARDS: u64 = 10;


#[cfg_attr(feature = "sdk", derive(serde::Serialize, serde::Deserialize))]
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum Clans {
    Ancients,
    Wildings,
    Creepers,
    Synths
}

pub const STARTING_CARDS_ANCIENTS_NAME:&str = "starting_cards_ancients";
pub const STARTING_CARDS_WILDINGS_NAME:&str = "starting_cards_wildings";
pub const STARTING_CARDS_CREEPERS_NAME:&str = "starting_cards_creepers";
pub const STARTING_CARDS_SYNTHS_NAME:&str = "starting_cards_synths";