import * as dotenv from 'dotenv';
dotenv.config();
dotenv.config({ path: `.env.local`, override: true });
import { writeFileSync } from "fs";

const MAX_X = 20;
const MAX_Y = 20;

main();
function main(){
    let config = {
        max_players: 32,
        game_token: "replace",
        spawn_claim_multiplier: 1.1,
        tokens_minted: 50000,
        max_score: 5000,

        mapmeta: {
            max_x: MAX_X,
            max_y: MAX_Y,
        },

        spawns: []
    }
    let grid = generateGrid(20, 20);
    //console.log(grid);
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
    writeFileSync(`configs/${Date.now()}-Config.json`, JSON.stringify(config,null,2));
}

function findIndexOfSmallest(numbers: number[]): number {
    let smallestIndex = 0;
    let smallestValue = numbers[0];

    for (let i = 1; i < numbers.length; i++) {
        if (numbers[i] < smallestValue) {
            smallestValue = numbers[i];
            smallestIndex = i;
        }
    }

    return smallestIndex;
}

type Spawn = 'Ancient' | 'Creeper' | 'Wilding' | 'Synth' | 'Empty';

function generateGrid(rows: number, columns: number): Spawn[][] {
    const grid: Spawn[][] = [];
    const totalSpawns = 25;
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

const rows = 20;
const columns = 20;
const grid = generateGrid(rows, columns);



/*
    // Equally distrbute spawns
    let distance_between_spawns = Math.floor((MAX_X*MAX_Y) / 4);
    let distance_since_last_spawn=distance_between_spawns; //first tile should be a spawn
    let spawns = [];
    let spawns_indexes = [0, 0, 0, 0]; // A W C S

    for(let x=0; x<MAX_X; x++){
        for(let y=0; y<MAX_Y; y++){
            if(distance_since_last_spawn >= distance_between_spawns) {
                // Spawn Something
                let fewest_spawns = findIndexOfSmallest(spawns_indexes);
                let clan = "";
                switch (fewest_spawns) {
                    case 0: 
                        clan = "Ancients"
                        break;
                    case 1:
                        clan = "Wildings"
                        break;
                    case 2:
                        clan = "Creepers"
                        break;
                    case 4: 
                        clan = "Synths"
                        break;
                }

                spawns.push({
                    x,
                    y,
                    cost: 10,
                    clan
                })
                distance_since_last_spawn =0;
            } else {
                distance_since_last_spawn +=1;
            }
        }
    }
*/