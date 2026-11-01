# CPU Backend Implementation Plan

**Goal:** Add a usable, statically dispatched CPU execution backend under `lib/rsph/src` without choosing or duplicating physical SPH equations.

**Architecture:** `Config` selects `Math`, `Physics`, `Neighbors`, and `Backend`. Shared `Physics<M>` computes local and pair rates and advances a particle. CPU execution evaluates all rates from the same old particle state and swaps reusable output buffers. CPU neighbor strategies own their index; arithmetic and vector contracts remain independent of the backend.

**Tech stack:** Rust 2024, existing `num-traits` dependency, no unsafe or new dependencies.

**Spec:** The design discussion in this chat and the request to implement the CPU specialization in `lib/rsph/src`.

## Constraints and decisions

- Work in the requested checkout; preserve all existing staged changes, manifests, and application sources. Do not commit or stage files.
- Supply sequential CPU execution, packed and aligned vectors, and `f32`/`f64` support.
- Provide exhaustive all-pairs and fixed-radius reference neighbor strategies; optimized spatial indexing can implement the same trait later.
- Physical equations and the numerical update rule are user-supplied through `Physics`; no claim to supply a complete classical or relativistic SPH model.
- Reject nonpositive or nonfinite time steps before updating particles.
- A fixed-radius query requires a positive finite radius with a finite nonzero representable square.

## Tasks

- [x] Numeric layer: common `Real`, `Vector3`, `Math`; `CpuMath`, packed `Vec3`, `AlignedVec3`; verify f32/f64 arithmetic and layouts.
- [x] Shared API and CPU implementation: `Config`, `Setup`, `Params`, `Physics`, `Execute`, `Solver`, `Cpu`, CPU state and neighbor strategies.
- [x] Consumer tests first: simultaneous updates over two steps, f32/f64 and packed/aligned vectors, radius boundary and coincident particles, moving neighbor positions, empty particles, invalid step and neighbor parameters.
- [x] Run crate and root tests, rustfmt and Clippy; request an independent review and address material issues.

## Review focus

- Pair updates must observe the old state, including when particles have nonzero velocities.
- Rebuilt neighbor indexes must follow moving particles on each step.
- Neighbor traversal must exclude self and retain distinct coincident particles.
- Parameter errors must leave the particle state unchanged.
- Backend-specific bounds must reject unsupported configurations without dynamic dispatch.

## Progress

- Baseline: library source directory empty; Cargo cannot load the dependency because it has no target yet.
- Numeric task delegated with tests before implementation; source files created only in the requested directory.
- Shared API and CPU consumer tests first failed with missing imports, then passed after implementation (15/15).
- Added parameter mutation checks; 17/17 tests pass.
- Clippy reported complexity of a four-component PhantomData tuple. Split it into individually named zero-sized selector fields; no behavioral change.
- Final verification: 17 unit tests and one compile-fail doctest pass. Root `cargo test --offline --locked`, library rustfmt check, and root/library Clippy with `-D warnings` pass.
- Independent read-only review: no material findings or deferred minor issues. Scope exclusions remain GPU compilation, concrete SPH equations, multistage integration, optimized spatial indexing and SIMD guarantees; those have not been implemented or claimed.
