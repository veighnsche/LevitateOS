# Levitate Variant (CP0 Model Scaffold)

This directory is the model CP0 declaration scaffold for all distro variants.

## Required CP0 Files

- `kconfig`
- `cp0.toml`
- `recipes/kernel.rhai`
- `checkpoint-0-build-capability.sh`

## CP0 Invariants Enforced

- Kernel configuration file must be declared as `kernel_kconfig_path = "kconfig"`.
- Kernel build must be orchestrated through Recipe Rhai:
  - `recipe_kernel_script = "distro-builder/recipes/linux.rhai"`
  - `recipe_kernel_invocation = "recipe install"`
- Kernel outputs and provenance fields are mandatory and validated.
- Modules installation path must be `/usr/lib/modules` for cross-distro consistency.

## Source Of Truth

`cp0.toml` is the authoritative CP0 conformance contract for this variant.
`distro-contract` loads and validates this manifest directly from `distro-variants`.
