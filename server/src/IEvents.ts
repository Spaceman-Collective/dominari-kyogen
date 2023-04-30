export interface AccountChange {
    id: string,
    data: string, //b64 encoded
}

/** Kyogen Events */
// GameStateChanged
export interface EventGameStateChanged {
    instance: string,
    newState: string // PlayPhase (will be lowercased)
}

// NewPlayer
export interface EventNewPlayer {
    instance: string,
    player: AccountChange,
    authority: string // Pubkey
    clan: string // Clan
}

// SpawnClaimed
export interface EventSpawnClaimed{
    instance: string,
    clan: string, //Clans
    tile: AccountChange,
    player: string // player id (no change)
}

// UnitSpawned
export interface EventUnitSpawned {
    instance: string,
    tile: AccountChange, // occupant changed
    player: AccountChange, // card used
    unit: AccountChange // entity created
}

// UnitMoved
export interface EventUnitMoved {
    instance: string,
    unit: AccountChange, //Last Used
    from: AccountChange, // occupant
    to: AccountChange, // occupant
}

// UnitAttacked
export interface EventUnitAttacked{
    instance: string,
    attacker: AccountChange, //last used
    defender: AccountChange, // hp
    tile: AccountChange // occupant if defender hp < 0
}

/** Structures Events */
// Meteor Mined
export interface EventMeteorMined {
    instance: string,
    tile: string, // no change
    meteor: AccountChange, // last used
    player: AccountChange // score
}

// PortalUsed
export interface EventPortalUsed{
    instance: string,
    from: AccountChange, //occupant
    to: AccountChange, //occupant,
    unit: AccountChange, //last used
}

// LootableLooted
export interface EventLootableLooted {
    instance: string,
    tile: string, //no change
    lootable: AccountChange, //last used
    player: AccountChange // card hand
}

// GameFished
export interface EventGameFinished {
    instance: string,
    winning_player_id: string, 
    winning_player_key: string,
    high_score: string
}