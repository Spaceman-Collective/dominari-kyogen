import { StatelessSDK } from '../../kyogen-sdk/kyogen-sdk-nodejs/kyogen_sdk';
import * as anchor from '@coral-xyz/anchor';
import * as dotenv from 'dotenv';
dotenv.config();

import fastify from 'fastify';
import fetch, { Headers } from 'node-fetch';
import cors from '@fastify/cors';

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
import * as Events from './IEvents';


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

server.register(cors, {
    origin: '*',
    methods: ['GET', 'PUT', 'POST', 'OPTIONS'],
}).then(() => {

    // middleware registration needs to be done first to apply to the follow endpoints
    // so maybe this will fix the cors problem be ensuring its registered as a pre-handler middleware
    // hook before attaching route handlers
    // should i restart the server?
    // yeah give that a shot

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

// TODO: Remove
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
        if(req.headers['x-api-key'] != process.env.SHYFT_KEY) {
            console.log("Callback headers are invalid.");
            res.code(400);
            return;
        } 
        
        const txn:Transaction = req.body as Transaction;
        console.log(JSON.stringify(txn, null, 2));
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
                    const gameIdHex: bigint = event.data.instance as bigint;
                    if(!gameIdHex || gameIdHex == 0n){
                        console.log("Event didn't have instance!");
                    } else {
                        let gameId = gameIdHex.toString();
                        console.log("Game ID: ", gameId);
                        let channel = gameChannels.get(gameId).channel;
                        // Don't deserialize the accounts, just pass them onto clients with structured json
                        // This way the server has to do less work

                        if(event.name == "GameStateChanged"){
                            let newState: Events.EventGameStateChanged = {
                                instance: gameId,
                                newState: Object.keys(event.data.newState)[0]
                            }
                            channel.broadcast(JSON.stringify({
                                name: event.name,
                                data: newState,
                            }))
                        } else if (event.name == "NewPlayer") {
                            let playerId = event.data.playerId.toString();
                            let playerAddress = sdk.fetch_address_by_id(BigInt(gameId), BigInt(playerId));

                            let newPlayer: Events.EventNewPlayer = {
                                instance: gameId,
                                player: {
                                    id: playerId,
                                    data: txn.accounts.find((acc) => acc.address == sdk.fetch_address_by_id(
                                            BigInt(gameId), 
                                            BigInt(playerId)
                                        )).data as string,
                                },
                                authority: event.data.authority as string,
                                clan: Object.keys(event.data.clan)[0]
                            };

                            // Update Hook with new Player Address
                            await updateHook(
                                gameId,
                                [
                                  playerAddress,
                                  ...flattenAddressListJSON(await sdk.fetch_addresses(BigInt(gameId)))
                                ]
                            )
                            channel.broadcast(JSON.stringify({
                                name: event.name,
                                data: newPlayer
                            }))
                        } else if (event.name == "SpawnClaimed") {
                            let player = event.data.playerId.toString();
                            let tile = event.data.tile.toString();

                            let spawnClaimed: Events.EventSpawnClaimed = {
                                instance: gameId,
                                clan: Object.keys(event.data.clan)[0],
                                tile: {
                                    id: tile,
                                    data: txn.accounts.find((acc) => acc.address == sdk.fetch_address_by_id(
                                        BigInt(gameId), 
                                        BigInt(tile)
                                        )).data as string,
                                },
                                player
                            }
                            channel.broadcast(JSON.stringify({
                                name: event.name,
                                data: spawnClaimed
                            }));
                        } else if (event.name == "UnitSpawned") {
                            let tile = event.data.tile.toString();
                            let player = event.data.tile.toString();
                            let unit = event.data.tile.toString();;
                            let unitAddress = sdk.fetch_address_by_id(BigInt(gameId), BigInt(unit));

                            let unitSpawned: Events.EventUnitSpawned = {
                                instance: gameId,
                                tile: {
                                    id: tile,
                                    data: txn.accounts.find((acc) => acc.address == sdk.fetch_address_by_id(
                                        BigInt(gameId), 
                                        BigInt(tile)
                                    )).data as string
                                },
                                player: {
                                    id: player,
                                    data: txn.accounts.find((acc) => acc.address == sdk.fetch_address_by_id(
                                        BigInt(gameId), 
                                        BigInt(player)
                                    )).data as string
                                },
                                unit: {
                                    id: unit,
                                    data: txn.accounts.find((acc) => acc.address == sdk.fetch_address_by_id(
                                        BigInt(gameId), 
                                        BigInt(unit)
                                    )).data as string
                                },
                            };

                            // Update Hook with new Player Address
                            await updateHook(
                                gameId,
                                [
                                    unitAddress,
                                    ...flattenAddressListJSON(await sdk.fetch_addresses(BigInt(gameId)))
                                ]
                            )

                            channel.broadcast(JSON.stringify({
                                name: event.name,
                                data: unitSpawned

                            }));

                        } else if (event.name == "UnitMoved") {
                            let unit = event.data.unit.toString();
                            let from = event.data.from.toString();
                            let to = event.data.to.toString();;

                            let unitMoved: Events.EventUnitMoved = {
                                instance: gameId,
                                unit: {
                                    id: unit,
                                    data: txn.accounts.find((acc) => acc.address == sdk.fetch_address_by_id(
                                        BigInt(gameId), 
                                        BigInt(unit)
                                    )).data as string
                                },
                                from: {
                                    id: from,
                                    data: txn.accounts.find((acc) => acc.address == sdk.fetch_address_by_id(
                                        BigInt(gameId), 
                                        BigInt(from)
                                    )).data as string
                                },
                                to: {
                                    id: to,
                                    data: txn.accounts.find((acc) => acc.address == sdk.fetch_address_by_id(
                                        BigInt(gameId), 
                                        BigInt(to)
                                    )).data as string
                                },
                            };

                            channel.broadcast(JSON.stringify({
                                name: event.name,
                                data: unitMoved

                            }));

                        } else if (event.name == "UnitAttacked") {
                            let attacker = event.data.attacker.toString();
                            let defender = event.data.defender.toString();
                            let tile = event.data.tile.toString();;

                            let unitAttacked: Events.EventUnitAttacked = {
                                instance: gameId,
                                attacker: {
                                    id: attacker,
                                    data: txn.accounts.find((acc) => acc.address == sdk.fetch_address_by_id(
                                        BigInt(gameId), 
                                        BigInt(attacker)
                                    )).data as string
                                },
                                defender: {
                                    id: defender,
                                    data: txn.accounts.find((acc) => acc.address == sdk.fetch_address_by_id(
                                        BigInt(gameId), 
                                        BigInt(defender)
                                    )).data as string
                                },
                                tile: {
                                    id: tile,
                                    data: txn.accounts.find((acc) => acc.address == sdk.fetch_address_by_id(
                                        BigInt(gameId), 
                                        BigInt(tile)
                                    )).data as string
                                },
                            };

                            channel.broadcast(JSON.stringify({
                                name: event.name,
                                data: unitAttacked
                            }));
                        } else if (event.name == "MeteorMined") {
                            let tile = event.data.tile.toString();
                            let meteor = event.data.meteor.toString();
                            let player = event.data.player.toString();

                            let meteorMined: Events.EventMeteorMined = {
                                instance: gameId,
                                meteor: {
                                    id: meteor,
                                    data: txn.accounts.find((acc) => acc.address == sdk.fetch_address_by_id(
                                        BigInt(gameId), 
                                        BigInt(meteor)
                                    )).data as string
                                },
                                player: {
                                    id: player,
                                    data: txn.accounts.find((acc) => acc.address == sdk.fetch_address_by_id(
                                        BigInt(gameId), 
                                        BigInt(player)
                                    )).data as string
                                },
                                tile,
                            };

                            channel.broadcast({
                                name: event.name,
                                data: meteorMined
                            });
                        } else if (event.name == "PortalUsed") {
                            let from = event.data.from.toString();
                            let to = event.data.to.toString();
                            let unit = event.data.unit.toString();;

                            let portalUsed: Events.EventPortalUsed = {
                                instance: gameId,
                                from: {
                                    id: from,
                                    data: txn.accounts.find((acc) => acc.address == sdk.fetch_address_by_id(
                                        BigInt(gameId), 
                                        BigInt(from)
                                    )).data as string
                                },
                                to: {
                                    id: to,
                                    data: txn.accounts.find((acc) => acc.address == sdk.fetch_address_by_id(
                                        BigInt(gameId), 
                                        BigInt(to)
                                    )).data as string
                                },
                                unit: {
                                    id: unit,
                                    data: txn.accounts.find((acc) => acc.address == sdk.fetch_address_by_id(
                                        BigInt(gameId), 
                                        BigInt(unit)
                                    )).data as string
                                },
                            };

                            channel.broadcast(JSON.stringify({
                                name: event.name,
                                data: portalUsed
                            }));  
                        } else if (event.name == "LootableLooted") {
                            let tile = event.data.tile.toString();
                            let lootable = event.data.lootable.toString();
                            let player = event.data.player.toString();;

                            let lootableUsed: Events.EventLootableLooted = {
                                instance: gameId,
                                tile,
                                lootable: {
                                    id: lootable,
                                    data: txn.accounts.find((acc) => acc.address == sdk.fetch_address_by_id(
                                        BigInt(gameId), 
                                        BigInt(lootable)
                                    )).data as string
                                },
                                player: {
                                    id: player,
                                    data: txn.accounts.find((acc) => acc.address == sdk.fetch_address_by_id(
                                        BigInt(gameId), 
                                        BigInt(player)
                                    )).data as string
                                },
                            };

                            channel.broadcast(JSON.stringify({
                                name: event.name,
                                data: lootableUsed
                            }));  

                        } else if (event.name == "GameFinished") {
                            let gameFinished: Events.EventGameFinished = {
                                instance: gameId,
                                winning_player_id: event.data.winningPlayerId.toString(),
                                winning_player_key: event.data.winningPlayerKey.toString(),
                                high_score: event.data.highScore.toString(),
                            }
                            channel.broadcast(JSON.stringify({
                                name: event.name,
                                data: gameFinished
                            }))
                        }                        
                    }
                }
            }
        }
    } catch (e) {
        console.error(e);
    }
    // Otherwise, broadcast update to the channel
    res.code(200);
});


/**
 * Lastly, start the server
 */

// server.register(cors, {
//     methods: ['GET', 'POST', 'PUT', 'DELETE'],
//     allowedHeaders: ['Content-Type'], // Allow these headers
//     credentials: true, // Allow cookies to be sent with CORS requests
// }).then(() => {
    server.listen({port: parseInt(process.env.PORT)}, (err, address) => {

        if (err){
            console.error(err);
            process.exit(1);
        }
        console.log(`Server listening at ${address}`);
    });
// })
})
