import * as dotenv from 'dotenv';
dotenv.config();
dotenv.config({ path: `.env.local`, override: true });
import { readFileSync, writeFileSync } from "fs";

const MAX_X = 10;
const MAX_Y = 10;
const MAX_PLAYERS = 6;
const METEOR_COUNT = 3;
const SPAWN_POINT_RATE = 0.03; // Per spawn; so if this is 0.05, it's 5% chance of Ancients, 5% for wildings, etc
const fileName = `configs/${MAX_X}x${MAX_Y}-${MAX_PLAYERS}-${Date.now()}.json`;
const MAX_SCORE = 4000;


main();
configChecker(fileName);

function main(){
    let config = {
        max_players: MAX_PLAYERS,
        game_token: "replace",
        spawn_claim_multiplier: 1.1,
        tokens_minted: 500000,
        max_score: MAX_SCORE,

        mapmeta: {
            max_x: MAX_X,
            max_y: MAX_Y,
        },

        spawns: [],
        structures: []
    }

    // Spawns
    let grid = generateGrid(MAX_X, MAX_Y);
    for(let row=0; row<grid.length; row++){
        for(let col=0; col<grid[row].length; col++){
            if(grid[row][col] == "Ancient") {
                config.spawns.push({
                    x: col,
                    y: row,
                    cost: 10,
                    clan: "Ancients"
                })
            } else if(grid[row][col] == "Creeper") {
                config.spawns.push({
                    x: col,
                    y: row,
                    cost: 10,
                    clan: "Creepers"
                })
            }  else if(grid[row][col] == "Wilding") {
                config.spawns.push({
                    x: col,
                    y: row,
                    cost: 10,
                    clan: "Wildings"
                })
            }  else if(grid[row][col] == "Synth") {
                config.spawns.push({
                    x: col,
                    y: row,
                    cost: 10,
                    clan: "Synths"
                })
            }
        } 
    }

    // Meteors
    for(let i=0; i<METEOR_COUNT; i++){
        let mx = Math.floor(Math.random()*MAX_X);
        let my = Math.floor(Math.random()*MAX_Y);

        // Check if spawn exists on those coordinates
        while(config.spawns.find((s) => s.x === mx && s.y == my)){
            mx = Math.floor(Math.random()*MAX_X);
            my = Math.floor(Math.random()*MAX_Y);        
        }
        
        config.structures.push({
            x: mx,
            y: my,
            structure_blueprint: "Meteor"
        })
    }

    writeFileSync(fileName, JSON.stringify(config,null,2));
}

type Spawn = 'Ancient' | 'Creeper' | 'Wilding' | 'Synth' | 'Empty';

function generateGrid(rows: number, columns: number): Spawn[][] {
    const grid: Spawn[][] = [];
    const totalSpawns = Math.floor((rows*columns)*SPAWN_POINT_RATE); // 10% of the Map should be spawns
    const spawnTypes: Spawn[] = ['Ancient', 'Creeper', 'Wilding', 'Synth'];
    const spawnCounts: Map<Spawn, number> = new Map();
    let remainingSpawns = totalSpawns * spawnTypes.length;

    // Initialize the grid with 'Empty' values.
    for (let i = 0; i < rows; i++) {
        const row: Spawn[] = [];
        for (let j = 0; j < columns; j++) {
            row.push('Empty');
        }
        grid.push(row);
    }

    // Randomly distribute the spawns.
    while (remainingSpawns > 0) {
        const row = Math.floor(Math.random() * rows);
        const col = Math.floor(Math.random() * columns);
        const spawnType = spawnTypes[Math.floor(Math.random() * spawnTypes.length)];

        if (grid[row][col] === 'Empty' && (!spawnCounts.has(spawnType) || spawnCounts.get(spawnType)! < totalSpawns)) {
            grid[row][col] = spawnType;
            spawnCounts.set(spawnType, (spawnCounts.get(spawnType) || 0) + 1);
            remainingSpawns--;
        }
    }

    return grid;
}


function configChecker(configName) {
    const CONFIG = JSON.parse(readFileSync(configName, {encoding:"utf-8"}));
    console.log("Spawns: ", CONFIG.spawns.length);
    console.log("\tAncients: ", CONFIG.spawns.filter((spawn) => spawn.clan == "Ancients").length)
    console.log("\tWildings: ", CONFIG.spawns.filter((spawn) => spawn.clan == "Wildings").length)
    console.log("\tCreepers: ", CONFIG.spawns.filter((spawn) => spawn.clan == "Creepers").length)
    console.log("\tSynths: ", CONFIG.spawns.filter((spawn) => spawn.clan == "Synths").length)
    console.log("Meteors:", CONFIG.structures.length);
    for(let m of CONFIG.structures) {
        if(CONFIG.spawns.find((s) => s.x === m.x && s.y == m.y)){
            console.log("Collision Found!")
            console.log("Meteor: ", m);
        }
    }        
}