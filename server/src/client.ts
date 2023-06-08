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
    const instance = "6387869488554589888"
    const game = new EventSource(`${process.env.SERVER_ADDRESS}/game/${instance}`);
    console.log("Listening to events!");
    game.onmessage = (event) => {
        console.log(event);
        const parsed = JSON.parse(event.data);
    }
}

main();