
import * as dotenv from 'dotenv';
dotenv.config();

import fastify from 'fastify';
import { readFileSync } from 'fs';
import fetch from 'node-fetch';

const server = fastify({
    http2: true,
    https: {
        key: readFileSync(process.env.SERVER_KEY),
        cert: readFileSync(process.env.SERVER_CERT),
    }
});

import {createSession, createChannel, Channel} from 'better-sse';

interface Game {
    id: string,
    channel: Channel,
    addresses: {
        kyogen_index: string,
        structures_index: string,
        units: Set<string>,
        structures: Set<string>,
    },
}

let gameChannels: Map<string, Game> = new Map();
let addressToGameIdMap: Map<string, string> = new Map();

server.listen({port: parseInt(process.env.PORT)}, (err, address) => {
    if (err){
        console.error(err);
        process.exit(1);
    }
    console.log(`Server listening at ${address}`);
});

server.get('/', async (req, res) => {
    res.code(200).send("Kyogen Server is working!");
})

server.get('/game/:gameId', async (req, res) => {
    const {gameId} = req.params as any;

    if(!gameChannels.has(gameId)){
        // Channel doesn't exist
        const kyogen_index = "";
        const structures_index = "";
        const units: Set<string> = new Set([]);
        const structures: Set<string> = new Set([]); 

        const hookId = await createHook(gameId, [kyogen_index, structures_index, ...Array.from(units), ...Array.from(structures)]);

        gameChannels.set(gameId, {
            id: gameId,
            channel: createChannel(),
            addresses: {
                kyogen_index,
                structures_index,
                units,
                structures
            },  
        })
    }

    // Channel now exists
    // create session then register it with channel
    const newUserSession = await createSession(req.raw, res.raw);
    // Sessions are automatically deregistered when they are disconnected
    gameChannels.get(gameId).channel.register(newUserSession);
});

async function createHook(gameId:string, addressList: string[]): Promise<string> {
    return "";
}

/**
 * Helius will hit this endpoint with payload
 */
server.post('/helius', async (req, res) => {
    
});