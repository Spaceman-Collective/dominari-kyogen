import * as dotenv from 'dotenv';
dotenv.config();
dotenv.config({ path: `.env.local`, override: true });
import * as anchor from '@coral-xyz/anchor';
import {readFileSync} from 'fs';
import * as sdk from '../kyogen-sdk/kyogen-sdk-nodejs/kyogen_sdk';
import { ixWasmToJs, randomU64 } from './util';
import YAML from 'yaml';

const programs = {
    COREDS: new anchor.web3.PublicKey(process.env.COREDS_ID),
    REGISTRY: new anchor.web3.PublicKey(process.env.REGISTRY_ID),
    KYOGEN: new anchor.web3.PublicKey(process.env.KYOGEN_ID),
    STRUCTURES: new anchor.web3.PublicKey(process.env.STRUCTURES_ID)
}
const CONNECTION = new anchor.web3.Connection(process.env.CONNECTION_URL, 'processed');
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
    programs.STRUCTURES.toString(),
    instance
);

// Needed to configure blueprints
let units = YAML.parseAllDocuments(readFileSync('./assets/units.yml', {encoding: "utf-8"}));

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
    await spawnUnit(p2kyogen, PLAYER2, 7, 0, 'Wilding Samurai');
    await gamestate.load_state();

    // Print Map
    console.log("Ancient Troop: ");
    await printTroopAtTile(0,0);
    console.log("Wilding Troop: ");
    await printTroopAtTile(7,0);
    
    // Move units next to each other
    await moveTroop(p1kyogen, PLAYER1, 0, 0, 1, 0), 
    await moveTroop(p2kyogen, PLAYER2, 7, 0, 6, 0),    
    await new Promise(resolve => setTimeout(resolve, 5000)); // 5 sec

    await moveTroop(p1kyogen, PLAYER1, 1, 0, 2, 0),
    await moveTroop(p2kyogen, PLAYER2, 6, 0, 5, 0),    
    await new Promise(resolve => setTimeout(resolve, 5000)); // 5 sec

    await moveTroop(p1kyogen, PLAYER1, 2, 0, 3, 0),
    await moveTroop(p2kyogen, PLAYER2, 5, 0, 4, 0),    
    await new Promise(resolve => setTimeout(resolve, 5000)); // 5 sec

    // Print Map
    console.log(gamestate.get_map());
    console.log("Ancient Troop: ");
    await printTroopAtTile(3,0);
    console.log("Wilding Troop: ");
    await printTroopAtTile(4,0)
    
    // Attack Wilding Troop
    await attackTile(p1kyogen, PLAYER1, 3, 0, 4, 0);
    await gamestate.load_state();

    // Print attack Results:
    console.log("Ancient Troop: ");
    await printTroopAtTile(3,0);
    console.log("Wilding Troop: ");
    await printTroopAtTile(4,0);
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
        kyogen.change_game_state(instance, BigInt(gamestate.get_map_id()), "Play")
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
            BigInt(gamestate.get_map_id()),
            randomU64(),
            BigInt(gamestate.get_tile_id(x, y)),
            BigInt(gamestate.get_playerjson_by_key(player.publicKey.toString()).id),
            gamestate.get_blueprint_key(unitName)
        )
    ); 

    const msg = new anchor.web3.TransactionMessage({
        payerKey: player.publicKey,
        recentBlockhash: (await CONNECTION.getLatestBlockhash()).blockhash,
        instructions: [
            anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({units: 400000}),
            ix
        ]
    }).compileToLegacyMessage();
    const tx = new anchor.web3.VersionedTransaction(msg);
    tx.sign([player]);
    const sig = await CONNECTION.sendTransaction(tx);
    await CONNECTION.confirmTransaction(sig);   
}

async function printTroopAtTile(x:number, y:number) {
    await gamestate.load_state();
    let map = gamestate.get_map();
    let troop = map.tiles.find((tile:any) => tile.x == x && tile.y == y).troop;
    console.log(JSON.stringify(troop, null, 2));
}

async function moveTroop(
    pkyogen: sdk.Kyogen, 
    player: anchor.web3.Keypair, 
    from_x: number, from_y: number,
    to_x: number, to_y: number, 
){
    await gamestate.load_state();
    let from_id = gamestate.get_tile_id(from_x, from_y);
    let from = gamestate.get_tile_json(BigInt(from_id));
    let unit = BigInt(from.troop.id);
    let to = BigInt(gamestate.get_tile_id(to_x, to_y));

    let ix = ixWasmToJs(
        pkyogen.move_unit(
            instance,
            BigInt(gamestate.get_map_id()),
            unit,
            BigInt(gamestate.get_playerjson_by_key(player.publicKey.toString()).id),
            from.id,
            to
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

async function attackTile(
    pkyogen: sdk.Kyogen, 
    player: anchor.web3.Keypair, 
    from_x: number, from_y: number,
    to_x: number, to_y: number,   
){
    await gamestate.load_state();
    let map = gamestate.get_map();
    let attacker = map.tiles.find((tile:any) => tile.x == from_x && tile.y == from_y).troop.id;
    let defending_tile = map.tiles.find((tile:any) => tile.x == to_x && tile.y == to_y);
    let defender = defending_tile.troop.id;
    

    let ix = ixWasmToJs(
        pkyogen.attack_unit(
            instance,
            BigInt(gamestate.get_map_id()),
            BigInt(attacker),
            BigInt(defender),
            BigInt(defending_tile.id)
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
