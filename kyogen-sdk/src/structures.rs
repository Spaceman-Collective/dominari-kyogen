use wasm_bindgen::prelude::*;
use anchor_lang::prelude::*;

#[wasm_bindgen]
#[derive(Default)]
pub struct Structures {
    pub core_id: Pubkey,
    pub registry_id: Pubkey,
    pub kyogen_id: Pubkey,
    pub structures: Pubkey,
    pub payer: Pubkey
}


// Instructions
#[wasm_bindgen]
impl Structures {
    
}