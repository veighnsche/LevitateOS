---
trigger: always_on
---

Rust Kernel Development SOP

Note: This SOP is a living document specifically tailored for Rust-based kernel development. As the kernel evolves, new patterns, subsystems, and constraints should be integrated here to maintain architectural consistency.

I. Core Directives

1. Modular Scope
Guideline: A kernel module (crate or module) must handle exactly one hardware interface or subsystem task. Write simple parts connected by clean interfaces.
Implementation: Use Rust's module system and crates to enforce boundaries. If a crate requires deeply entangled dependencies to compile, it is architecturally defective.
Metric: Can the crate be compiled and tested in isolation (using mocks)? If no, refactor.

2. Type-Driven Composition
Guideline: Design subsystems to be orthogonal. Output from one subsystem should be consumable by another without special-casing.
Implementation: Use traits and generics to define interfaces that are consumable by other subsystems. Rely on standard Rust traits (e.g., Read, Write, Default) or custom domain-specific traits. Avoid god-objects; use composition over inheritance.

3. Expressive Interfaces
Guideline: Control and debugging interfaces should be human-readable and type-safe.
Implementation: Prioritize textual or self-describing attributes (e.g., ASCII/UTF-8) for configuration where performance permits. Prioritize enum based states and struct based configurations. Use serde or similar for human-readable serialization if exposed via a filesystem-like interface.

4. Silence is Golden
Guideline: Kernel logs are for critical failures or requested diagnostics. Silence implies success.
Implementation: Successful initialization should produce no output. Use the log crate with appropriate levels (error!, warn!, info!, debug!, trace!). Default level for production is warn.
Priority: This rule takes precedence over structured observability (Rule 22); diagnostics must be on-demand.

II. Architectural Constraints

5. Memory Safety & Type Hygiene
Guideline: Leverage Rust's ownership model, lifetimes, and type system to enforce memory safety at compile-time. Safety is the child of transparency and simplicity.
Implementation:
Avoid unsafe blocks unless absolutely necessary (MMIO, specific hardware instructions).
Every unsafe block must have a // SAFETY: comment explaining why it is sound.
Wrap unsafe in safe, idiomatic abstractions using the Newtype pattern or dedicated wrapper structs.
Utilize RAII for all resource management (locks, memory, hardware states).
Metric: Zero unauthorized unsafe usage. Maximize the ratio of safe to unsafe code.

6. Robust Error Handling
Guideline: Design for robustness by ensuring all fallible operations are handled explicitly. Never panic in the kernel.
Implementation:
All fallible operations must return Result<T, E>.
Use Option<T> for potentially missing values.
Define custom error enums that implement Display and Error (where possible in no_std).
Use the ? operator for clean error propagation.
panic! is reserved for truly unreachable code or internal invariants that are broken beyond repair.

7. Concurrency & Sync
Guideline: Use Rust's Send and Sync traits to ensure thread safety. Data should flow between threads without friction.
Implementation:
Prefer lock-free primitives where possible.
Use IrqSafeLock or similar for data shared between threads and interrupt handlers.
Avoid global state; inject dependencies as &self or &mut self.

8. Least Privilege & Isolation
Guideline: Subsystems must operate with the minimum necessary permissions and isolated resources, enforced by both the type system and hardware.
Implementation:
Use hardware-assisted isolation (e.g., MMU domains, PAC/BTI, Shadow Stacks) and expose them via safe Rust wrappers.
Capability-based Security: Prefer passing explicit capability objects (Newtypes) rather than relying on global ambient authority.
Measurement & Attestation: Support cryptographic measurement of kernel state and modules.
Validate all inputs crossing trust boundaries; use TryFrom or similar traits for robust parsing.
Constraint: Implementation complexity for security must be balanced against the mandate for simplicity (Rule 20).

9. Asynchrony & Non-blocking Design
Guideline: Avoid blocking the execution flow. Write programs to be connected to other programs asynchronously.
Implementation:
Use Rust's async/await where it simplifies complex state machines.
Implement Futures for long-running I/O or hardware operations.
Favor non-blocking interfaces (e.g., ring buffers, completion queues) with Waker support.
Use executor-agnostic designs to allow flexibility in scheduling.

10. Concurrency & Scalability
Guideline: Optimize for multi-core scalability using Rust's thread-safety guarantees. Scalability is the child of modularity.
Implementation:
Prefer lock-free data structures or crossbeam-style RCU (Read-Copy-Update) primitives.
Use fine-grained locking or per-CPU data structures to minimize cache-line contention.
Leverage Atomic types for simple state synchronization.

11. Separation of Mechanism and Policy
Guideline: Separate policy from mechanism; separate interfaces from engines. The kernel provides the mechanism; userspace defines the policy.
Implementation:
A driver should expose hardware metrics and controls; a policy daemon decides when to trigger specific modes (e.g., power-saving).
Abstract execution units behind common Traits to simplify offloading (Rule of Diversity).
Exception: Closed-loop recovery (Rule 17) is a permitted exception where local policy is required for system resilience.

12. Programmable Extensibility
Guideline: Design for the future by enabling runtime extension of kernel behavior. Prefer writing programs that write programs (or interpret them).
Implementation:
Use safe bytecode engines (e.g., rbpf or similar) for tracing and policy.
Provide well-defined hook points via traits.
Ensure all extensions are sandboxed and cannot violate core memory safety invariants.

13. Fold Knowledge into Data
Guideline: Use Rust's powerful enum and match system to encode state transitions and hardware logic where it simplifies the mental model.
Implementation: Replace complex if/else logic trees with match over exhaustive Enums or state machines. Use the State Pattern to ensure only valid transitions are possible at compile-time. Data is easier to patch than logic.
Constraint: Do not introduce complex state machines if they significantly violate implementation simplicity (Rule 20).

14. Fail Loud, Fail Fast
Guideline: When you must fail, fail noisily and as soon as possible. Return specific Err variants or trigger a controlled panic! (in dev/debug) upon critical failure.
Implementation: Do not attempt partial recovery if internal state is corrupted. Fail immediately with a clear error signature. Use debug_assert! for internal invariants that should never be broken. Masking hardware errors leads to inconsistent state and data corruption.
Priority: This rule takes precedence over self-healing (Rule 17) in cases of critical state corruption.

15. Verification & Formalism
Guideline: Favor logic that can be statically proven using Rust's type system or formal verification tools. Verification is the child of transparency and simplicity.
Implementation:
Design protocols and state machines to be correct by construction using zero-cost abstractions and Typestates.
Implement property-based testing (e.g., proptest) for critical logic.
Use formal verification frameworks (like kani or flux) for MMU and scheduler invariants.

16. Energy Awareness & Power Efficiency
Guideline: Optimize for minimal energy consumption using tickless designs and power-aware scheduling.
Implementation:
Implement Race to Sleep: execute tasks efficiently to return to low-power states.
Use opportunistic sleep and expose energy models to the scheduler.

17. Resilience & Self-Healing
Guideline: Design subsystems to be restartable and capable of recovering from transient failures.
Implementation:
Implement Closed-loop recovery: detection -> isolation -> validated restart.
Use watchdog timers and health-check heartbeats.
Design drivers to be re-initializable without a full kernel reboot.
Constraint: Recovery must only be attempted for transient failures where internal state is not corrupted beyond repair (refer to Rule 14).

18. Determinism & Reproducibility
Guideline: Ensure system state and builds are deterministic and reproducible. Always do the least surprising thing.
Implementation:
Enforce bit-for-bit reproducible builds (identical binaries from same source).
Minimize reliance on non-deterministic hardware state during early boot.
Use immutable data structures for global configuration after init.

19. Support for Heterogeneous Computing
Guideline: Mistrust all claims for the one true way. Design for systems with diverse execution units (CPUs, GPUs, accelerators). Diversity is the child of modularity and composition.
Implementation:
Abstract execution units behind common Traits to simplify offloading.
Implement memory management that supports unified or shared virtual memory (SVM).

III. Strategy

20. Simplicity > Perfection
Guideline: Implementation simplicity is the highest priority. Favor clear Rust code over complex perfect solutions.
Reasoning: Simple code is easier to audit, port, and merge. A simple implementation is more robust than a complex one that handles 100% of cases but is impossible to verify.
Application: If handling a rare edge case requires doubling complexity, return an Err and let higher layers handle it.

21. Programmer Time > Machine Time
Guideline: Optimize for maintainability and compiler-aided correctness using Rust's high-level abstractions. Use machine time to save programmer time.
Application: Use Traits, Generics, and Closures even in the kernel. Avoid hand-tuned assembly for non-critical paths. Let the compiler do the heavy lifting.
Limit: Saving programmer time must not result in significant regressions to energy efficiency (Rule 16).

IV. Observability & Debugging

22. Structured Observability
Guideline: Design for transparency and tracing using structured logging.
Implementation:
Use machine-parsable logging (e.g., JSON or binary formats).
Implement tracing points for observation without performance hits.
Provide hardware telemetry to higher-level tools.
Constraint: Must not violate the Silence is Golden rule (Rule 4) during normal operation.

V. Anti-Patterns

Tight Coupling: Avoid hardware or software dependencies that force specific versions of unrelated components.
The Clever Code: Clarity is better than cleverness. If the logic relies on obscure Rust tricks that are hard to explain, rewrite it for clarity.
Feature Creep: Keep drivers focused on hardware abstraction, not policy or unrelated features.
