import * as anchor from '@coral-xyz/anchor';
import { Connection, Keypair, PublicKey, SystemProgram } from '@solana/web3.js';
import { createMint, getOrCreateAssociatedTokenAccount, mintTo } from '@solana/spl-token';
import dotenv from 'dotenv';
import fs from 'fs';

dotenv.config();

const { RPC_URL, WALLET, PROGRAM_ID, VAULT } = process.env;
if (!RPC_URL || !WALLET || !PROGRAM_ID || !VAULT) {
  console.error('Missing env vars');
  process.exit(1);
}

const connection = new Connection(RPC_URL, 'confirmed');
const payer = Keypair.fromSecretKey(
  new Uint8Array(JSON.parse(fs.readFileSync(WALLET, 'utf8')))
);

const provider = new anchor.AnchorProvider(connection, new anchor.Wallet(payer), {
  preflightCommitment: 'confirmed',
});
anchor.setProvider(provider);

const programId = new PublicKey(PROGRAM_ID);
const idl = await anchor.Program.fetchIdl(programId, provider);
if (!idl) {
  console.error('IDL not found for program', programId.toBase58());
  process.exit(1);
}

const program = new anchor.Program(idl, programId, provider);

// Derive config PDA
const [configPda] = PublicKey.findProgramAddressSync([
  Buffer.from('config'),
], programId);

// Initialize config if missing
const configInfo = await connection.getAccountInfo(configPda);
if (!configInfo) {
  console.log('Initializing config');
  await program.methods
    .initializeConfig(new PublicKey(VAULT))
    .accounts({
      config: configPda,
      admin: payer.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
}

// Create mint record if missing
const recipient = payer.publicKey;
const nonce = Math.floor(Math.random() * 1e9);
const [recordPda] = PublicKey.findProgramAddressSync([
  Buffer.from('record'),
  recipient.toBuffer(),
  new anchor.BN(nonce).toArrayLike(Buffer, 'le', 8),
], programId);

const recordInfo = await connection.getAccountInfo(recordPda);
if (!recordInfo) {
  console.log('Creating mint record');
  await program.methods
    .createMintRecord(recipient, rarity(), new anchor.BN(nonce))
    .accounts({
      config: configPda,
      record: recordPda,
      admin: payer.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .rpc();
}

// Mint NFT into vault
const mintPubkey = await createMint(
  connection,
  payer,
  payer.publicKey,
  payer.publicKey,
  0
);
const vaultAta = await getOrCreateAssociatedTokenAccount(
  connection,
  payer,
  mintPubkey,
  new PublicKey(VAULT)
);
await mintTo(
  connection,
  payer,
  mintPubkey,
  vaultAta.address,
  payer,
  1
);

// Confirm mint in on-chain record
await program.methods
  .confirmMint(recipient, new anchor.BN(nonce), mintPubkey)
  .accounts({
    config: configPda,
    record: recordPda,
    admin: payer.publicKey,
    vaultAta: vaultAta.address,
  })
  .rpc();

console.log('Program ID:', programId.toBase58());
console.log('Config PDA:', configPda.toBase58());
console.log('Record PDA:', recordPda.toBase58());
console.log('Mint:', mintPubkey.toBase58());

function rarity() {
  return 0; // simple placeholder rarity
}
