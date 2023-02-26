import * as anchor from '@coral-xyz/anchor';
import { randomBytes } from 'crypto';

export const ixWasmToJs = (ix: any):anchor.web3.TransactionInstruction => {
    return new anchor.web3.TransactionInstruction({
        programId: new anchor.web3.PublicKey(ix.program_id as Uint8Array), 
        keys: ix.accounts.map((account:any) => {
            account.isSigner = account.is_signer,
            account.isWritable = account.is_writable,
            account.pubkey = new anchor.web3.PublicKey(account.pubkey as Uint8Array);
            return account;
        }),
        data: ix.data as Buffer,
    });
}

export const ixPack = async (ixs: anchor.web3.TransactionInstruction[]): Promise<anchor.web3.TransactionInstruction[][]> => {
    const dummyKey = new anchor.web3.Keypair();
    let ixGroupArray: anchor.web3.TransactionInstruction[][] = [];
    let ixBuffer:anchor.web3.TransactionInstruction[] = [];
    for(let ix of ixs){
        ixBuffer.push(ix);

        let tempTx = new anchor.web3.Transaction();
        tempTx.add(...ixBuffer);
        tempTx.feePayer = dummyKey.publicKey;
        tempTx.recentBlockhash = dummyKey.publicKey.toBase58(); //doesn't matter, just a dummy hash
        
        if(tempTx.serializeMessage().length > 800){
            ixGroupArray.push(ixBuffer);
            ixBuffer = [];
        }
    }

    // Any leftover ix
    if(ixBuffer.length > 0){
        ixGroupArray.push(ixBuffer)
    }

    return ixGroupArray;
}

export const randomU64 = ():bigint => {
    return BigInt(`0x${randomBytes(8).toString("hex")}`);
}