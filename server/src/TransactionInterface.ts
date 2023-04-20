export interface Transaction {
    timestamp: string;
    fee: number;
    fee_payer: string;
    signers?: (string)[] | null;
    signatures?: (string)[] | null;
    protocol: SourceProtocolOrProtocol;
    type: string;
    status: string;
    actions?: (ActionsEntity)[] | null;
    raw: Raw;
    accounts?: (AccountsEntity)[] | null;
  }
  
  export interface SourceProtocolOrProtocol {
    address: string;
    name: string;
  }
  export interface ActionsEntity {
    info: Info;
    source_protocol: SourceProtocolOrProtocol;
    type: string;
  }
  export interface Info {
  }
  export interface Raw {
    blockTime: number;
    meta: Meta;
    slot: number;
    transaction: Transaction1;
    version: string;
  }
  export interface Meta {
    computeUnitsConsumed: number;
    err?: null;
    fee: number;
    innerInstructions?: (InnerInstructionsEntity)[] | null;
    logMessages?: (string)[] | null;
    postBalances?: (number)[] | null;
    postTokenBalances?: (null)[] | null;
    preBalances?: (number)[] | null;
    preTokenBalances?: (null)[] | null;
    rewards?: (null)[] | null;
    status: Status;
  }
  export interface InnerInstructionsEntity {
    index: number;
    instructions?: (InstructionsEntity)[] | null;
  }
  export interface InstructionsEntity {
    accounts?: (string)[] | null;
    data: string;
    programId: string;
  }
  export interface Status {
    Ok?: null;
  }
  export interface Transaction1 {
    message: Message;
    signatures?: (string)[] | null;
  }
  export interface Message {
    accountKeys?: (AccountKeysEntity)[] | null;
    addressTableLookups?: null;
    instructions?: (InstructionsEntity1)[] | null;
    recentBlockhash: string;
  }
  export interface AccountKeysEntity {
    pubkey: string;
    signer: boolean;
    source: string;
    writable: boolean;
  }
  export interface InstructionsEntity1 {
    accounts?: (string | null)[] | null;
    data: string;
    programId: string;
  }
  export interface AccountsEntity {
    address: string;
    owner: string;
    lamports: number;
    data: string;
  }
  