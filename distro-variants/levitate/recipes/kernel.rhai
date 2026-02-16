// CP0 contract declaration: Levitate kernel build must go through Recipe Rhai.
// The concrete build script remains centralized at distro-builder/recipes/linux.rhai.
let required_kernel_recipe = "distro-builder/recipes/linux.rhai";
let required_invocation = "recipe install";
