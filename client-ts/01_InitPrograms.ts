import * as dotenv from 'dotenv';
dotenv.config();
import * as anchor from '@coral-xyz/anchor';
import {readFileSync} from 'fs';
import * as sdk from '../kyogen-sdk/kyogen-sdk-nodejs/kyogen_sdk';
import { ixWasmToJs, ixPack } from './util';
import YAML from 'yaml';

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
)
let component_index = new sdk.ComponentIndex(programs.REGISTRY.toString());

let components = YAML.parse(readFileSync('./assets/components.yml', {encoding: "utf-8"}));
let units = YAML.parseAllDocuments(readFileSync('./assets/units.yml', {encoding: "utf-8"}));
let structures = YAML.parseAllDocuments(readFileSync('./assets/structures.yml', {encoding: "utf-8"}));
let packs = YAML.parseAllDocuments(readFileSync('./assets/packs.yml', {encoding: "utf-8"}));

main();
async function main() {
    // Assume Local Validator is spun up with Core DS program @ process.env.COREDS_ID
    // Assume Registry, Kyogen, Structure programs are deployed @ process.env.<Program Name> 

    console.log("ADMIN: ", ADMIN_KEY.publicKey.toString());

    // Initialize Programs & Register Components
    await init_programs();

    // Register ABs  (Kyogen, Structures, Cards) & register AB w/ Components
    await register_ab();

    // Register Blueprints with Kyogen
    await register_blueprints();

    // Register Packs
    await register_packs();
}

/**
 * Initialize Registry
 * Register Components w/ Registry
 * Initialize Action Bundles
 */
async function init_programs() {
    let instructions = []

    // Initialize Registry (core_ds_program_id, payer_as_authority)
    instructions.push(ixWasmToJs(registry.initialize()));
    console.log("Prepared registry init ix...");

    // Register Components w/ Registry (schema, payer)
    for(let component_uri of Object.values(components)) {
        instructions.push(
            ixWasmToJs(
                registry.register_component(<string>component_uri)
            )
        )
    }
    console.log("Prepared component registration ixs...");

    // Init Kyogen
    instructions.push(ixWasmToJs(kyogen.initialize(
        component_index
    )));
    console.log("Prepared kyogen init ix...")

    // TODO: Init Structures

    // Submit TX
    let ix_groups = await ixPack(instructions);
    for(let group of ix_groups){
        const msg = new anchor.web3.TransactionMessage({
            payerKey: ADMIN_KEY.publicKey,
            recentBlockhash: (await CONNECTION.getLatestBlockhash()).blockhash,
            instructions: group
        }).compileToLegacyMessage();
        const tx = new anchor.web3.VersionedTransaction(msg);
        tx.sign([ADMIN_KEY]);
        const sig = await CONNECTION.sendTransaction(tx);
        await CONNECTION.confirmTransaction(sig);
        console.log("TX Confirmed: ", sig); 
    }

    console.log("Registry initialized, components registered, action bundles initialized.");
}

// Register ABs
// Add Components to ABs
async function register_ab() {
    let instructions = [];
    // Register Kyogen
    instructions.push(ixWasmToJs(registry.register_action_bundle(sdk.Kyogen.get_kyogen_signer_str(programs.KYOGEN.toString()))));

    // Register Components w/ Kyogen
    instructions.push(ixWasmToJs(registry.add_components_to_action_bundle_registration(
        sdk.Kyogen.get_kyogen_signer_str(programs.KYOGEN.toString()),
        Object.values(components)
    )));
    // TODO: Register Structures

    // Submit Tx
    let ix_group = await ixPack(instructions);
    for(let group of ix_group){
        const msg = new anchor.web3.TransactionMessage({
            payerKey: ADMIN_KEY.publicKey,
            recentBlockhash: (await CONNECTION.getLatestBlockhash()).blockhash,
            instructions: group
        }).compileToLegacyMessage();
        const tx = new anchor.web3.VersionedTransaction(msg);
        tx.sign([ADMIN_KEY]);
        const sig = await CONNECTION.sendTransaction(tx);
        await CONNECTION.confirmTransaction(sig);
        console.log("TX Confirmed: ", sig);  
    }
    console.log("Action Bundles registered...");
}

async function register_blueprints() {
    let instructions = [];
    // Units
    for(let unit of units){
        instructions.push(
            ixWasmToJs(
                kyogen.register_blueprint(
                    unit.toJSON().metadata.name,
                    component_index,
                    unit.toJSON(),
                )
            )
        )
    }
    console.log("Prepared unit blueprints...");
    // Structures
    for(let structure of structures){
        instructions.push(
            ixWasmToJs(
                kyogen.register_blueprint(
                    structure.toJSON().metadata.name,
                    component_index,
                    structure.toJSON(),
                )
            )
        )
    }
    console.log("Prepared structure blueprints...");
    // Submit Tx
    // All blueprints registrations can be submitted together
    let txs = [];
    let ix_group = await ixPack(instructions);
    for(let group of ix_group){
        const msg = new anchor.web3.TransactionMessage({
            payerKey: ADMIN_KEY.publicKey,
            recentBlockhash: (await CONNECTION.getLatestBlockhash()).blockhash,
            instructions: group
        }).compileToLegacyMessage();
        const tx = new anchor.web3.VersionedTransaction(msg);
        tx.sign([ADMIN_KEY]);
        const sig = await CONNECTION.sendTransaction(tx);
        txs.push(CONNECTION.confirmTransaction(sig))
    }
    Promise.all(txs).then(() => {
        console.log("Blueprints registered...");
    })
}

async function register_packs(){
    let instructions = [];

    for(let doc of packs){
        let pack = doc.toJSON();
        let card_keys = (<string[]>pack.cards)
                            .map((name) => {
                                return anchor.web3.PublicKey.findProgramAddressSync([
                                    Buffer.from("blueprint"),
                                    Buffer.from(pack.name)
                                ],programs.KYOGEN)[0].toString()
                            })

        instructions.push(ixWasmToJs(kyogen.register_pack(
            pack.name,
            card_keys
        )));
    }

    // Submit Tx
    // Packs can be submitted together
    let txs = [];
    let ix_group = await ixPack(instructions);
    for(let group of ix_group){
        const msg = new anchor.web3.TransactionMessage({
            payerKey: ADMIN_KEY.publicKey,
            recentBlockhash: (await CONNECTION.getLatestBlockhash()).blockhash,
            instructions: group
        }).compileToLegacyMessage();
        const tx = new anchor.web3.VersionedTransaction(msg);
        tx.sign([ADMIN_KEY]);
        const sig = await CONNECTION.sendTransaction(tx);
        txs.push(CONNECTION.confirmTransaction(sig));
    }
    Promise.all(txs).then(() => {
        console.log("Packs registered...");
    })
}