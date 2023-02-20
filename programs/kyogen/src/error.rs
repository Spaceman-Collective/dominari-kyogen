use anchor_lang::prelude::*;



#[error_code]
pub enum KyogenError {
    #[msg("Player count exceeded")]
    PlayerCountExceeded,

    #[msg("String too long")]
    StringTooLong,

    #[msg("Wrong pack passed in!")]
    WrongPack,
}