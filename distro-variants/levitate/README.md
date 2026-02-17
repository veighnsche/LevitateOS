# Levitate Variant (Stage 00 Model Scaffold)

This directory is the model Stage 00 declaration scaffold for all distro variants.

## Required Stage 00 Files

- `kconfig`
- `00Build.toml`
- `recipes/kernel.rhai`
- `00Build-build.sh`
- `00Build-build-capability.sh`

## Stage 00 Invariants Enforced

- Kernel configuration file must be declared as `kernel_kconfig_path = "kconfig"`.
- Kernel build must be orchestrated through Recipe Rhai:
  - `recipe_kernel_script = "distro-builder/recipes/linux.rhai"`
  - `recipe_kernel_invocation = "recipe install"`
- Kernel outputs and provenance fields are mandatory and validated.
- Modules installation path must be `/usr/lib/modules` for cross-distro consistency.

## Source Of Truth

`00Build.toml` is the authoritative Stage 00 conformance contract for this variant.
`distro-contract` loads and validates this manifest directly from `distro-variants`.
