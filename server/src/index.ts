import { StatelessSDK } from '../../kyogen-sdk/kyogen-sdk-nodejs/kyogen_sdk';
import * as anchor from '@coral-xyz/anchor';
import * as dotenv from 'dotenv';
dotenv.config();

import fastify from 'fastify';
import { readFileSync, writeFile, writeFileSync } from 'fs';
import fetch, { Headers } from 'node-fetch';

const server = fastify({
    /* https: {
        key: readFileSync(process.env.SERVER_KEY),
        cert: readFileSync(process.env.SERVER_CERT),
    } */
});

import {createSession, createChannel, Channel} from 'better-sse';

import { Idl } from "@coral-xyz/anchor/dist/cjs/idl";
import { BorshEventCoder} from "@coral-xyz/anchor";
import * as KyogenIDL from "../../target/idl/kyogen.json";
import * as StructuresIDL from "../../target/idl/structures.json";
import { Transaction } from './TransactionInterface';


const programs = {
    COREDS: new anchor.web3.PublicKey(process.env.COREDS_ID),
    REGISTRY: new anchor.web3.PublicKey(process.env.REGISTRY_ID),
    KYOGEN: new anchor.web3.PublicKey(process.env.KYOGEN_ID),
    STRUCTURES: new anchor.web3.PublicKey(process.env.STRUCTURES_ID)
}

const sdk = new StatelessSDK(
    process.env.RPC,
    programs.KYOGEN.toString(),
    programs.REGISTRY.toString(),
    programs.COREDS.toString(),
    programs.STRUCTURES.toString(),
);

interface Game {
    id: string,
    channel: Channel,
    addresses: AddressListJSON,
    hookId: string,
}

interface AddressListJSON {
    kyogen_index: string,
    structures_index: string,
    map: string,
    tiles: string[],
    units: string[],
    players: string[],
    portals: string[],
    healers: string[],
    lootables: string[],
    meteors: string[]
}

interface AddressType {
    type: "kyogen_index" | "structures_index" | "map" | "tile" | "unit" | "player" | "portal" | "healer" | "lootable" | "meteor",
    gameId: string,
}

let gameChannels: Map<string, Game> = new Map();
let addressToGameIdMap: Map<string, AddressType> = new Map();
let txnProcessedBuffer: string[] = [];
let gameIdsThatNeedCleanup: Set<string> = new Set();

const TXN_BUFFER_LENGTH = 10;
const LOG_START_INDEX = "Program data: ".length;
const KyogenEventCoder = new BorshEventCoder(KyogenIDL as Idl);
const StructuresEventCoder = new BorshEventCoder(StructuresIDL as Idl);


server.get('/', async (req, res) => {
    res.code(200).send("Kyogen Server is working!");
})

server.get('/game/:gameId', async (req, res) => {
    const {gameId} = req.params as any;
    console.log(`Creating Hook for Game ID: ${gameId}`);
    if(!gameChannels.has(gameId)){
        const addresses:AddressListJSON = await sdk.fetch_addresses(BigInt(gameId));
        const addressList = flattenAddressListJSON(addresses);
        // Track all the addresses for reverse lookup
        setReverseAddressLookup(gameId, addresses);

        // Channel doesn't exist
        const hookId = await createHook(gameId, addressList);
        const newChannel = createChannel();
        // Every time a session disconnects, check if it's the last session
        // If no more sessions are connected to this channel, deregister the webhook
        newChannel.on("session-deregistered", async () => {
            if(newChannel.sessionCount < 1) {
                console.log(`${gameId} has no more sessions connected!`)
                // Remvoe from addressToGameId Map as well
                addressList.forEach((addr: string) => {
                    addressToGameIdMap.delete(addr);
                })
                const wasRemoved = await removeHook(gameId);
                if(!wasRemoved){
                    gameIdsThatNeedCleanup.add(gameId);
                }
                gameChannels.delete(gameId);
                console.log(`Hook deregistered for ${gameId}`);
            }
        })

        gameChannels.set(gameId, {
            id: gameId,
            channel: newChannel,
            addresses,
            hookId,
        });
        console.log(`New channel created for game id: ${gameId}`)
    }

    // Channel now exists
    // create session then register it with channel
    const newUserSession = await createSession(req.raw, res.raw);
    // Sessions are automatically deregistered when they are disconnected
    gameChannels.get(gameId).channel.register(newUserSession);
    newUserSession.push("connected");
});

async function createHook(gameId:string, addresses: string[]): Promise<string> {
    try {
        const ENDPOINT = "https://api.shyft.to/sol/v1/callback/create"

        const requestOpts = {
            method: 'POST',
            headers: new Headers({
                'x-api-key': process.env.SHYFT_KEY,
                'Content-Type': "application/json"
            }),
            body: JSON.stringify({
                network: process.env.SOL_NETWORK,
                addresses,
                callback_url: `${process.env.SERVER_ADDRESS}/shyft`,
                enable_raw: true,
            }),
            timeout: 60000, //60s timeout
        };

        const response = await (await fetch(ENDPOINT, requestOpts)).json()
        return response.result.id as string;
    } catch (e) {
        throw e;
    }
}

async function updateHook(gameId:string, addresses: string[]): Promise<boolean> {
    try {
        const ENDPOINT = "https://api.shyft.to/sol/v1/callback/update"
        const requestOpts = {
            method: 'POST',
            headers: new Headers({
                'x-api-key': process.env.SHYFT_KEY,
                'Content-Type': "application/json"
            }),
            body: JSON.stringify({
                id: gameChannels.get(gameId).hookId,
                addresses,
            })
        };

        const response = await (await fetch(ENDPOINT, requestOpts)).json()
        console.log(response);
        return true; // success
    } catch (e) {
        console.error(e);
        return false; // not success
    }
}

async function removeHook(gameId:string): Promise<boolean> {
    console.log(`Trying to remove hook: ${gameChannels.get(gameId).hookId} for gameId: ${gameId}`)
    try {
        const ENDPOINT = "https://api.shyft.to/sol/v1/callback/remove"
        const requestOpts = {
            method: 'DELETE',
            headers: new Headers({
                'x-api-key': process.env.SHYFT_KEY,
                'Content-Type': "application/json"
            }),
            body: JSON.stringify({
                id: gameChannels.get(gameId).hookId,
            }),
            timeout: 120000 //120s timeout
        };

        const response = await (await fetch(ENDPOINT, requestOpts)).json()
        return true; //success
    } catch (e) {
        console.error(e);
        return false; // not success
    }
}

function flattenAddressListJSON(addrs:AddressListJSON): string[] {
    return [
        addrs.kyogen_index,
        addrs.structures_index,
        addrs.map,
        ...addrs.tiles,
        ...addrs.units,
        ...addrs.players,
        ...addrs.portals,
        ...addrs.healers,
        ...addrs.lootables,
        ...addrs.meteors
    ]
}

function setReverseAddressLookup(gameId:string, addrs: AddressListJSON) {
    addressToGameIdMap.set(addrs.kyogen_index, {
        gameId,
        type: "kyogen_index"
    });

    addressToGameIdMap.set(addrs.structures_index, {
        gameId,
        type: "structures_index"
    });

    addressToGameIdMap.set(addrs.map, {
        gameId,
        type: "map"
    });

    addrs.tiles.forEach((addr:string) => {
        addressToGameIdMap.set(addr, {
            gameId,
            type: "tile"
        })
    });

    addrs.units.forEach((addr:string) => {
        addressToGameIdMap.set(addr, {
            gameId,
            type: "unit"
        })
    });

    addrs.players.forEach((addr:string) => {
        addressToGameIdMap.set(addr, {
            gameId,
            type: "player"
        })
    });

    addrs.portals.forEach((addr:string) => {
        addressToGameIdMap.set(addr, {
            gameId,
            type: "portal"
        })
    });

    addrs.healers.forEach((addr:string) => {
        addressToGameIdMap.set(addr, {
            gameId,
            type: "healer"
        })
    });

    addrs.lootables.forEach((addr:string) => {
        addressToGameIdMap.set(addr, {
            gameId,
            type: "lootable"
        })
    });

    addrs.meteors.forEach((addr:string) => {
        addressToGameIdMap.set(addr, {
            gameId,
            type: "meteor"
        })
    });
}

/**
 * Shyft will hit this endpoint with payload
 */
server.post('/shyft', async (req, res) => {
    try {
        const txn:Transaction = req.body as Transaction;

        // Check to see if txn has already been processed
        if(txnProcessedBuffer.includes(txn.signatures[0])) {
            // Already processed this txn, redundant callback
            console.log("Already processed this transaction.")
            res.code(200);
            return;
        } else {
            // Keeps the buffer small so it doesn't fill up all of memory
            if(txnProcessedBuffer.length > TXN_BUFFER_LENGTH) {
                // Remove the first element in the array
                txnProcessedBuffer.shift();
            }
            // Add txn signature to buffer,
            txnProcessedBuffer.push(txn.signatures[0]);
        }
    
        let logs: string[] = txn.raw.meta.logMessages;
        let coder: BorshEventCoder = undefined;
        for(let i=0; i < logs.length; i++) {
            // Logs can be from Kyogen OR Structures, but NOT both.
            if(coder == undefined && logs[i].includes(programs.KYOGEN.toString())){
                coder = KyogenEventCoder;
            } else if (coder == undefined && logs[i].includes(programs.STRUCTURES.toString())){
                coder = StructuresEventCoder;
            } else if(logs[i].startsWith("Program data:")) {
                let logData = logs[i].slice(LOG_START_INDEX);
                if(coder){
                    const event = coder.decode(logData);
                    console.log(`Event: ${event.name}`);
                    console.log(`Data:\n${JSON.stringify(event.data, null, 2)}`);
                    // All events should have an "instance" that's used as "gameId"
                    const gameIdHex: string = event.data.instance as string;
                    if(!gameIdHex || gameIdHex == ""){
                        console.log("Event didn't have instance!");
                    } else {
                        let gameId = BigInt(`0x${gameIdHex}`).toString();
                        // Don't deserialize the events, just pass them onto clients with accounts
                        // This way the server has to do less work
                        //Kyogen Events
                        //Structures Events
                    }
                }
            }
        }
    } catch (e) {
        console.error(e);
    }
    // Otherwise, broadcast update to the channel
    // Events:
        // GameStateChanged
        // NewPlayer
        // SpawnClaimed
        // UnitSpawned
        // UnitMoved
        // UnitAttacked
        // Meteor Mined
        // PortalUsed
        // LootableLooted
        // GameFished
    res.code(200);
});





/**
 * Lastly, start the server
 */
server.listen({port: parseInt(process.env.PORT)}, (err, address) => {
    if (err){
        console.error(err);
        process.exit(1);
    }
    console.log(`Server listening at ${address}`);
});