import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Kyogen } from "../target/types/kyogen";

describe("kyogen", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Kyogen as Program<Kyogen>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
