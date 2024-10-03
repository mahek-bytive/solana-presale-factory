import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { SolanaPresaleFactory } from '../target/types/solana_presale_factory';
import { assert } from 'chai';

describe('solana_presale_factory', () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.local();
  anchor.setProvider(provider);
  const program = anchor.workspace.SolanaPresaleFactory as Program<SolanaPresaleFactory>;

  let factory = anchor.web3.Keypair.generate();  // Create a new keypair for the factory account

  it('Initialize the Factory', async () => {
    const platformFee = new anchor.BN(500); // Set the platform fee (500 = 5%)

    // Call the initialize_factory function
    await program.methods.initializeFactory(platformFee)
      .accounts({
        factory: factory.publicKey,
        owner: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([factory])
      .rpc();

    // Fetch the factory account to verify its state
    const factoryAccount = await program.account.factory.fetch(factory.publicKey);

    // Assertions to verify correct initialization
    assert.equal(factoryAccount.owner.toString(), provider.wallet.publicKey.toString());
    assert.equal(factoryAccount.presaleCount.toNumber(), 0);
    assert.equal(factoryAccount.platformFee.toNumber(), 500);
  });
});
