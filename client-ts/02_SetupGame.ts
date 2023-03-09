import * as dotenv from 'dotenv';
dotenv.config();
import * as anchor from '@coral-xyz/anchor';
import {readFileSync} from 'fs';
import * as sdk from '../kyogen-sdk/kyogen-sdk-nodejs/kyogen_sdk';
import { ixWasmToJs, ixPack, randomU64 } from './util';
import YAML from 'yaml';
import * as spl from '@solana/spl-token';

const programs = {
    COREDS: new anchor.web3.PublicKey(process.env.COREDS_ID),
    REGISTRY: new anchor.web3.PublicKey(process.env.REGISTRY_ID),
    KYOGEN: new anchor.web3.PublicKey(process.env.KYOGEN_ID),
    STRUCTURES: new anchor.web3.PublicKey(process.env.STRUCTURES_ID)
}
const ADMIN_KEY = anchor.web3.Keypair.fromSecretKey(Buffer.from(JSON.parse(readFileSync(process.env.PRIVATE_KEY_PATH).toString())));
const CONNECTION = new anchor.web3.Connection(process.env.CONNECTION_URL, 'finalized');
let registry = new sdk.Registry(
    programs.COREDS.toString(),
    programs.REGISTRY.toString(),
    ADMIN_KEY.publicKey.toString()
);
let kyogen = new sdk.Kyogen(
    programs.COREDS.toString(),
    programs.REGISTRY.toString(),
    programs.KYOGEN.toString(),
    ADMIN_KEY.publicKey.toString()
);
let component_index = new sdk.ComponentIndex(programs.REGISTRY.toString());
let config = YAML.parse(readFileSync('./configs/4PlayerTest.yml', {encoding: "utf-8"}));

main();
async function main(){
    // Assume 01_InitProgram.ts has been run

    // Create SPL Mint per game
    const gameMint = await create_mint();
    config.game_token = gameMint.toString();

    // Create Kyogen Game instance
    const instance = await create_game_instance();

    // Register ABs for Instance
    await append_registry_index(instance);

    // TODO: Init Index for Structures
    // TODO: Mint Tokens into Structure Index

    // Init Map
    await init_map(instance);

    // Init Tiles
    await init_tiles(instance);

    // TODO: Init Structures
}

async function create_game_instance(): Promise<bigint> {
    let newInstanceId = randomU64();
    const createGameInstanceIx = ixWasmToJs(
        // See json_wrappers.rs for GameConfigJSON object spec
        kyogen.create_game_instance(newInstanceId, {
            max_players: config.max_players,
            game_token: config.game_token,
            spawn_claim_multiplier: config.spawn_claim_multiplier
        })
    );
    
    const msg = new anchor.web3.TransactionMessage({
        payerKey: ADMIN_KEY.publicKey,
        recentBlockhash: (await CONNECTION.getLatestBlockhash()).blockhash,
        instructions: [createGameInstanceIx]
    }).compileToLegacyMessage();
    const tx = new anchor.web3.VersionedTransaction(msg);
    tx.sign([ADMIN_KEY]);
    const sig = await CONNECTION.sendTransaction(tx);
    await CONNECTION.confirmTransaction(sig);
    console.log(`Game Instance ${newInstanceId.toString()} created: ${sig}`); 

    return newInstanceId;
}

async function append_registry_index(instance: bigint) {
    // Append Kyogen AB
    const appendKyogenIx = ixWasmToJs(
        registry.append_registry_index(
            sdk.Kyogen.get_kyogen_signer_str(programs.KYOGEN.toString()),
            instance,
        )
    )
    // TODO: Append Structures AB

    const msg = new anchor.web3.TransactionMessage({
        payerKey: ADMIN_KEY.publicKey,
        recentBlockhash: (await CONNECTION.getLatestBlockhash()).blockhash,
        instructions: [
            appendKyogenIx
        ]
    }).compileToLegacyMessage();
    const tx = new anchor.web3.VersionedTransaction(msg);
    tx.sign([ADMIN_KEY]);
    const sig = await CONNECTION.sendTransaction(tx);
    await CONNECTION.confirmTransaction(sig);    
    console.log("Action bundles registered with instance...");
}

async function init_map(instance: bigint) {
    const mapId = randomU64();
    const initMapIx = ixWasmToJs(
        kyogen.init_map(instance, mapId, config.mapmeta.max_x, config.mapmeta.max_y)
    )

    const msg = new anchor.web3.TransactionMessage({
        payerKey: ADMIN_KEY.publicKey,
        recentBlockhash: (await CONNECTION.getLatestBlockhash()).blockhash,
        instructions: [initMapIx]
    }).compileToLegacyMessage();
    const tx = new anchor.web3.VersionedTransaction(msg);
    tx.sign([ADMIN_KEY]);
    const sig = await CONNECTION.sendTransaction(tx);
    await CONNECTION.confirmTransaction(sig);
    console.log(`Map ${mapId.toString()} initialized: ${sig}`); 
}

async function init_tiles(instance:bigint) {
    // Build Spawns Array
    let spawns: bigint[][] = new Array(config.mapmeta.max_y).fill(new Array(config.mapmeta.max_x).fill(undefined));
    for(let spawn of config.spawns) {
        spawns[spawn.x][spawn.y] = BigInt(spawn.cost)
    }

    let tileIxGroup = [];
    for(let x=0; x<config.mapmeta.max_x; x++) {
        for(let y=0; y<config.mapmeta.max_y; y++) {
            let tileId = randomU64();
            let spawnable = spawns[x][y] ? true : false;
            let spawnCost = spawnable ? spawns[x][y] : BigInt(0);

            const initTileIx = ixWasmToJs(
                kyogen.init_tile(
                    instance,
                    tileId,
                    x,
                    y,
                    spawnable,
                    spawnCost
                )
            );   
        }
    }

    let ix_groups = await ixPack(tileIxGroup);
    let tx_group = [];
    for(let group of ix_groups){
        const msg = new anchor.web3.TransactionMessage({
            payerKey: ADMIN_KEY.publicKey,
            recentBlockhash: (await CONNECTION.getLatestBlockhash()).blockhash,
            instructions: group
        }).compileToLegacyMessage();
        const tx = new anchor.web3.VersionedTransaction(msg);
        tx.sign([ADMIN_KEY]);
        let sig = await CONNECTION.sendTransaction(tx);
        tx_group.push(CONNECTION.confirmTransaction(sig));
    }
    Promise.all(tx_group).then(() => {
        console.log("Tiles created!");
    })
}


/**
 * Will create a new token to use for the game. Mints into the Structures Idx ATA 
 * SI ATA pays to Player ATA when mining asteroids
 * Player ATA pays to SI ATA when using Structures  
 */
async function create_mint(): Promise<anchor.web3.PublicKey> {
    // Create the Mint
    const mintAddress = await spl.createMint(
        CONNECTION,
        ADMIN_KEY,
        ADMIN_KEY.publicKey,
        ADMIN_KEY.publicKey,
        9
    );

    return mintAddress;
}

async function mint_spl() {
    // Create ATA For Structures Index
    // Mint Max Token Amount into ATA for Structures Index
}