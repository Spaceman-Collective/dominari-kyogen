import EventSource from 'eventsource';
import { StatelessSDK } from '../../kyogen-sdk/kyogen-sdk-nodejs/kyogen_sdk';
import * as anchor from '@coral-xyz/anchor';
import * as dotenv from 'dotenv';
dotenv.config();

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


async function main(){
    const instance = "10371318924185010950"
    const game = new EventSource(`https://a080-208-78-215-79.ngrok-free.app/game/${instance}`);
    console.log("Listening to events!");
    game.onmessage = (event) => {
        console.log(event);
        const parsed = JSON.parse(event.data);
        if(parsed.name == "MeteorMined"){
            const player = parsed.data.player;
            const pID = player.id;
            const data = player.data;
            console.log("Data Len: ", Buffer.from(data, 'hex').length);
            console.log(sdk.fetch_address_by_id(BigInt(instance), BigInt(pID)));
            console.log(JSON.stringify(sdk.get_player_json(data, BigInt(pID)), null, 2));
        }
    }
}

main();