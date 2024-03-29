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
5. Add `.env.local` and set local `PRIVATE_KEY_PATH`
6. (Initalize Programs) Run `ts-node 01_InitPrograms.ts`
7. (Create a game) Run `ts-node 02_SetupGame.ts`

## Example ENV:
COREDS_ID = "GN5Ww5qa8ej4evFCJxMhV6AFEPKhD1Drdu8qYYptVgDJ"
REGISTRY_ID = "7Vpu3mY18uA2iWBhAyKc72F9xs1SaMByV5KaPpuLhFQz"
KYOGEN_ID = "CTQCiB97LrAjAtHy1eqGwqGiy2mjefBXR762nrDhWYTL"
STRUCTURES_ID = "4Bo4cgr4RhGpXJsQUV4KENCf3HJwPveFsPELJGGN9GkR"
PRIVATE_KEY_PATH = "<keypath>"
CONNECTION_URL = "http://127.0.0.1:8899"