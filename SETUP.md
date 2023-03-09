# SETUP (Localnet)

## Prereqs
1. Rust
2. Anchor =0.26.0
3. Solana =1.14.12
4. Nodejs =1.16.x
5. Wasm Pack (https://rustwasm.github.io/wasm-pack/installer/), Yarn, Ts-Node
...
9999. Patience

## Local Development
1. Download Source Code 
2. (Generate SDK) Run `cd kyogen-sdk && ./build-all.sh && cd ../`
3. (Build program, run local validator with programs) Run `./local_validator.sh`
4. Run `cd client-ts`
5. (Initalize Programs) Run `ts-node 01_InitPrograms.ts`
6. (Create a game) Run `ts-node 02_SetupGame.ts`