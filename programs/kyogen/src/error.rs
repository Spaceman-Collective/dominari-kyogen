use anchor_lang::prelude::*;



#[error_code]
pub enum KyogenError {
    #[msg("")]
    PlayerCountExceeded, // 6000

    #[msg("")]
    StringTooLong, // 6001

    #[msg("")]
    WrongPack, // 6002

    #[msg("")]
    WrongTile, // 6003

    #[msg("")]
    TileOccupied, // 6004

    #[msg("")]
    TileIsNotEmpty, // 6005

    #[msg("")]
    NoOccupantOnTile, // 6006

    #[msg("")]
    PlayerDoesntOwnUnit, // 6007

    #[msg("")]
    AttackingSelfOwnedUnit, // 6008

    #[msg("")]
    TileIsNotSpawnable, // 6009

    #[msg("")]
    TileAlreadyClaimed, // 6010

    #[msg("")]
    TileOutOfRange, // 6011

    #[msg("")]
    WrongUnit, // 6012

    #[msg("")]
    WrongPlayer, // 6013

    #[msg("")]
    GamePaused, // 6014

    #[msg("")]
    UnitRecovering, // 6015

    #[msg("")]
    UnitNotActive, // 6016

    #[msg("")]
    EntityCannotBeClosedByThisProgram, // 6017

    #[msg("")]
    PlayerCanOnlyBeClosedByOwner, // 6018

    #[msg("")]
    UnitCanOnlyBeClosedByOwner, // 6019

    #[msg("")]
    TileCanOnlyBeClosedByInstanceAuthority, // 6020

    
}