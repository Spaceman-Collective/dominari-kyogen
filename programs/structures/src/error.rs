use anchor_lang::prelude::*;

#[error_code]
pub enum StructureError {
    #[msg("")]
    LocationMismatch,

    #[msg("")]
    StructureCannotBeInitializedOnSpawn,

    #[msg("")]
    InvalidUnit,

    #[msg("")]
    InvalidLocation,

    #[msg("")]
    InvalidOwner,

    #[msg("")]
    StructureInCooldown,

    #[msg("")]
    WrongStructure,
}