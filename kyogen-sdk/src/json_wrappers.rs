use serde::Deserialize;


#[derive(Deserialize, Debug)]
pub struct GameConfigJson {
    pub max_players: u16,
    pub game_token: String,
    pub spawn_claim_multiplier: f64,
}