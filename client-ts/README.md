# Deploy Flow

## Admin Setup
1. Deploy all programs
2. Initialize Registry (core_ds_program_id, payer_as_authority)
3. Register Components w/ Registry (schema, payer)
4. Initialize Kyogen AB (ComponentIndex)
5. Initialize Structures AB (ComponentIndex) 
6. Initialize Cards AB (ComponentIndex)
7. Register ABs  (Kyogen, Structures, Cards)
8. Register AB w/ Components

## Game Setup
1. After Creating Game with Kyogen, init_index Structures & Cards with same payer, they'll CPI into Registry to register themselves
2. Mint an SPL token and give it to Instance Index (for Solarite transfers)
