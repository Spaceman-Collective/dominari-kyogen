use anchor_lang::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;


#[derive(Deserialize, Debug)]
pub struct GameConfigJson {
    pub max_players: u16,
    pub game_token: String,
    pub spawn_claim_multiplier: f64,
}