import * as YAML from 'yaml';

const MAX_X = 20;
const MAX_Y = 20;
const SPAWN_POINT_RATE = 0.05;
const METEOR_RATE = 0.02; 
const MAX_PLAYERS = 32;

async function createArray(){
    for (let x=0; x<MAX_X; x++){
        for (let y=0; y<MAX_Y; y++) {
            let roll = Math.random();
            
        }
    }
}

interface Tile {
    x: number,
    y: number,
    structure: "empty" | "spawn_ancients" | "spawn_wildings" | "spawn_creepers" | "spawn_synths" | "meteor"
}