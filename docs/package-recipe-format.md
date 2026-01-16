# Package Recipe S-Expression Format

This document describes the S-expression format for LevitateOS package recipes. The format is designed to be:
- **LLM-generatable** - SmolLM3-3B can produce valid recipes
- **Minimal parser** - Parser is ~30 lines, all logic in recipes
- **Composable** - Higher-order recipes can wrap other recipes

## Parser

The entire parser:

```rust
enum Expr {
    Atom(String),
    List(Vec<Expr>),
}

fn parse(input: &str) -> Result<Expr, Error> {
    // ~30 lines total
}
```

All semantics live in the recipe interpreter, not the parser.

## Basic Structure

```lisp
(package "name" "version"
  (action1 ...)
  (action2 ...)
  ...)
```

## Actions

### Metadata

```lisp
(description "Human-readable description")
(license "MIT" "Apache-2.0")
(homepage "https://example.com")
(maintainer "name <email>")
```

### Dependencies

```lisp
;; Required at runtime
(deps "openssl" "zlib")

;; Required only for building
(build-deps "gcc" "make" "rust")

;; Optional features
(optional-deps
  ("geoip" "Enable GeoIP support")
  ("perl" "Enable Perl modules"))
```

### Acquire

How to download the package source or binary.

```lisp
;; Download source tarball
(acquire
  (source "https://example.com/pkg-1.0.tar.gz")
  (verify (sha256 "abc123...")))

;; Download pre-built binary
(acquire
  (binary
    (x86_64 "https://example.com/pkg-linux-x86_64.tar.gz")
    (aarch64 "https://example.com/pkg-linux-aarch64.tar.gz")))

;; Clone git repository
(acquire
  (git "https://github.com/user/repo.git"
    (tag "v1.0.0")))

;; Use OS package manager as fallback
(acquire
  (os-package
    (pacman "package-name")
    (apt "package-name")
    (dnf "package-name")))

;; Conditional acquisition
(acquire
  (if (prefer 'binary)
    (binary ...)
    (source ...)))
```

### Build

How to compile or extract the package.

```lisp
;; No build needed (pre-built binary)
(build skip)

;; Simple extraction
(build (extract tar-gz))

;; Standard configure/make
(build
  (configure "./configure --prefix=$PREFIX")
  (compile "make -j$NPROC")
  (test "make test"))

;; Rust/Cargo project
(build
  (cargo "build --release"))

;; Meson project
(build
  (meson "setup build")
  (ninja "-C build"))

;; Custom commands
(build
  (run "cmake -B build")
  (run "cmake --build build"))
```

### Install

Where to place files.

```lisp
;; Simple binary installation
(install
  (to-bin "target/release/myapp"))

;; Multiple files
(install
  (to-bin "build/myapp" "myapp")
  (to-lib "build/libfoo.so")
  (to-config "myapp.conf" "/etc/myapp/")
  (to-man "docs/myapp.1")
  (to-share "assets/" "/usr/share/myapp/"))

;; With permissions
(install
  (to-bin "myapp" (mode 755))
  (to-config "myapp.conf" (mode 644)))

;; Create symlinks
(install
  (link "$PREFIX/bin/myapp" "/usr/local/bin/myapp"))
```

### Configure

Post-install configuration.

```lisp
;; Create system user
(configure
  (create-user "myapp" (system) (no-login)))

;; Create directories
(configure
  (create-dir "/var/lib/myapp" (owner "myapp"))
  (create-dir "/var/log/myapp"))

;; Template config file
(configure
  (template "/etc/myapp/config.toml"
    (port 8080)
    (data-dir "/var/lib/myapp")))

;; Run post-install script
(configure
  (run "myapp --init"))
```

### Start

How to run the application.

```lisp
;; Simple executable
(start (exec "myapp"))

;; With arguments passthrough
(start (exec "myapp" $@))

;; As systemd service
(start (service "systemd" "myapp.service"))

;; Sandboxed execution
(start
  (sandbox
    (allow-read "/")
    (allow-write "$HOME/.config/myapp")
    (allow-net)
    (exec "myapp")))

;; Wrapped with another tool
(start
  (wrap "firejail" "--private" "--" "myapp"))
```

### Stop

How to stop the application.

```lisp
;; Stop systemd service
(stop (service-stop "myapp.service"))

;; Kill by name
(stop (pkill "myapp"))

;; Graceful then force
(stop
  (signal "myapp" SIGTERM)
  (wait 5)
  (signal "myapp" SIGKILL))
```

### Update

How to check for and apply updates.

```lisp
;; Check GitHub releases
(update
  (check (github "user/repo")))

;; Check URL pattern
(update
  (check (url-pattern "https://example.com/releases/" "pkg-([0-9.]+).tar.gz")))

;; Migration from older versions
(update
  (migrate-from "1.x"
    (backup "/etc/myapp")
    (run "myapp migrate")))

;; Self-update command
(update
  (self-update "myapp update"))
```

### Remove

How to uninstall.

```lisp
;; Simple removal
(remove
  (rm-prefix))  ;; Remove everything under $PREFIX

;; Detailed removal
(remove
  (stop-first)  ;; Stop service before removing
  (rm-bin "myapp")
  (rm-config "/etc/myapp" (prompt))  ;; Ask user
  (rm-data "/var/lib/myapp" (keep))  ;; Keep user data
  (rm-user "myapp"))

;; With hooks
(remove
  (pre-remove (backup "/etc/myapp"))
  (rm-prefix)
  (post-remove (log "Package removed")))
```

### Clean

Remove build artifacts and cache.

```lisp
(clean
  (rm-build-dir)
  (rm-download-cache)
  (rm-tmp))
```

### Hooks

Pre/post actions for lifecycle events.

```lisp
(hooks
  (pre-install
    (check-port-available 80)
    (check-disk-space "500M"))
  (post-install
    (enable-service "myapp")
    (log "Installation complete"))
  (pre-remove
    (backup-config))
  (post-remove
    (cleanup-orphans)))
```

## Complete Example: ripgrep

```lisp
(package "ripgrep" "14.1.0"
  (description "Fast grep alternative written in Rust")
  (license "MIT")
  (homepage "https://github.com/BurntSushi/ripgrep")

  (deps)  ;; No runtime deps (static binary)

  (acquire
    (binary
      (x86_64 "https://github.com/BurntSushi/ripgrep/releases/download/14.1.0/ripgrep-14.1.0-x86_64-unknown-linux-musl.tar.gz")
      (aarch64 "https://github.com/BurntSushi/ripgrep/releases/download/14.1.0/ripgrep-14.1.0-aarch64-unknown-linux-gnu.tar.gz"))
    (verify (sha256-url ".sha256")))

  (build (extract tar-gz))

  (install
    (to-bin "rg")
    (to-man "doc/rg.1")
    (to-complete "complete/rg.bash" "bash")
    (to-complete "complete/rg.zsh" "zsh"))

  (start (exec "rg" $@))

  (update (check (github "BurntSushi/ripgrep")))

  (remove (rm-prefix)))
```

## Complete Example: nginx (complex)

```lisp
(package "nginx" "1.25.0"
  (description "High-performance HTTP server")
  (license "BSD-2-Clause")
  (homepage "https://nginx.org")

  (deps "openssl" "pcre2" "zlib")
  (build-deps "gcc" "make")

  (acquire
    (source "https://nginx.org/download/nginx-1.25.0.tar.gz")
    (verify (sha256 "abc123...")))

  (build
    (configure "./configure"
      "--prefix=/opt/nginx"
      "--with-http_ssl_module"
      "--with-http_v2_module"
      "--with-pcre")
    (compile "make -j$NPROC")
    (test "make test"))

  (install
    (to-bin "objs/nginx")
    (to-config "conf/nginx.conf" "/etc/nginx/")
    (to-config "conf/mime.types" "/etc/nginx/")
    (to-man "docs/man/nginx.8"))

  (configure
    (create-user "nginx" (system) (no-login))
    (create-dir "/var/log/nginx" (owner "nginx"))
    (create-dir "/var/cache/nginx" (owner "nginx")))

  (start (service "systemd" "nginx.service"))
  (stop (service-stop "nginx.service"))

  (update
    (check (url-pattern "https://nginx.org/en/download.html" "nginx-([0-9.]+)"))
    (migrate-from "*" (backup "/etc/nginx")))

  (remove
    (stop-first)
    (rm-prefix)
    (rm-config "/etc/nginx" (prompt))
    (rm-data "/var/log/nginx" (prompt))
    (rm-user "nginx"))

  (hooks
    (pre-install (check-port-available 80 443))
    (post-install (systemctl "enable" "nginx"))))
```

## Higher-Order Recipes

Define reusable templates:

```lisp
;; Define a template for Rust CLI tools
(define (rust-cli-binary name version repo)
  (package name version
    (deps)
    (acquire (github-release repo version))
    (build (extract tar-gz))
    (install (to-bin name))
    (start (exec name $@))
    (update (check (github repo)))
    (remove (rm-prefix))))

;; Use the template
(rust-cli-binary "fd" "9.0.0" "sharkdp/fd")
(rust-cli-binary "bat" "0.24.0" "sharkdp/bat")
(rust-cli-binary "eza" "0.18.0" "eza-community/eza")

;; Wrap with sandboxing
(define (sandboxed-app base)
  (compose base
    (start (sandbox (allow-read "/") (deny-net) (exec)))))

(sandboxed-app (rust-cli-binary "ripgrep" "14.1.0" "BurntSushi/ripgrep"))
```

## SmolLM3-3B Integration

The LLM generates recipes via function call:

```python
PACKAGE_TOOL = {
    "type": "function",
    "function": {
        "name": "generate_package_recipe",
        "description": "Generate an S-expression package recipe",
        "parameters": {
            "type": "object",
            "properties": {
                "recipe": {
                    "type": "string",
                    "description": "Complete S-expression package recipe"
                }
            },
            "required": ["recipe"]
        }
    }
}
```

User: "install ripgrep"

Model output:
```
<start_function_call>call:generate_package_recipe{recipe:<escape>(package "ripgrep" "14.1.0" (acquire (github-release "BurntSushi/ripgrep" "14.1.0")) (build (extract tar-gz)) (install (to-bin "rg")) (start (exec "rg" $@)) (update (check (github "BurntSushi/ripgrep"))) (remove (rm-prefix)))<escape>}<end_function_call>
```

## Training Data Format

```python
{
    "messages": [
        {"role": "developer", "content": "You generate package recipes..."},
        {"role": "user", "content": "install ripgrep"},
        {"role": "assistant", "tool_calls": [{
            "type": "function",
            "function": {
                "name": "generate_package_recipe",
                "arguments": {
                    "recipe": "(package \"ripgrep\" \"14.1.0\" ...)"
                }
            }
        }]}
    ],
    "tools": [PACKAGE_TOOL]
}
```
