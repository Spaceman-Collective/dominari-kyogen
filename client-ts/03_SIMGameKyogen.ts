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
const CONNECTION = new anchor.web3.Connection(process.env.CONNECTION_URL, 'finalized');

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


simulate();
// Basic Kyogen Simulation
async function simulate(){
    console.log("Instance: ", instance.toString());

    // Print Map
    await gamestate.load_state();
    let map = gamestate.get_map();
    console.log(map);
    
}


// Print state of map and players
// Create a couple players
// Start/Unpause the game
// Check hand
// Spawn units
// Attack units


