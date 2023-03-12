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
const CONNECTION = new anchor.web3.Connection(process.env.CONNECTION_URL, 'confirmed');
const ADMIN_KEY = anchor.web3.Keypair.fromSecretKey(Buffer.from(JSON.parse(readFileSync(process.env.PRIVATE_KEY_PATH).toString())));
let kyogen = new sdk.Kyogen(
    programs.COREDS.toString(),
    programs.REGISTRY.toString(),
    programs.KYOGEN.toString(),
    ADMIN_KEY.publicKey.toString()
);


const PLAYER1 = new anchor.web3.Keypair();
CONNECTION.requestAirdrop(PLAYER1.publicKey, 10e9);
const PLAYER2 = new anchor.web3.Keypair();
CONNECTION.requestAirdrop(PLAYER2.publicKey, 10e9);

let p1kyogen = new sdk.Kyogen(
    programs.COREDS.toString(),
    programs.REGISTRY.toString(),
    programs.KYOGEN.toString(),
    PLAYER1.publicKey.toString()
);

let p2kyogen = new sdk.Kyogen(
    programs.COREDS.toString(),
    programs.REGISTRY.toString(),
    programs.KYOGEN.toString(),
    PLAYER2.publicKey.toString()
);

const instance = BigInt(process.argv[2]);
let gamestate = new sdk.GameState(
    process.env.CONNECTION_URL,
    programs.KYOGEN.toString(),
    programs.REGISTRY.toString(),
    programs.COREDS.toString(),
    instance
);

// Needed to configure blueprints
let units = YAML.parseAllDocuments(readFileSync('./assets/units.yml', {encoding: "utf-8"}));

// Print state of map and players
// Create a couple players
// Start/Unpause the game
// Check hand
// Spawn units
// Attack units

simulate();
// Basic Kyogen Simulation
async function simulate(){
    console.log("Instance: ", instance.toString());
    console.log("Waiting 5s to get airdrop confirmation.")
    await new Promise(resolve => setTimeout(resolve, 5000)); // 5 sec

    // Load Blueprint into state
    let unit_names = [];
    for(let unit of units){
        unit_names.push(unit.toJSON().metadata.name);
    }
    gamestate.add_blueprints(unit_names);

    // Print Map
    await gamestate.load_state();
    console.log(gamestate.debug());
    console.log(gamestate.get_map());
    console.log("Play Phase: ", gamestate.get_play_phase());

    // Create a couple players
    await createPlayers();
    await gamestate.load_state(); //refresh state after creating players
    console.log("Players: ", gamestate.get_players());

    // Check Player Hands
    console.log("Player 1 Hand: ", getPlayerHand(PLAYER1));
    console.log("Player 2 Hand: ", getPlayerHand(PLAYER2));

    // Unpause the game
    await unpause();

    // Spawn Units
    await spawnUnit(p1kyogen, PLAYER1, 0, 0, "Ancient Samurai");
    await gamestate.load_state();

    // Print Map
    console.log(gamestate.get_map());
    
}

async function createPlayers() {
    const ix1 = ixWasmToJs(p1kyogen.init_player(
        instance,
        randomU64(),
        'Player 1',
        'Ancients'
    ));

    const msg = new anchor.web3.TransactionMessage({
        payerKey: PLAYER1.publicKey,
        recentBlockhash: (await CONNECTION.getLatestBlockhash()).blockhash,
        instructions: [
            ix1
        ]
    }).compileToLegacyMessage();
    const tx = new anchor.web3.VersionedTransaction(msg);
    tx.sign([PLAYER1]);
    const sig = await CONNECTION.sendTransaction(tx);
    await CONNECTION.confirmTransaction(sig);    

    const ix2 = ixWasmToJs(p2kyogen.init_player(
        instance,
        randomU64(),
        'Player 2',
        'Wildings'
    ));
    const msg2 = new anchor.web3.TransactionMessage({
        payerKey: PLAYER2.publicKey,
        recentBlockhash: (await CONNECTION.getLatestBlockhash()).blockhash,
        instructions: [
            ix2
        ]
    }).compileToLegacyMessage();
    const tx2 = new anchor.web3.VersionedTransaction(msg2);
    tx2.sign([PLAYER2]);
    const sig2 = await CONNECTION.sendTransaction(tx2);
    await CONNECTION.confirmTransaction(sig2);    

    console.log("Players 1 and 2 Created!");
}

async function unpause() {
    const ix1 = ixWasmToJs(
        kyogen.change_game_state(instance, "Play")
    );

    const msg = new anchor.web3.TransactionMessage({
        payerKey: ADMIN_KEY.publicKey,
        recentBlockhash: (await CONNECTION.getLatestBlockhash()).blockhash,
        instructions: [
            ix1
        ]
    }).compileToLegacyMessage();
    const tx = new anchor.web3.VersionedTransaction(msg);
    tx.sign([ADMIN_KEY]);
    const sig = await CONNECTION.sendTransaction(tx);
    await CONNECTION.confirmTransaction(sig);
    await gamestate.load_state();
    console.log("Play Phase: ", gamestate.get_play_phase())    
}

function getPlayerHand(player: anchor.web3.Keypair) {
    let players = gamestate.get_players();
    const playerHand:string[] = players.find((playerJSON:any) => playerJSON.owner === player.publicKey.toString()).cards;
    return playerHand.map((cardKeyString) => {
        return gamestate.get_blueprint_name(cardKeyString);
    });
}

async function spawnUnit(pkyogen: sdk.Kyogen, player: anchor.web3.Keypair, x:number, y:number, unitName:string){
    let ix = ixWasmToJs(
        pkyogen.spawn_unit(
            instance,
            randomU64(),
            BigInt(gamestate.get_tile_id(x, y)),
            BigInt(gamestate.get_player_by_key(player.publicKey.toString()).id),
            gamestate.get_blueprint_key(unitName)
        )
    ); 

    const msg = new anchor.web3.TransactionMessage({
        payerKey: player.publicKey,
        recentBlockhash: (await CONNECTION.getLatestBlockhash()).blockhash,
        instructions: [
            ix
        ]
    }).compileToLegacyMessage();
    const tx = new anchor.web3.VersionedTransaction(msg);
    tx.sign([player]);
    const sig = await CONNECTION.sendTransaction(tx);
    await CONNECTION.confirmTransaction(sig);   
}