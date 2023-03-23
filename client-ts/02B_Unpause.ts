import * as dotenv from 'dotenv';
dotenv.config();
dotenv.config({ path: `.env.local`, override: true });
import * as anchor from '@coral-xyz/anchor';
import {readFileSync} from 'fs';
import * as sdk from '../kyogen-sdk/kyogen-sdk-nodejs/kyogen_sdk';
import { ixWasmToJs, randomU64 } from './util';

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

const instance = BigInt(process.argv[2]);
let gamestate = new sdk.GameState(
    process.env.CONNECTION_URL,
    programs.KYOGEN.toString(),
    programs.REGISTRY.toString(),
    programs.COREDS.toString(),
    programs.STRUCTURES.toString(),
    instance
);

unpause(); 
async function unpause() {
    await gamestate.load_state();

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