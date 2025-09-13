# CityChests Vault

MVP Anchor program storing prize records and a CLI script to mint NFTs into a custodial vault.

## Setup
1. Install Solana CLI and Anchor CLI 0.30.x.
2. Run `anchor build` once to generate a new program id.
3. Update `declare_id!` in `programs/citychests_vault/src/lib.rs` and the `[programs.devnet]` section in `Anchor.toml` with that id.
4. Copy `.env.example` to `.env` and fill `PROGRAM_ID` and `VAULT`.
5. Install Node dependencies: `npm install`.

## Usage
Run the drop script which creates a record, mints an NFT to the vault and confirms it:

```sh
npm run drop
# or
node scripts/drop.mjs
```

The script logs the program id, config PDA, record PDA and mint address. After success the vault's ATA should show `amount = 1` and the on-chain record will have `minted = true`.
