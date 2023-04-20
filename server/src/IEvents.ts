export interface AccountChange {
    id: string,
    data: string, //b64 encoded
}

/** Kyogen Events */
// GameStateChanged
export interface EventGameStateChanged {
    instance: string,
    newState: string // PlayPhase
}

// NewPlayer
export interface EventNewPlayer {
    instance: string,
    playerId: AccountChange,
    authority: string // Pubkey
    clan: string // Clan
}

// SpawnClaimed
// UnitSpawned
// UnitMoved
// UnitAttacked

/** Structures Events */
// Meteor Mined
// PortalUsed
// LootableLooted
// GameFished
