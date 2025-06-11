import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Stake } from "../target/types/stake";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getAccount } from "@solana/spl-token";
import { getAssociatedTokenAddressSync } from '@solana/spl-token';
import * as bs58 from 'bs58';

describe("stake", () => {
  const METADATA_PROGRAM_ID = new anchor.web3.PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');
  const provider = anchor.AnchorProvider.env();
 const privatekey="33YLSRPPbPu3EuzPxHd6bpHT7xzho39mcsVo1ctw1Vdwj9MFnBBRSBveFAKeaxdcxCTeUcbVig2n5ShqxFzAhpGe";
  const user = anchor.web3.Keypair.fromSecretKey(
    bs58.decode(privatekey)
  );
  anchor.setProvider(provider);
  const program = anchor.workspace.stake as Program<Stake>;
  // const user = anchor.web3.Keypair.generate();
const payer=anchor.web3.Keypair.generate();
  const mintkeypair = anchor.web3.Keypair.generate();


  let stakePda: anchor.web3.PublicKey;
  let vaultPda: anchor.web3.PublicKey;
  before(async () => {
    // await provider.connection.confirmTransaction(
    //   await provider.connection.requestAirdrop(
    //     mintkeypair.publicKey,
    //     10 * anchor.web3.LAMPORTS_PER_SOL
    //   )
    // );
    console.log("sada",payer.publicKey.toBase58());
   console.log(user.secretKey);
   console.log(user.publicKey);
    // await provider.connection.confirmTransaction(
    //   await provider.connection.requestAirdrop(
    //     payer.publicKey,
    //     1 * anchor.web3.LAMPORTS_PER_SOL
    //   )
    // );

    [stakePda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("client1"), payer.publicKey.toBuffer()],
      program.programId
    );

    [vaultPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("vault"), payer.publicKey.toBuffer()],
      program.programId
    );

    console.log("Stake PDA:", stakePda.toBase58());
    console.log("Vault PDA:", vaultPda.toBase58());
  });

  it("Initialize the staking program", async () => {
    const initTx = await program.methods
      .initialize()
      .accountsStrict({
        signer: payer.publicKey,
        pda: stakePda,
        vault: vaultPda,
        systemProgram: anchor.web3.SystemProgram.programId
      })
      .signers([payer])
      .rpc();

    console.log("Initialize transaction:", initTx);
    const pdaData = await program.account.stakeAccount.fetch(stakePda);
    console.log("Initial PDA data:", {
      points: pdaData.point.toString(),
      stakedAmount: pdaData.stakedAmount.toString(),
      owner: pdaData.owner.toBase58(),
      bump: pdaData.bump,
      lastUpdateAmount: pdaData.lastUpdateAmount.toString()
    });
  });
  it("create NFT", async () => {
    const metadata = {
      name: "Mint",
      symbol: "",
      uri:"https://devnet.irys.xyz/BNqgXCViZHFjL3Wxf1QS96UTeJCjTWT9jBeKEsS711LH"
    };
    const associatedTokenAccount = getAssociatedTokenAddressSync(
      mintkeypair.publicKey,
      user.publicKey
    );
    const [metadataAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        METADATA_PROGRAM_ID.toBuffer(),
        user.publicKey.toBuffer()
      ],
      METADATA_PROGRAM_ID
    );

    const [editionAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        METADATA_PROGRAM_ID.toBuffer(),
        user.publicKey.toBuffer(),
        Buffer.from("edition")
      ],
       METADATA_PROGRAM_ID
    );
  
    try {
      const tx = await program.methods.mintNft(
        metadata.name,
        metadata.symbol,
        metadata.uri
      ).accountsStrict({
        signer: user.publicKey,
        mintAccount: user.publicKey,
        metadataAccount: metadataAccount,
        editionAccount: editionAccount,
        associatedTokenAccount: associatedTokenAccount,
        tokenMetadataProgram: METADATA_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        associatedTokenProgram:ASSOCIATED_TOKEN_PROGRAM_ID
      }).signers([user]).rpc();
  
      console.log("NFT created successfully:", tx);
      
      // Verify the NFT was created
      
      // expect(Number(mintedToken.amount)).to.equal(1);
  
    } catch (error) {
      console.error("Error creating NFT:", error);
      throw error;
    }
  });
  it("Create token mint", async () => {
    const metadataProgramInfo = await provider.connection.getAccountInfo(METADATA_PROGRAM_ID);
    if (!metadataProgramInfo?.executable) {
      throw new Error("Token Metadata program is not executable");
    }
    const metadata = {
      name: "Test Token",
      symbol: "TEST",
      uri: "ex"
    };
    const [metadataAccount] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("metadata"),
        METADATA_PROGRAM_ID.toBuffer(),
        mintkeypair.publicKey.toBuffer()
      ],
      METADATA_PROGRAM_ID
    );

    console.log("Metadata Account:", metadataAccount.toBase58());
    console.log("Mint Account:", mintkeypair.publicKey.toBase58());

    const transaction = await program.methods
      .createTokenMint(9, metadata.name, metadata.symbol, metadata.uri)
      .accountsStrict({
        payer: user.publicKey,
        metadataAccount: metadataAccount,
        mintAccount: mintkeypair.publicKey,
        tokenMetadata: METADATA_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY
      })
      .signers([user, mintkeypair])
      .rpc();

    console.log("Token mint created:", transaction);
  });

  it("Stake SOL", async () => {
    const stakeAmount = new anchor.BN(1);
    const assoicatedaccount=getAssociatedTokenAddressSync(new anchor.web3.PublicKey("AuzCK8jdZQ9Dvud9DnFbZ8KuUeuhqZJ2BDoAwzGEmEWd"),payer.publicKey)
    const stakeTx = await program.methods
      .stake(stakeAmount)
      .accountsStrict({
        signer: payer.publicKey,
        pda: stakePda,
        vault: vaultPda,
        payer:user.publicKey,
        associatedTokenAccount:assoicatedaccount,
              mintAccount:new anchor.web3.PublicKey("AuzCK8jdZQ9Dvud9DnFbZ8KuUeuhqZJ2BDoAwzGEmEWd"),
               associatedTokenProgram:ASSOCIATED_TOKEN_PROGRAM_ID,
               tokenProgram:TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId
      })
      .signers([payer,user])
      .rpc();

    console.log("Stake transaction:", stakeTx);
    const pdaData = await program.account.stakeAccount.fetch(stakePda);
    console.log("After stake PDA data:", {
      stakedAmount: pdaData.stakedAmount.toString()
    });
  });

  it("Get points", async () => {
    const getPointsTx = await program.methods
      .getPoints()
      .accountsStrict({
        user: payer.publicKey,
        pdaAccount: stakePda
      })
      .signers([payer])
      .rpc();

    console.log("Get points transaction:", getPointsTx);
  });
  it("Mint token", async () => {
    const assoicatedaccount=getAssociatedTokenAddressSync(new anchor.web3.PublicKey("FCj3EhebxZuV4cyTfLj1SjrdpHqpC1HV5CFt4rMdZyCq"),user.publicKey)
    
    console.log("payer",payer.publicKey.toBase58());
    console.log("dasd",payer.secretKey.toString());
    const getPointsTx = await program.methods
      .mintToken(new anchor.BN(3))
      .accountsStrict({
        payer:user.publicKey,
        recipent:payer.publicKey,
        associatedTokenAccount:assoicatedaccount,
        mintAccount:new anchor.web3.PublicKey("FCj3EhebxZuV4cyTfLj1SjrdpHqpC1HV5CFt4rMdZyCq"),
         associatedTokenProgram:ASSOCIATED_TOKEN_PROGRAM_ID,
         tokenProgram:TOKEN_PROGRAM_ID,
         systemProgram:anchor.web3.SystemProgram.programId
      })
      .signers([user,payer])
      .rpc();

    console.log("Get points transaction:", getPointsTx);
    console.log("payer",payer.publicKey);
    console.log("dasd",payer.secretKey.toString());
  });

  it("Claim points", async () => {
    const claimPointsTx = await program.methods
      .claimPoints()
      .accountsStrict({
        user: payer.publicKey,
        pdaAccount: stakePda
      })
      .signers([payer])
      .rpc();

    console.log("Claim points transaction:", claimPointsTx);
    const pdaData = await program.account.stakeAccount.fetch(stakePda);
    console.log("After claiming points PDA data:", {
      points: pdaData.point.toString()
    });
  });
  it("Mint nft",async()=>{
    const nftmint=anchor.web3.Keypair.generate();
    const [metadataAccount] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("metadata"),
        METADATA_PROGRAM_ID.toBuffer(),
        nftmint.publicKey.toBuffer()
      ],
      METADATA_PROGRAM_ID
    );
    const metadataProgramInfo = await provider.connection.getAccountInfo(METADATA_PROGRAM_ID);
    const metadata = {
      name: "Test Token",
      symbol: "TEST",
      uri: "ex"
    };
    const transaction=await program.methods.createTokenMint(0,metadata.name,metadata.symbol,metadata.uri).accountsStrict({
      payer: user.publicKey,
        metadataAccount: metadataAccount,
        mintAccount: nftmint.publicKey,
        tokenMetadata: METADATA_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY
    }).signers([user,nftmint]).rpc()
    console.log("tzxjn",transaction);
  })

  it("Unstake SOL", async () => {
    const unstakeAmount = new anchor.BN(1);
    const assoicatedaccount=getAssociatedTokenAddressSync(new anchor.web3.PublicKey("AuzCK8jdZQ9Dvud9DnFbZ8KuUeuhqZJ2BDoAwzGEmEWd"),payer.publicKey)
    const unstakeTx = await program.methods
      .unstake(unstakeAmount)
      .accountsStrict({
        signer: payer.publicKey,
        pda: stakePda,
        vault: vaultPda,
        systemProgram: anchor.web3.SystemProgram.programId,
        payer:user.publicKey,
        associatedTokenAccount:assoicatedaccount,
        mintAccount:new anchor.web3.PublicKey("AuzCK8jdZQ9Dvud9DnFbZ8KuUeuhqZJ2BDoAwzGEmEWd"),
        associatedTokenProgram:ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram:TOKEN_PROGRAM_ID,
      })
      .signers([payer, user])
      .rpc();

    console.log("Unstake transaction:", unstakeTx);
    const pdaData = await program.account.stakeAccount.fetch(stakePda);
    console.log("After unstake PDA data:", {
      stakedAmount: pdaData.stakedAmount.toString()
    });
  });
});