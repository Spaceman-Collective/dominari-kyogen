/* tslint:disable */
/* eslint-disable */
/**
* Initialize Javascript logging and panic handler
*/
export function solana_program_init(): void;
/**
*/
export class BlueprintIndex {
  free(): void;
/**
* @param {string} dominari
* @returns {BlueprintIndex}
*/
  static new(dominari: string): BlueprintIndex;
/**
* @param {string} blueprint
*/
  insert_blueprint_name(blueprint: string): void;
/**
*
*     * Returns the pubkey if no matching name is found
*     * Basically "unkown" Blueprint
*     
* @param {string} pubkey
* @returns {string}
*/
  get_blueprint_name(pubkey: string): string;
/**
* @param {string} blueprint
* @returns {string}
*/
  get_blueprint_key(blueprint: string): string;
}
/**
*/
export class ComponentIndex {
  free(): void;
/**
* @param {string} registry_id
*/
  constructor(registry_id: string);
/**
* @param {string} schema
*/
  insert_component_url(schema: string): void;
/**
* @param {string} schema
* @returns {string}
*/
  get_component_pubkey_as_str(schema: string): string;
/**
* @param {string} schema
* @returns {Pubkey}
*/
  get_component_pubkey(schema: string): Pubkey;
/**
* @param {string} pubkey
* @returns {string}
*/
  get_component_url(pubkey: string): string;
}
/**
*/
export class GameState {
  free(): void;
/**
* @param {string} rpc
* @param {string} kyogen_str
* @param {string} registry_str
* @param {string} coreds_str
* @param {string} structures_str
* @param {bigint} instance
*/
  constructor(rpc: string, kyogen_str: string, registry_str: string, coreds_str: string, structures_str: string, instance: bigint);
/**
* @param {any} blueprints_json
*/
  add_blueprints(blueprints_json: any): void;
/**
* @param {string} pubkey
* @returns {string}
*/
  get_blueprint_name(pubkey: string): string;
/**
* @param {string} name
* @returns {string}
*/
  get_blueprint_key(name: string): string;
/**
* @returns {string}
*/
  get_play_phase(): string;
/**
* @returns {string}
*/
  get_map_id(): string;
/**
* @returns {any}
*/
  get_current_high_score(): any;
/**
* @returns {any}
*/
  get_game_config(): any;
/**
* @returns {Promise<void>}
*/
  update_index(): Promise<void>;
/**
* @returns {Promise<void>}
*/
  load_state(): Promise<void>;
/**
* @param {bigint} entity_id
* @returns {Promise<void>}
*/
  update_entity(entity_id: bigint): Promise<void>;
/**
* @param {number} x
* @param {number} y
* @returns {string}
*/
  get_tile_id(x: number, y: number): string;
/**
* @param {number} x
* @param {number} y
* @returns {string}
*/
  get_structure_id(x: number, y: number): string;
/**
* @param {bigint} tile_id
* @returns {any}
*/
  get_tile_json(tile_id: bigint): any;
/**
* @param {bigint} structure_id
* @returns {any}
*/
  get_structure_json(structure_id: bigint): any;
/**
* @param {bigint} troop_id
* @returns {any}
*/
  get_troop_json(troop_id: bigint): any;
/**
* @returns {any}
*/
  get_map(): any;
/**
* @returns {any}
*/
  get_players(): any;
/**
* @param {bigint} player_id
* @returns {any}
*/
  get_player_json(player_id: bigint): any;
/**
* @param {string} player_key
* @returns {any}
*/
  get_playerjson_by_key(player_key: string): any;
/**
*/
  coreds_id: Pubkey;
/**
*/
  instance: bigint;
/**
*/
  kyogen_id: Pubkey;
/**
*/
  registry_id: Pubkey;
/**
*/
  structures_id: Pubkey;
}
/**
* A hash; the 32-byte output of a hashing algorithm.
*
* This struct is used most often in `solana-sdk` and related crates to contain
* a [SHA-256] hash, but may instead contain a [blake3] hash, as created by the
* [`blake3`] module (and used in [`Message::hash`]).
*
* [SHA-256]: https://en.wikipedia.org/wiki/SHA-2
* [blake3]: https://github.com/BLAKE3-team/BLAKE3
* [`blake3`]: crate::blake3
* [`Message::hash`]: crate::message::Message::hash
*/
export class Hash {
  free(): void;
/**
* Create a new Hash object
*
* * `value` - optional hash as a base58 encoded string, `Uint8Array`, `[number]`
* @param {any} value
*/
  constructor(value: any);
/**
* Return the base58 string representation of the hash
* @returns {string}
*/
  toString(): string;
/**
* Checks if two `Hash`s are equal
* @param {Hash} other
* @returns {boolean}
*/
  equals(other: Hash): boolean;
/**
* Return the `Uint8Array` representation of the hash
* @returns {Uint8Array}
*/
  toBytes(): Uint8Array;
}
/**
* A directive for a single invocation of a Solana program.
*
* An instruction specifies which program it is calling, which accounts it may
* read or modify, and additional data that serves as input to the program. One
* or more instructions are included in transactions submitted by Solana
* clients. Instructions are also used to describe [cross-program
* invocations][cpi].
*
* [cpi]: https://docs.solana.com/developing/programming-model/calling-between-programs
*
* During execution, a program will receive a list of account data as one of
* its arguments, in the same order as specified during `Instruction`
* construction.
*
* While Solana is agnostic to the format of the instruction data, it has
* built-in support for serialization via [`borsh`] and [`bincode`].
*
* [`borsh`]: https://docs.rs/borsh/latest/borsh/
* [`bincode`]: https://docs.rs/bincode/latest/bincode/
*
* # Specifying account metadata
*
* When constructing an [`Instruction`], a list of all accounts that may be
* read or written during the execution of that instruction must be supplied as
* [`AccountMeta`] values.
*
* Any account whose data may be mutated by the program during execution must
* be specified as writable. During execution, writing to an account that was
* not specified as writable will cause the transaction to fail. Writing to an
* account that is not owned by the program will cause the transaction to fail.
*
* Any account whose lamport balance may be mutated by the program during
* execution must be specified as writable. During execution, mutating the
* lamports of an account that was not specified as writable will cause the
* transaction to fail. While _subtracting_ lamports from an account not owned
* by the program will cause the transaction to fail, _adding_ lamports to any
* account is allowed, as long is it is mutable.
*
* Accounts that are not read or written by the program may still be specified
* in an `Instruction`'s account list. These will affect scheduling of program
* execution by the runtime, but will otherwise be ignored.
*
* When building a transaction, the Solana runtime coalesces all accounts used
* by all instructions in that transaction, along with accounts and permissions
* required by the runtime, into a single account list. Some accounts and
* account permissions required by the runtime to process a transaction are
* _not_ required to be included in an `Instruction`s account list. These
* include:
*
* - The program ID &mdash; it is a separate field of `Instruction`
* - The transaction's fee-paying account &mdash; it is added during [`Message`]
*   construction. A program may still require the fee payer as part of the
*   account list if it directly references it.
*
* [`Message`]: crate::message::Message
*
* Programs may require signatures from some accounts, in which case they
* should be specified as signers during `Instruction` construction. The
* program must still validate during execution that the account is a signer.
*/
export class Instruction {
  free(): void;
}
/**
*/
export class Instructions {
  free(): void;
/**
*/
  constructor();
/**
* @param {Instruction} instruction
*/
  push(instruction: Instruction): void;
}
/**
* A vanilla Ed25519 key pair
*/
export class Keypair {
  free(): void;
/**
* Create a new `Keypair `
*/
  constructor();
/**
* Convert a `Keypair` to a `Uint8Array`
* @returns {Uint8Array}
*/
  toBytes(): Uint8Array;
/**
* Recover a `Keypair` from a `Uint8Array`
* @param {Uint8Array} bytes
* @returns {Keypair}
*/
  static fromBytes(bytes: Uint8Array): Keypair;
/**
* Return the `Pubkey` for this `Keypair`
* @returns {Pubkey}
*/
  pubkey(): Pubkey;
}
/**
*/
export class Kyogen {
  free(): void;
/**
* @param {string} core_id
* @param {string} registry_id
* @param {string} kyogen_id
* @param {string} payer
*/
  constructor(core_id: string, registry_id: string, kyogen_id: string, payer: string);
/**
* @param {ComponentIndex} component_index
* @returns {any}
*/
  initialize(component_index: ComponentIndex): any;
/**
* @param {string} name
* @param {ComponentIndex} component_index
* @param {any} blueprint_json
* @returns {any}
*/
  register_blueprint(name: string, component_index: ComponentIndex, blueprint_json: any): any;
/**
*
*     * Pass in a pubkey strings of the blueprints
*     
* @param {string} name
* @param {any} blueprints_list
* @returns {any}
*/
  register_pack(name: string, blueprints_list: any): any;
/**
* @param {bigint} instance
* @param {any} game_config_json
* @returns {any}
*/
  create_game_instance(instance: bigint, game_config_json: any): any;
/**
* @param {bigint} instance
* @param {bigint} map_id
* @param {string} play_phase_str
* @returns {any}
*/
  change_game_state(instance: bigint, map_id: bigint, play_phase_str: string): any;
/**
* @param {bigint} instance
* @param {bigint} entity_id
* @param {number} max_x
* @param {number} max_y
* @returns {any}
*/
  init_map(instance: bigint, entity_id: bigint, max_x: number, max_y: number): any;
/**
* @param {bigint} instance
* @param {bigint} entity_id
* @param {number} x
* @param {number} y
* @param {boolean} spawnable
* @param {bigint} spawn_cost
* @param {string} clan_str
* @returns {any}
*/
  init_tile(instance: bigint, entity_id: bigint, x: number, y: number, spawnable: boolean, spawn_cost: bigint, clan_str: string): any;
/**
* @param {bigint} instance
* @param {bigint} entity_id
* @param {string} name
* @param {string} clan_str
* @returns {any}
*/
  init_player(instance: bigint, entity_id: bigint, name: string, clan_str: string): any;
/**
* @param {bigint} instance
* @param {bigint} map_id
* @param {bigint} tile_id
* @param {bigint} unit_id
* @param {bigint} player_id
* @param {string} game_token_str
* @returns {any}
*/
  claim_spawn(instance: bigint, map_id: bigint, tile_id: bigint, unit_id: bigint, player_id: bigint, game_token_str: string): any;
/**
* @param {bigint} instance
* @param {bigint} map_id
* @param {bigint} unit_id
* @param {bigint} tile_id
* @param {bigint} player_id
* @param {string} unit_blueprint_str
* @returns {any}
*/
  spawn_unit(instance: bigint, map_id: bigint, unit_id: bigint, tile_id: bigint, player_id: bigint, unit_blueprint_str: string): any;
/**
* @param {bigint} instance
* @param {bigint} map_id
* @param {bigint} unit_id
* @param {bigint} player_id
* @param {bigint} from_tile_id
* @param {bigint} to_tile_id
* @returns {any}
*/
  move_unit(instance: bigint, map_id: bigint, unit_id: bigint, player_id: bigint, from_tile_id: bigint, to_tile_id: bigint): any;
/**
* @param {bigint} instance
* @param {bigint} map_id
* @param {bigint} attacker_id
* @param {bigint} defender_id
* @param {bigint} defending_tile_id
* @returns {any}
*/
  attack_unit(instance: bigint, map_id: bigint, attacker_id: bigint, defender_id: bigint, defending_tile_id: bigint): any;
/**
* @param {bigint} instance
* @param {bigint} entity_id
* @returns {any}
*/
  close_entity(instance: bigint, entity_id: bigint): any;
/**
* @param {string} kyogen_id
* @returns {string}
*/
  static get_kyogen_signer_str(kyogen_id: string): string;
/**
* @param {string} kyogen_id
* @param {string} name
* @returns {string}
*/
  static get_pack_key(kyogen_id: string, name: string): string;
/**
*/
  core_id: Pubkey;
/**
*/
  kyogen_id: Pubkey;
/**
*/
  payer: Pubkey;
/**
*/
  registry_id: Pubkey;
}
/**
* A Solana transaction message (legacy).
*
* See the [`message`] module documentation for further description.
*
* [`message`]: crate::message
*
* Some constructors accept an optional `payer`, the account responsible for
* paying the cost of executing a transaction. In most cases, callers should
* specify the payer explicitly in these constructors. In some cases though,
* the caller is not _required_ to specify the payer, but is still allowed to:
* in the `Message` structure, the first account is always the fee-payer, so if
* the caller has knowledge that the first account of the constructed
* transaction's `Message` is both a signer and the expected fee-payer, then
* redundantly specifying the fee-payer is not strictly required.
*/
export class Message {
  free(): void;
/**
* The id of a recent ledger entry.
*/
  recent_blockhash: Hash;
}
/**
* The address of a [Solana account][acc].
*
* Some account addresses are [ed25519] public keys, with corresponding secret
* keys that are managed off-chain. Often, though, account addresses do not
* have corresponding secret keys &mdash; as with [_program derived
* addresses_][pdas] &mdash; or the secret key is not relevant to the operation
* of a program, and may have even been disposed of. As running Solana programs
* can not safely create or manage secret keys, the full [`Keypair`] is not
* defined in `solana-program` but in `solana-sdk`.
*
* [acc]: https://docs.solana.com/developing/programming-model/accounts
* [ed25519]: https://ed25519.cr.yp.to/
* [pdas]: https://docs.solana.com/developing/programming-model/calling-between-programs#program-derived-addresses
* [`Keypair`]: https://docs.rs/solana-sdk/latest/solana_sdk/signer/keypair/struct.Keypair.html
*/
export class Pubkey {
  free(): void;
/**
* Create a new Pubkey object
*
* * `value` - optional public key as a base58 encoded string, `Uint8Array`, `[number]`
* @param {any} value
*/
  constructor(value: any);
/**
* Return the base58 string representation of the public key
* @returns {string}
*/
  toString(): string;
/**
* Check if a `Pubkey` is on the ed25519 curve.
* @returns {boolean}
*/
  isOnCurve(): boolean;
/**
* Checks if two `Pubkey`s are equal
* @param {Pubkey} other
* @returns {boolean}
*/
  equals(other: Pubkey): boolean;
/**
* Return the `Uint8Array` representation of the public key
* @returns {Uint8Array}
*/
  toBytes(): Uint8Array;
/**
* Derive a Pubkey from another Pubkey, string seed, and a program id
* @param {Pubkey} base
* @param {string} seed
* @param {Pubkey} owner
* @returns {Pubkey}
*/
  static createWithSeed(base: Pubkey, seed: string, owner: Pubkey): Pubkey;
/**
* Derive a program address from seeds and a program id
* @param {any[]} seeds
* @param {Pubkey} program_id
* @returns {Pubkey}
*/
  static createProgramAddress(seeds: any[], program_id: Pubkey): Pubkey;
/**
* Find a valid program address
*
* Returns:
* * `[PubKey, number]` - the program address and bump seed
* @param {any[]} seeds
* @param {Pubkey} program_id
* @returns {any}
*/
  static findProgramAddress(seeds: any[], program_id: Pubkey): any;
}
/**
*/
export class Registry {
  free(): void;
/**
* @param {string} coreds
* @param {string} registry_id
* @param {string} payer
*/
  constructor(coreds: string, registry_id: string, payer: string);
/**
* @returns {any}
*/
  initialize(): any;
/**
* @param {string} schema
* @returns {any}
*/
  register_component(schema: string): any;
/**
*
*     * @param ab_signer This is the AB Signer PDA, not the program address of the AB
*     
* @param {string} ab_signer
* @returns {any}
*/
  register_action_bundle(ab_signer: string): any;
/**
*
*     * @param component_list is a list of string schema urls or names
*     
* @param {string} ab_signer
* @param {any} component_list
* @returns {any}
*/
  add_components_to_action_bundle_registration(ab_signer: string, component_list: any): any;
/**
* @param {string} ab_signer
* @param {bigint} instance
* @returns {any}
*/
  append_registry_index(ab_signer: string, instance: bigint): any;
/**
* @param {string} registry_id
* @returns {string}
*/
  static get_registry_signer_str(registry_id: string): string;
}
/**
*/
export class StatelessSDK {
  free(): void;
/**
* @param {string} rpc
* @param {string} kyogen_str
* @param {string} registry_str
* @param {string} coreds_str
* @param {string} structures_str
*/
  constructor(rpc: string, kyogen_str: string, registry_str: string, coreds_str: string, structures_str: string);
/**
* @param {bigint} instance
* @returns {Promise<any>}
*/
  fetch_addresses(instance: bigint): Promise<any>;
/**
* @param {bigint} instance
* @param {bigint} id
* @returns {string}
*/
  fetch_address_by_id(instance: bigint, id: bigint): string;
/**
* @param {string} data
* @param {bigint} player_id
* @returns {any}
*/
  get_player_json(data: string, player_id: bigint): any;
/**
* @param {string} data
* @param {bigint} tile_id
* @returns {any}
*/
  get_tile_json(data: string, tile_id: bigint): any;
/**
* @param {string} data
* @param {bigint} structure_id
* @returns {any}
*/
  get_structure_json(data: string, structure_id: bigint): any;
/**
* @param {string} data
* @param {bigint} troop_id
* @returns {any}
*/
  get_troop_json(data: string, troop_id: bigint): any;
/**
*/
  coreds_id: Pubkey;
/**
*/
  kyogen_id: Pubkey;
/**
*/
  registry_id: Pubkey;
/**
*/
  structures_id: Pubkey;
}
/**
*/
export class Structures {
  free(): void;
/**
* @param {string} core_id
* @param {string} registry_id
* @param {string} kyogen_id
* @param {string} structures_id
* @param {string} payer
*/
  constructor(core_id: string, registry_id: string, kyogen_id: string, structures_id: string, payer: string);
/**
* @param {ComponentIndex} component_index
* @returns {any}
*/
  initialize(component_index: ComponentIndex): any;
/**
* @param {bigint} instance
* @param {string} game_token_mint
* @returns {any}
*/
  init_structure_index(instance: bigint, game_token_mint: string): any;
/**
* @param {bigint} instance
* @param {bigint} entity_id
* @param {bigint} tile_id
* @param {number} x
* @param {number} y
* @param {string} structure_blueprint_key
* @returns {any}
*/
  init_structure(instance: bigint, entity_id: bigint, tile_id: bigint, x: number, y: number, structure_blueprint_key: string): any;
/**
* @param {bigint} instance
* @param {bigint} map_id
* @param {bigint} meteor_id
* @param {bigint} tile_id
* @param {bigint} unit_id
* @param {bigint} player_id
* @param {string} game_token_mint
* @returns {any}
*/
  use_meteor(instance: bigint, map_id: bigint, meteor_id: bigint, tile_id: bigint, unit_id: bigint, player_id: bigint, game_token_mint: string): any;
/**
* @param {bigint} instance
* @param {bigint} map_id
* @param {string} game_token_mint
* @param {bigint} from_tile
* @param {bigint} from_portal
* @param {bigint} to_tile
* @param {bigint} to_portal
* @param {bigint} unit
* @returns {any}
*/
  use_portal(instance: bigint, map_id: bigint, game_token_mint: string, from_tile: bigint, from_portal: bigint, to_tile: bigint, to_portal: bigint, unit: bigint): any;
/**
* @param {bigint} instance
* @param {bigint} map_id
* @param {string} game_token_mint
* @param {bigint} tile_id
* @param {bigint} unit_id
* @param {bigint} lootable_id
* @param {bigint} player_id
* @param {string} pack_key
* @returns {any}
*/
  use_lootable(instance: bigint, map_id: bigint, game_token_mint: string, tile_id: bigint, unit_id: bigint, lootable_id: bigint, player_id: bigint, pack_key: string): any;
/**
* @param {bigint} instance
* @param {bigint} map_id
* @param {bigint} winning_player_id
* @returns {any}
*/
  claim_victory(instance: bigint, map_id: bigint, winning_player_id: bigint): any;
/**
* @param {bigint} instance
* @param {bigint} entity_id
* @returns {any}
*/
  close_structure(instance: bigint, entity_id: bigint): any;
/**
* @param {string} structures_id
* @returns {string}
*/
  static get_structures_signer_str(structures_id: string): string;
/**
* @param {bigint} instance
* @returns {string}
*/
  get_structures_index(instance: bigint): string;
/**
*/
  core_id: Pubkey;
/**
*/
  kyogen_id: Pubkey;
/**
*/
  payer: Pubkey;
/**
*/
  registry_id: Pubkey;
/**
*/
  structures_id: Pubkey;
}
/**
* An atomically-commited sequence of instructions.
*
* While [`Instruction`]s are the basic unit of computation in Solana,
* they are submitted by clients in [`Transaction`]s containing one or
* more instructions, and signed by one or more [`Signer`]s.
*
* [`Signer`]: crate::signer::Signer
*
* See the [module documentation] for more details about transactions.
*
* [module documentation]: self
*
* Some constructors accept an optional `payer`, the account responsible for
* paying the cost of executing a transaction. In most cases, callers should
* specify the payer explicitly in these constructors. In some cases though,
* the caller is not _required_ to specify the payer, but is still allowed to:
* in the [`Message`] structure, the first account is always the fee-payer, so
* if the caller has knowledge that the first account of the constructed
* transaction's `Message` is both a signer and the expected fee-payer, then
* redundantly specifying the fee-payer is not strictly required.
*/
export class Transaction {
  free(): void;
/**
* Create a new `Transaction`
* @param {Instructions} instructions
* @param {Pubkey | undefined} payer
*/
  constructor(instructions: Instructions, payer?: Pubkey);
/**
* Return a message containing all data that should be signed.
* @returns {Message}
*/
  message(): Message;
/**
* Return the serialized message data to sign.
* @returns {Uint8Array}
*/
  messageData(): Uint8Array;
/**
* Verify the transaction
*/
  verify(): void;
/**
* @param {Keypair} keypair
* @param {Hash} recent_blockhash
*/
  partialSign(keypair: Keypair, recent_blockhash: Hash): void;
/**
* @returns {boolean}
*/
  isSigned(): boolean;
/**
* @returns {Uint8Array}
*/
  toBytes(): Uint8Array;
/**
* @param {Uint8Array} bytes
* @returns {Transaction}
*/
  static fromBytes(bytes: Uint8Array): Transaction;
}
