// No imports needed: web3, anchor, pg and more are globally available
// TODO: Replace generated keypairs for token accounts with properly initialized SPL token accounts using the @solana/spl-token library.

describe("Token Swap", () => {
  it("Initialize Pool", async () => {
    // Generate keypair for the liquidity pool
    const poolAccountKp = new web3.Keypair();

    // Generate keypairs for token accounts
    const tokenAAccountKp = new web3.Keypair();
    const tokenBAccountKp = new web3.Keypair();

    // Generate keypair for the liquidity mint account
    const liquidityMintKp = new web3.Keypair();

    // Admin account (program's wallet public key)
    const admin = pg.wallet.publicKey;

    // Fee percentage as a number (u16)
    const feePercentage = 100; // Example: 1% fee

    // Admin fee account (optional PublicKey)
    const adminFeeAccount = admin;

    // Debugging logs
    console.log("Admin Public Key:", admin.toString());
    console.log("Pool Account Public Key:", poolAccountKp.publicKey.toString());
    console.log("Token A Account Public Key:", tokenAAccountKp.publicKey.toString());
    console.log("Token B Account Public Key:", tokenBAccountKp.publicKey.toString());
    console.log("Liquidity Mint Public Key:", liquidityMintKp.publicKey.toString());

    // Send transaction to initialize the pool
    const txHash = await pg.program.methods
      .initializePool(feePercentage, adminFeeAccount)
      .accounts({
        pool: poolAccountKp.publicKey,
        mintA: tokenAAccountKp.publicKey,
        mintB: tokenBAccountKp.publicKey,
        tokenA: tokenAAccountKp.publicKey,
        tokenB: tokenBAccountKp.publicKey,
        poolTokenA: tokenAAccountKp.publicKey,
        poolTokenB: tokenBAccountKp.publicKey,
        userLiquidityMint: liquidityMintKp.publicKey,
        admin,
        systemProgram: web3.SystemProgram.programId,
      })
      // Include all explicitly created keypairs as signers
      .signers([poolAccountKp, tokenAAccountKp, tokenBAccountKp, liquidityMintKp])
      .rpc();

    console.log(`Transaction successful with hash: ${txHash}`);

    // Confirm the transaction
    await pg.connection.confirmTransaction(txHash);

    // Fetch the created pool account
    const poolAccount = await pg.program.account.liquidityPool.fetch(
      poolAccountKp.publicKey
    );

    console.log("On-chain pool data is:", poolAccount);

    // Check the data on-chain matches the initialization parameters
    assert(poolAccount.tokenA.equals(tokenAAccountKp.publicKey), "Token A mismatch");
    assert(poolAccount.tokenB.equals(tokenBAccountKp.publicKey), "Token B mismatch");
    assert(poolAccount.mintA.equals(tokenAAccountKp.publicKey), "Mint A mismatch");
    assert(poolAccount.mintB.equals(tokenBAccountKp.publicKey), "Mint B mismatch");
    assert.equal(poolAccount.feePercentage, feePercentage, "Fee percentage mismatch");
    assert(poolAccount.adminFeeAccount.equals(admin), "Admin fee account mismatch");
    assert(poolAccount.totalLiquidity.eq(new anchor.BN(0)), "Total liquidity should be zero");
  });
});
