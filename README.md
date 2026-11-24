# OmniPays Core

OmniPays Core is a constant-product automated market maker (AMM) for Stellar Soroban. The protocol deploys a factory that registers liquidity pools at deterministic addresses, a pair contract that holds reserves and mints LP shares, and a router that is the safe user-facing entry point for liquidity and multi-hop swaps. A shared library crate provides quote math and pair address derivation, and a small Express API publishes the deployed addresses and token lists as JSON.

The Rust contracts live under `contracts/`; TypeScript deployment, seeding, address publishing and manual testing tooling live under `scripts/` and `utils/`. Protocol documentation is available at [docs.soroswap.finance](https://docs.soroswap.finance/).

## Architecture

| Component | Code | Documentation |
|---|---|---|
| Router — liquidity and multi-hop swaps, deadlines and slippage | `contracts/router/` | [router.md](docs/modules/router.md) |
| Factory — pair registration, fee switch, deterministic pair deployment | `contracts/factory/`, `contracts/factory-interface/` | [factory.md](docs/modules/factory.md), [factory-interface.md](docs/modules/factory-interface.md) |
| Pair — pool reserves, 0.3% swap fee, embedded LP token | `contracts/pair/` | [pair.md](docs/modules/pair.md) |
| Library — quote math and deterministic pair address derivation | `contracts/library/` | [library.md](docs/modules/library.md) |
| Test token — standard Soroban token used for local seeding | `contracts/token/` | [token.md](docs/modules/token.md) |
| Deploy and manual test tooling | `scripts/`, `utils/` | [deploy-tooling.md](docs/modules/deploy-tooling.md) |
| Address and token-list JSON API | `api/`, `public/`, `vercel.json` | [address-api.md](docs/modules/address-api.md) |
| Module documentation index | `docs/modules/` | [README.md](docs/modules/README.md) |

Each contract is an independent Rust crate with its own `Cargo.lock`; there is no Cargo workspace. Build order is `token`, `pair`, `factory`, `library`, `router`, as driven by `contracts/Makefile`.

## Prerequisites

- Git, `make`, `jq`, Docker and Docker Compose
- Node.js and Yarn 1.22.x
- Rust 1.85 with the `wasm32-unknown-unknown` target:
  ```bash
  rustup toolchain install 1.85.0 --profile minimal --target wasm32-unknown-unknown
  rustup override set 1.85.0
  ```
- The Stellar CLI (`soroban`) for `soroban contract optimize`, which `make build` calls
- A local `.env` copied from `.env.example` that supplies `SOROSWAP_ADMIN_SECRET_KEY`, `TEST_TOKENS_ADMIN_SECRET_KEY`, `TESTING_ACCOUNT_SECRET_KEY`, `TRUSTLINE_WALLET_SECRET_KEY` and, for mainnet, `MAINNET_RPC_URL`

Never commit `.env` or any secret key. `.soroban/` and `.env` are gitignored; `public/` is committed on purpose.

## Quickstart

1. Clone this repository and install the Node dependencies:

   ```bash
   git clone <repository-url>
   cd Core
   yarn
   ```

2. Create your environment file and fill in the values described above:

   ```bash
   cp .env.example .env
   ```

3. Start a local network and the Soroban preview container in separate terminals:

   ```bash
   bash scripts/quickstart.sh standalone   # or futurenet / testnet
   bash scripts/run.sh
   ```

4. Inside the preview container, build the contracts and run the deployment pipeline:

   ```bash
   cd contracts && make build && cd ..
   yarn build
   yarn deploy standalone
   ```

## Build and test

```bash
cd contracts && make build   # compile and optimize all five contracts
cd contracts && make test    # build, then cargo test for each crate
cd contracts && make fmt     # cargo fmt for each crate (check everywhere except token)
```

Per crate, use `make build` or `make test` from its directory. Router budget profiling is available with `cargo test budget -- --nocapture` from `contracts/router`.

To exercise the deployed contracts end to end (mint, add liquidity, swap, remove liquidity):

```bash
yarn build
yarn test standalone              # addresses from .soroban/
yarn test testnet public          # addresses from public/
```

`yarn test <network> [public]` is the only supported form; the optional second argument switches the address source. The entry point is `scripts/manual_testing/all.ts`. There is no `all.sh` script.

## Deploy and publish addresses

```bash
yarn build
yarn deploy <network>
yarn upload <network>
git add public && git commit -m "chore: publish <network> addresses"
```

Accepted networks are `mainnet`, `standalone`, `futurenet`, `testnet` and `testnet-public`. `yarn deploy` installs the WASM, deploys and initializes the factory and router, and **unconditionally seeds test tokens and liquidity on every network, including mainnet**. Do not run it against mainnet unless that is intentional.

`yarn upload <network>` backs up the current `public/` JSON files to a timestamped directory and copies the `.soroban/` outputs into `public/`; build first because it runs `dist/scripts/upload_addresses.js` directly. Vercel serves the committed `public/` files. For local serving instead, run `bash scripts/serve_with_docker.sh` and use `http://localhost:8010/api/tokens`, `/api/random_tokens` and `/api/<network>/<factory|router>`.

Other commands: `yarn trustline <network> <folder>`, `yarn seeTrustline <network>`.

## Deployed addresses

Addresses below are taken from the committed public JSON files; pair addresses are derived on chain and are not listed. Confirm values against `public/mainnet.contracts.json` and `public/testnet.contracts.json` before integrating.

| Network | Factory | Router |
|---|---|---|
| Mainnet | [`CA4HEQTL2WPEUYKYKCDOHCDNIV4QHNJ7EL4J4NQ6VADP7SYHVRYZ7AW2`](https://stellar.expert/explorer/public/contract/CA4HEQTL2WPEUYKYKCDOHCDNIV4QHNJ7EL4J4NQ6VADP7SYHVRYZ7AW2) | [`CAG5LRYQ5JVEUI5TEID72EYOVX44TTUJT5BQR2J6J77FH65PCCFAJDDH`](https://stellar.expert/explorer/public/contract/CAG5LRYQ5JVEUI5TEID72EYOVX44TTUJT5BQR2J6J77FH65PCCFAJDDH) |
| Testnet | [`CDP3HMUH6SMS3S7NPGNDJLULCOXXEPSHY4JKUKMBNQMATHDHWXRRJTBY`](https://stellar.expert/explorer/public/contract/CDP3HMUH6SMS3S7NPGNDJLULCOXXEPSHY4JKUKMBNQMATHDHWXRRJTBY) | [`CCJUD55AG6W5HAI5LRVNKAE5WDP5XGZBUDS5WNTIVDU7O264UZZE7BRD`](https://stellar.expert/explorer/public/contract/CCJUD55AG6W5HAI5LRVNKAE5WDP5XGZBUDS5WNTIVDU7O264UZZE7BRD) |

## Audit and security notes

The protocol was audited by OtterSec; the report is included at [audits/2024-02-22_soroswap_ottersec_audit.pdf](audits/2024-02-22_soroswap_ottersec_audit.pdf).

- Integrate through the router. Pair `deposit`, `swap` and `withdraw` do not call `require_auth` and assume the caller has already transferred the tokens in.
- Factory and router `initialize` are unauthenticated and first-caller-wins; verify the initialized instance before use.
- Deadlines passed to the router are Unix timestamps in seconds, not milliseconds.
- Deploy scripts submit real transactions to the selected network. Review `.env`, the target network and the seeding behavior before running them.
- The in-repo library crate is not the version the router links against; changes to it require publishing and bumping the router dependency.

## Contributing

Open an issue or pull request with the affected module. Before submitting, run `make fmt` and `make test` from `contracts/`, run the relevant Node checks, and update the corresponding living document under `docs/modules/` in the same change. Keep secrets out of commits and do not hand-edit files under `public/`.

## License

The repository is MIT licensed; see [LICENSE](LICENSE). The `contracts/library` crate is published separately under GPL-3.0.
