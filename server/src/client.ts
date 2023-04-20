import EventSource from 'eventsource';

async function main(){
    const game = new EventSource("https://aec0-207-229-153-241.ngrok-free.app/game/10371318924185010950");
    console.log("Listening to events!");
    game.onmessage = (event) => {
        console.log(event);
    }
}

main();