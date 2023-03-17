use anchor_lang::prelude::*;



#[error_code]
pub enum KyogenError {
    #[msg("")]
    PlayerCountExceeded,

    #[msg("")]
    StringTooLong,

    #[msg("")]
    WrongPack,

    #[msg("")]
    WrongTile,

    #[msg("")]
    TileOccupied,

    #[msg("")]
    TileIsNotEmpty,

    #[msg("")]
    NoOccupantOnTile,

    #[msg("")]
    PlayerDoesntOwnUnit,

    #[msg("")]
    AttackingSelfOwnedUnit,

    #[msg("")]
    TileIsNotSpawnable,

    #[msg("")]
    TileAlreadyClaimed,

    #[msg("")]
    TileOutOfRange,

    #[msg("")]
    WrongUnit,

    #[msg("")]
    WrongPlayer,

    #[msg("")]
    GamePaused,

    #[msg("")]
    UnitRecovering,

    #[msg("")]
    UnitNotActive,

    #[msg("")]
    EntityCannotBeClosedByThisProgram,

    #[msg("")]
    PlayerCanOnlyBeClosedByOwner,

    #[msg("")]
    UnitCanOnlyBeClosedByOwner,

    #[msg("")]
    TileCanOnlyBeClosedByInstanceAuthority,

    
}