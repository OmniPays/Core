# Module Documentation Index

Living docs, one per module. **Read the relevant doc before modifying a module; update it in the same change.** See "Module Documentation Convention" in `CLAUDE.md` for the workflow. All links below were verified on 2026-11-24.

Each Rust contract under `contracts/` is its own crate with its own `Cargo.lock`. There is no cargo workspace.

## Contracts and crates

| Doc | Module | One-liner | Code entry point |
|---|---|---|---|
| [factory.md](factory.md) | `contracts/factory/` | Deploys and registers SoroswapPair instances at deterministic addresses, holds the protocol fee switch. | `contracts/factory/src/lib.rs:268` (`create_pair`) |
| [factory-interface.md](factory-interface.md) | `contracts/factory-interface/` | Shared rlib crate with the `SoroswapFactoryTrait`, its generated client, and `FactoryError`. | `contracts/factory-interface/src/lib.rs:15` (`SoroswapFactoryTrait`) |
| [pair.md](pair.md) | `contracts/pair/` | Constant product AMM pool with a built in LP token, 0.3% swap fee, optional protocol fee. | `contracts/pair/src/lib.rs:267` (`swap`) |
| [router.md](router.md) | `contracts/router/` | User facing entrypoint for add/remove liquidity and multi hop swaps, with deadline and slippage checks. | `contracts/router/src/lib.rs:583` (`swap_exact_tokens_for_tokens`) |
| [library.md](library.md) | `contracts/library/` | Published `soroswap-library` crate plus deployable contract: token sorting, deterministic pair address, quote math. | `contracts/library/src/quotes.rs:46` (`get_amount_out`) |
| [token.md](token.md) | `contracts/token/` | Standard Soroban token used only as a test token, not part of the protocol. | `contracts/token/src/contract.rs:26` (`initialize`) |

## Tooling

| Doc | Module | One-liner | Code entry point |
|---|---|---|---|
| [deploy-tooling.md](deploy-tooling.md) | `scripts/`, `utils/`, `configs.json` | TypeScript install/deploy/initialize pipeline, address book, manual testing flows. | `scripts/deploy.ts:12` (`deployAndInitContracts`) |
| [address-api.md](address-api.md) | `api/`, `public/`, `vercel.json` | Express service on Vercel that serves the deployed contract addresses and token lists as JSON. | `api/server.js:40` (`/api/:network/:contractName`) |

## Suggested reading paths

- Integrators: [router.md](router.md) first, then [factory-interface.md](factory-interface.md) for the factory client and [pair.md](pair.md) for pool behavior.
- Pool or fee changes: [pair.md](pair.md), then [factory.md](factory.md) and [router.md](router.md) for the callers that depend on pool behavior.
- Quote or deterministic-address changes: [library.md](library.md), then [router.md](router.md) for the pinned published-library gotcha.
- Deployment and published addresses: [deploy-tooling.md](deploy-tooling.md), then [address-api.md](address-api.md).
