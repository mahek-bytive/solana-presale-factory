import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaPresaleFactory } from '../target/types/solana_presale_factory';
import { assert } from 'chai';
import { createMint, getOrCreateAssociatedTokenAccount, getMint, transfer } from "@solana/spl-token";

describe('solana-presale-factory', () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.SolanaPresaleFactory as Program<SolanaPresaleFactory>;

  console.log("Program ID:", program.programId.toString());

  let factory = anchor.web3.Keypair.generate();  // Create a new keypair for the factory account
  let presale = anchor.web3.Keypair.generate();  // Create a new keypair for the presale account
  let presaleVault = anchor.web3.Keypair.generate();  // Create a new keypair for the presale vault
  let tokenMint = null;
  let tokenVault = null;

  it('Initialize the Factory', async () => {
    const platformFee = new anchor.BN(500); // Set the platform fee (500 = 5%)

    // Call the initialize_factory function
    const txSignature = await program.methods.initializeFactory(platformFee)
      .accounts({
        factory: factory.publicKey,
        owner: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([factory])
      .rpc();
    

    console.log("Transaction Signature:", txSignature);

    // Fetch the factory account to verify its state
    console.log("factory Public Key: ", factory.publicKey);
    const factoryAccount = await program.account.factory.fetch(factory.publicKey);

    // Assertions to verify correct initialization
    assert.equal(factoryAccount.owner.toString(), provider.wallet.publicKey.toString());
    assert.equal(factoryAccount.presaleCount.toNumber(), 0);
    assert.equal(factoryAccount.platformFee.toNumber(), 500);

    console.log("Factory initialized successfully:", factoryAccount);
  });

  it('Create Presale', async () => {
    tokenMint = await createMint(
      provider.connection,          // Connection to the Solana network
      provider.wallet.payer,        // Payer of the transaction
      provider.wallet.publicKey,    // Mint authority (can mint new tokens)
      null,                         // Freeze authority (optional)
      9                             // Decimals for the token
    );
    // Define presale parameters
    const _owner = provider.wallet.publicKey;
    const _token = tokenMint,
    const _payment_token = anchor.web3.Keypair.generate().publicKey; // Mock payment token address
    const _dex_router = anchor.web3.Keypair.generate().publicKey; // Mock dex router address
    const _presale_rate = new anchor.BN(100);
    const _soft_cap = new anchor.BN(50000);
    const _hard_cap = new anchor.BN(100000);
    const _min_buy = new anchor.BN(100);
    const _max_buy = new anchor.BN(1000);
    const _start_sale = new anchor.BN(Math.floor(Date.now() / 1000)); // Current time
    const _end_sale = new anchor.BN(Math.floor(Date.now() / 1000) + 86400); // 24 hours from now
    const _liquidity_percent = new anchor.BN(50);
    const _is_fund = false;
    const _is_native = true;
    const _is_whitelist = false;
    const _is_auto_listing = false;
    const _is_vesting = false;
    const _first_release_percent = new anchor.BN(0);
    const _vesting_period = new anchor.BN(0);
    const _tokens_release_percent = new anchor.BN(0);
    const _listing_rate = new anchor.BN(100);
    const _demy_address = anchor.web3.Keypair.generate().publicKey; // Mock demy address
    const _liquidity_time = new anchor.BN(3600);
    const _qerralock = anchor.web3.Keypair.generate().publicKey; // Mock qerralock address
    const _uniswap_factory = anchor.web3.Keypair.generate().publicKey; // Mock uniswap factory address

    tokenVault = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      tokenMint,
      provider.wallet.publicKey
    );

    // Call the create_presale function
    const txSignature = await program.methods.createPresale(
      _owner,
      _token,
      _payment_token,
      _dex_router,
      _presale_rate,
      _soft_cap,
      _hard_cap,
      _min_buy,
      _max_buy,
      _start_sale,
      _end_sale,
      _liquidity_percent,
      _is_fund,
      _is_native,
      _is_whitelist,
      _is_auto_listing,
      _is_vesting,
      _first_release_percent,
      _vesting_period,
      _tokens_release_percent,
      _listing_rate,
      _demy_address,
      _liquidity_time,
      _qerralock,
      _uniswap_factory,
    )
      .accounts({
        factory: factory.publicKey,
        presale: presale.publicKey,
        owner: provider.wallet.publicKey,
        presaleVault: presaleVault.publicKey,
        tokenVault: tokenVault.address,
        tokenMint: tokenMint,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([presale])
      .rpc();

    const presaleAccount = await program.account.factory.presale.fetch(presale.publicKey);
    console.log("Presale Account fetched successfully:", presaleAccount);

    // Assertions to verify correct initialization of the presale
    assert.equal(presaleAccount.owner.toString(), _owner.toString());
    assert.equal(presaleAccount.token.toString(), _token.toString());
    assert.equal(presaleAccount.paymentToken.toString(), _payment_token.toString());
    assert.equal(presaleAccount.dexRouter.toString(), _dex_router.toString());
    assert.equal(presaleAccount.presaleRate.toNumber(), _presale_rate.toNumber());
    assert.equal(presaleAccount.softCap.toNumber(), _soft_cap.toNumber());
    assert.equal(presaleAccount.hardCap.toNumber(), _hard_cap.toNumber());
    assert.equal(presaleAccount.minBuy.toNumber(), _min_buy.toNumber());
    assert.equal(presaleAccount.maxBuy.toNumber(), _max_buy.toNumber());
    assert.equal(presaleAccount.startSale.toNumber(), _start_sale.toNumber());
    assert.equal(presaleAccount.endSale.toNumber(), _end_sale.toNumber());
    assert.equal(presaleAccount.liquidityPercent.toNumber(), _liquidity_percent.toNumber());
    assert.equal(presaleAccount.isFund, _is_fund);
    assert.equal(presaleAccount.isNative, _is_native);
    assert.equal(presaleAccount.isWhitelist, _is_whitelist);
    assert.equal(presaleAccount.isAutoListing, _is_auto_listing);
    assert.equal(presaleAccount.isVesting, _is_vesting);
    assert.equal(presaleAccount.firstReleasePercent.toNumber(), _first_release_percent.toNumber());
    assert.equal(presaleAccount.vestingPeriod.toNumber(), _vesting_period.toNumber());
    assert.equal(presaleAccount.tokensReleasePercent.toNumber(), _tokens_release_percent.toNumber());
    assert.equal(presaleAccount.listingRate.toNumber(), _listing_rate.toNumber());
    assert.equal(presaleAccount.demyAddress.toString(), _demy_address.toString());
    assert.equal(presaleAccount.liquidityTime.toNumber(), _liquidity_time.toNumber());
    assert.equal(presaleAccount.qerralock.toString(), _qerralock.toString());
    assert.equal(presaleAccount.uniswapFactory.toString(), _uniswap_factory.toString());
    assert.equal(presaleAccount.tokensSold.toNumber(), 0);
    assert.equal(presaleAccount.fundsRaised.toNumber(), 0);
    assert.equal(presaleAccount.isFinalized, false);
  });

  it('Buy Tokens', async () => {
    // const buyer = provider.wallet.publicKey;
    const amountToBuy = new anchor.BN(1000); // Amount in SOL or SPL tokens

    // Create buyer's associated token account for the presale token
    const buyerTokenAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      tokenMint,
      buyer
    );

    console.log("Buyer Token Account Address:", buyerTokenAccount.address.toString());

    // // Transfer some tokens to the buyer's token account for the test
    // const mintToBuyerAccountTx = await program.methods.mintTo(buyer, amountToBuy.toNumber())
    //   .accounts({
    //     mint: tokenMint,
    //     to: buyerTokenAccount.address,
    //     authority: provider.wallet.publicKey,
    //   })
    //   .rpc();

    // console.log("Minted tokens to buyer's account:", mintToBuyerAccountTx);

    // Now call the buy_tokens method
    const txSignature = await program.methods.buyTokens(amountToBuy.toNumber())
      .accounts({
        presale: presale.publicKey,
        buyer: provider.wallet.publicKey,
        presaleVault: presaleVault.publicKey,
        presalePaymentVault: presaleVault.publicKey, // Assuming payment vault is the same for simplicity
        buyerPaymentAccount: buyerTokenAccount.address, // This would typically be an SPL token account if not native
        buyerTokenAccount: buyerTokenAccount.address,
        tokenVault: tokenVault.address,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([provider.wallet.payer])
      .rpc();

    console.log("Tokens bought successfully. Amount:", amountToBuy.toString());

    // Fetch the updated presale account
    const updatedPresaleAccount = await program.account.presale.fetch(presale.publicKey);
    assert.equal(updatedPresaleAccount.tokensSold.toNumber(), amountToBuy.toNumber());
    assert.equal(updatedPresaleAccount.fundsRaised.toNumber(), amountToBuy.toNumber());

    console.log("Updated presale state:", updatedPresaleAccount);
  });
});

