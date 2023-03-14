import * as dotenv from 'dotenv';
dotenv.config();
dotenv.config({ path: `.env.local`, override: true });
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
const registry = new sdk.Registry(
    programs.COREDS.toString(),
    programs.REGISTRY.toString(),
    ADMIN_KEY.publicKey.toString()
);
const kyogen = new sdk.Kyogen(
    programs.COREDS.toString(),
    programs.REGISTRY.toString(),
    programs.KYOGEN.toString(),
    ADMIN_KEY.publicKey.toString()
);
const structures = new sdk.Structures(
    programs.COREDS.toString(),
    programs.REGISTRY.toString(),
    programs.KYOGEN.toString(),
    programs.STRUCTURES.toString(),
    ADMIN_KEY.publicKey.toString()
);


let config = YAML.parse(readFileSync('./configs/TestConfig.yml', {encoding: "utf-8"}));

main();
async function main(){
    // Assume 01_InitProgram.ts has been run

    // Create SPL Mint per game
    const gameMint = await create_mint();
    config.game_token = gameMint.toString();

    // Create Kyogen Game instance
    const instance = await create_game_instance();
    console.log("Game Instance: ", instance.toString());

    // Register ABs for Instance
    await append_registry_index(instance);

    // Init Index for Structures
    await init_structure_index(instance);

    // Mint Tokens into Structure Index
    await mint_spl(instance);

    // Init Map
    await init_map(instance);

    // Init Tiles
    await init_tiles(instance);
    await new Promise(resolve => setTimeout(resolve, 5000)); // 5 sec

    // TODO: Init Structures
    await init_structures(instance);
}

async function create_game_instance(): Promise<bigint> {
    let newInstanceId = randomU64();
    const createGameInstanceIx = ixWasmToJs(
        // See json_wrappers.rs for GameConfigJSON object spec
        kyogen.create_game_instance(newInstanceId, config)
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

async function init_structure_index(instance:bigint){
    const ix = ixWasmToJs(
        structures.init_structure_index(instance, config.game_token)
    );
    const msg = new anchor.web3.TransactionMessage({
        payerKey: ADMIN_KEY.publicKey,
        recentBlockhash: (await CONNECTION.getLatestBlockhash()).blockhash,
        instructions: [ix]
    }).compileToLegacyMessage();
    const tx = new anchor.web3.VersionedTransaction(msg);
    tx.sign([ADMIN_KEY]);
    const sig = await CONNECTION.sendTransaction(tx);
    await CONNECTION.confirmTransaction(sig);
    console.log(`Structure Index created: ${sig}`); 
}

async function append_registry_index(instance: bigint) {
    // Append Kyogen AB
    const appendKyogenIx = ixWasmToJs(
        registry.append_registry_index(
            sdk.Kyogen.get_kyogen_signer_str(programs.KYOGEN.toString()),
            instance,
        )
    )
    // Append Structures AB
    const appendStructuresIx = ixWasmToJs(
        registry.append_registry_index(
            sdk.Structures.get_structures_signer_str(programs.STRUCTURES.toString()),
            instance,
        )
    )

    const msg = new anchor.web3.TransactionMessage({
        payerKey: ADMIN_KEY.publicKey,
        recentBlockhash: (await CONNECTION.getLatestBlockhash()).blockhash,
        instructions: [
            appendKyogenIx,
            appendStructuresIx
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
    let tileIxGroup = [];
    for(let x=0; x<config.mapmeta.max_x; x++) {
        for(let y=0; y<config.mapmeta.max_y; y++) {
            let tileId = randomU64();
            const possibleSpawn = config.spawns.find((spawn:any) => {
                if(spawn.x == x && spawn.y == y){
                    return spawn
                } else {
                    return undefined;
                }
            })

            let spawnable = possibleSpawn ? true : false;
            let spawnCost = possibleSpawn ? BigInt(possibleSpawn.cost) : BigInt(0);
            let clan = possibleSpawn ? possibleSpawn.clan : "";


            const initTileIx = ixWasmToJs(
                kyogen.init_tile(
                    instance,
                    tileId,
                    x,
                    y,
                    spawnable,
                    spawnCost,
                    clan,
                )
            );
            tileIxGroup.push(initTileIx);   
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
    await Promise.all(tx_group).then(() => {
        console.log("Tiles created!");
    })
}

async function init_structures(instance:bigint) {
    let ixs = [];
    let gamestate = new sdk.GameState(
        CONNECTION.rpcEndpoint, 
        programs.KYOGEN.toString(),
        programs.REGISTRY.toString(),
        programs.COREDS.toString(),
        programs.STRUCTURES.toString(),
        instance,
    );
    await gamestate.load_state();

    for(let s of config.structures) {
        ixs.push(ixWasmToJs(
            structures.init_structure(
                instance,
                randomU64(),
                BigInt(gamestate.get_tile_id(s.x, s.y)),
                s.x,
                s.y,
                gamestate.get_blueprint_key(s.structure_blueprint)
            )
        ));
    }

    let ix_groups = await ixPack(ixs);
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
    await Promise.all(tx_group).then(() => {
        console.log("Structures created!");
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

async function mint_spl(instance:bigint) {
    // Mint Max Token Amount into ATA for Structures Index
    let si = new anchor.web3.PublicKey(structures.get_structures_index(instance));
    let mint = new anchor.web3.PublicKey(config.game_token);

    let structures_ata = await spl.getAssociatedTokenAddress(
        mint,
        si,
        true
    );

    await spl.mintTo(
        CONNECTION,
        ADMIN_KEY,
        mint,
        structures_ata,
        ADMIN_KEY.publicKey,
        config.max_score,
    );
    console.log(`${config.max_score} tokens minted`)
}