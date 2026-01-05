# Phase 2: Structural Extraction

## Target Design
We will introduce new `clap` structs to represent the nested commands.

```rust
#[derive(Subcommand)]
enum Commands {
    // === High Level ===
    Test(TestArgs),
    Clean,

    // === Groups ===
    #[command(subcommand)]
    Build(BuildCommands),
    
    #[command(subcommand)]
    Run(RunCommands),
    
    #[command(subcommand)]
    Image(ImageCommands),
    
    #[command(subcommand)]
    Check(CheckCommands), // For linting, formating etc later?
}

#[derive(Args)]
struct TestArgs {
    #[arg(default_value = "all")]
    suite: String,
}

#[derive(Subcommand)]
enum BuildCommands {
    /// Build everything (Kernel + Userspace + Disk) - Default if possible, or explicit 'All'
    All,
    /// Build kernel only
    Kernel,
    /// Build userspace only
    Userspace {
        /// Build specific package?
        #[arg(long)]
        package: Option<String>,
    },
}

#[derive(Subcommand)]
enum RunCommands {
    /// Run default QEMU (512MB, generic)
    Default,
    /// Run Pixel 6 Profile
    Pixel6,
    /// Run with VNC for browser verification
    Vnc,
}

#[derive(Subcommand)]
enum ImageCommands {
    /// Create/Format the disk image
    Create,
    /// Install userspace binaries to disk
    Install,
    /// Dump framebuffer to file (QMP) - Moved from GpuDump
    Screenshot {
        #[arg(default_value = "screenshot.png")]
        output: String,
    },
}
```

## Extraction Strategy
We will modify `xtask/src/main.rs` directly as it is small enough (<600 lines) to do in one pass without complex multi-file extraction yet. `qmp.rs` and `tests/` modules are already separate.

## Step 1: Define Structs
Modify `Cli` and `Commands` to usage the new nested structure.

## Step 2: Refactor `main` match arm
Update the `match cli.command` block to handle the new nesting.
- `Commands::Build(cmd)` -> `match cmd { ... }`
- `Commands::Run(cmd)` -> `match cmd { ... }`

## Step 3: Implement `Clean`
Add logic to `xtask clean`:
```rust
fn clean() -> Result<()> {
    // Kill QEMU
    // Kill websockify
    // Remove sock files
    // Remove target dir? No, that's cargo clean.
    Ok(())
}
```
