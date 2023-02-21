# Deploy Flow

## Admin Setup
1. Deploy all programs
2. Initialize Registry (core_ds_program_id, payer_as_authority)
3. Register Components w/ Registry (schema, payer)
4. Register ABs  (Kyogen, Structures, Cards)
5. Register AB w/ Components
6. Initialize Kyogen AB (ComponentIndex)
7. Initialize Structures AB (ComponentIndex) 
8. Initialize Cards AB (ComponentIndex)

## Other Setup
1. Mint an SPL token and give it to players

## Game Setup
1. After Creating Game with Kyogen, initialize Structures & Cards with same payer, they'll CPI into Registry to register themselves
