# Library Module

> **Living document.** Read this before modifying the module. Update it in the same change whenever the exported functions, the math, or the error enum change.

**Source:** `contracts/library/` · **Crate:** `soroswap-library` 2.0.0, `cdylib` + `rlib`, published to crates.io (`Cargo.toml:3`, `:12`, `:15`) · **Last verified:** 2026-11-24

## Purpose

Pure helper math and address derivation shared across the protocol: sorting a token pair, computing a pair's address without calling the factory, and the constant product quote formulas. It ships two ways, as a crate other contracts link against and as a deployable `SoroswapLibrary` contract exposing the same functions. Because `pair_for` reproduces the factory's salt off chain, a change to the salt or the sort order silently redirects every quote to the wrong address.

## Structure

| File | Purpose |
|---|---|
| `src/lib.rs` | Re-exports, `SoroswapLibraryTrait`, and the deployable `SoroswapLibrary` contract. |
| `src/tokens.rs` | `pair_salt`, `sort_tokens`, `pair_for`. |
| `src/reserves.rs` | `get_reserves_with_factory`, `get_reserves_with_pair`, and the pair WASM import. |
| `src/quotes.rs` | `quote`, `get_amount_out`, `get_amount_in`, `get_amounts_out`, `get_amounts_in`. |
| `src/math.rs` | `CheckedCeilingDiv`. |
| `src/error.rs` | `SoroswapLibraryError`. |
| `src/soroswap_pair.wasm` | Checked in copy of the pair WASM, overwritten by the build (`Makefile:12`). |
| `src/test/` | Suites: `quote`, `get`, `tokens` (`src/test.rs:121`). |

## Public surface

Exported from the crate (`src/lib.rs:14`) and mirrored as contract functions on `SoroswapLibrary` (`src/lib.rs:184`). All return `Result<_, SoroswapLibraryError>`.

| Function | Signature | Source |
|---|---|---|
| `sort_tokens` | `(token_a: Address, token_b: Address) -> (Address, Address)` | `src/tokens.rs:36` |
| `pair_for` | `(e: Env, factory: Address, token_a: Address, token_b: Address) -> Address` | `src/tokens.rs:64` |
| `get_reserves_with_factory` | `(e: Env, factory: Address, token_a: Address, token_b: Address) -> (i128, i128)` | `src/reserves.rs:22` |
| `get_reserves_with_pair` | `(e: Env, pair: Address, token_a: Address, token_b: Address) -> (i128, i128)` | `src/reserves.rs:54` |
| `quote` | `(amount_a: i128, reserve_a: i128, reserve_b: i128) -> i128` | `src/quotes.rs:17` |
| `get_amount_out` | `(amount_in: i128, reserve_in: i128, reserve_out: i128) -> i128` | `src/quotes.rs:46` |
| `get_amount_in` | `(amount_out: i128, reserve_in: i128, reserve_out: i128) -> i128` | `src/quotes.rs:90` |
| `get_amounts_out` | `(e: Env, factory: Address, amount_in: i128, path: Vec<Address>) -> Vec<i128>` | `src/quotes.rs:130` |
| `get_amounts_in` | `(e: Env, factory: Address, amount_out: i128, path: Vec<Address>) -> Vec<i128>` | `src/quotes.rs:172` |

## Fee and quote math

- `quote` = `amount_a * reserve_b / reserve_a` (`src/quotes.rs:28`), no fee, used only for liquidity ratios.
- `get_amount_out` (`src/quotes.rs:46`): `fee = ceil(amount_in * 3 / 1000)`, then `(amount_in - fee) * reserve_out / (reserve_in + amount_in - fee)` (`:60`). This matches the pair's own fee computation, which is what makes the K check pass.
- `get_amount_in` (`src/quotes.rs:90`): `ceil(reserve_in * amount_out * 1000 / ((reserve_out - amount_out) * 997)) + 1` (`:105`). It rejects `amount_out >= reserve_out` with `InsufficientLiquidity` (`:101`) instead of returning a negative amount or dividing by zero. The ceiling division and the extra `+ 1` both round in the pool's favor, so a round trip through `get_amount_out(get_amount_in(x))` will not return exactly `x`.
- Overflow in either quote function returns `Overflow` (307) instead of panicking (`:58`, `:108`).
- `get_amounts_out` walks the path forward pushing to the back (`:143`); `get_amounts_in` walks it backward pushing to the front (`:185`). Both require `path.len() >= 2` (`:136`, `:178`).

## Deterministic pair address

`pair_for` (`src/tokens.rs:64`) sorts the tokens, builds `salt = sha256(token_0.to_xdr() || token_1.to_xdr())` (`src/tokens.rs:23`), then returns `e.deployer().with_address(factory, salt).deployed_address()`. This must stay byte for byte identical to the factory's `Pair::salt` and its `with_current_contract` deploy. The approach is documented at `src/tokens.rs:52`, pointing to `paltalabs/deterministic-address-soroban`.

## Errors

`SoroswapLibraryError` (`src/error.rs:6`): `InsufficientAmount` 301, `InsufficientLiquidity` 302, `InsufficientInputAmount` 303, `InsufficientOutputAmount` 304, `InvalidPath` 305, `SortIdenticalTokens` 306, `Overflow` 307. Arithmetic that overflows in `get_amount_out` or `get_amount_in` now returns `Overflow` rather than panicking. The in-repo router consumes a different published crate version and re-maps 301 to 306 to 510 to 515 (`contracts/router/src/error.rs:62`); it does not know about 307 yet.

## Events

None. This module publishes no events.

## Dependencies

- `soroban-sdk` 22.0.0-rc.2.1 (`Cargo.toml:18`), resolved to 22.0.3 (`Cargo.lock:1133`). Every other crate in the repo is on 20.2.0.
- `num-integer` (`Cargo.toml:19`).
- Compile time import of `./src/soroswap_pair.wasm` for the reserves client (`src/reserves.rs:7`).
- Consumed by the router as the published crates.io version 0.3.0, not this source. See the router doc.

## Gotchas and invariants

- **This source is not what the router runs.** `contracts/router/Cargo.toml:23` pins `soroswap-library = "0.3.0"` from crates.io (`contracts/router/Cargo.lock:1117`), and the router calls `soroswap_library::get_reserves` (`contracts/router/src/lib.rs:84`), a name this crate does not export. Changes here reach the router only after `cargo publish` and a version bump there.
- **The SDK version is out of step.** 22.x here against 20.2.0 in factory, pair, router and token. The same source will not compile as a path dependency of those crates without alignment.
- `get_reserves_with_pair` (`src/reserves.rs:52`) does not verify that the given `pair` address actually corresponds to `token_a` and `token_b`. It sorts the tokens only to decide the return order. A caller passing an unrelated pair gets plausible looking garbage.
- The build copies `../pair/target/wasm32-unknown-unknown/release/soroswap_pair.wasm` over `./src/soroswap_pair.wasm` before compiling (`Makefile:12`). The copy checked into git is whatever was last committed, so a build without that copy step compiles against a stale pair ABI.
- `Makefile:9` builds `../token`, `../pair` and `../factory` first, and the tests import the *optimized* WASM (`src/test.rs:20`, `:26`), so `soroban contract optimize` must have run for those crates.
- `publish = true` (`Cargo.toml:12`) and the license is GPL-3.0 (`Cargo.toml:9`), unlike the repo root MIT license (`package.json:15`). This crate is released independently, see `README.md` in this directory for the publish steps.

## Testing

`make test` in `contracts/library` (`Makefile:5`). `src/test/tokens.rs` guards `sort_tokens` and `pair_for`, `src/test/quote.rs` the fee math, and `src/test/get.rs` the chained path helpers plus direct failure-path tests for overflow and reserve exhaustion in `get_amount_in` / `get_amount_out`.
