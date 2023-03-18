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

    #[msg("")]
    TileOccupied,

    #[msg("")]
    UnitCooldown,

    #[msg("")]
    PortalChannelMismatch,

    #[msg("")]
    GamePaused,

    #[msg("")]
    InvalidPack,

    #[msg("")]
    HighScoreNotReached,

    #[msg("")]
    WrongPlayer,

    #[msg("")]
    NotAStructure,
}