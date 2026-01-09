# images/

External OS images for integration testing.

## Required Images

| File | Architecture | Source |
|------|--------------|--------|
| `alpine-virt-3.20.0-x86_64.iso` | x86_64 | Alpine Linux |
| `alpine-virt-3.20.0-aarch64.iso` | aarch64 | Alpine Linux |

## Download

Run the download script:

```bash
./tests/images/download.sh
```

Or manually download from:
- https://dl-cdn.alpinelinux.org/alpine/v3.20/releases/x86_64/alpine-virt-3.20.0-x86_64.iso
- https://dl-cdn.alpinelinux.org/alpine/v3.20/releases/aarch64/alpine-virt-3.20.0-aarch64.iso

## Why Alpine?

- Small (~50MB ISO)
- Fast boot (~2-3 seconds)
- Stable versioned releases
- Supports both x86_64 and aarch64
- No installation needed (live mode)

## .gitignore

ISO files are large and should not be committed:

```
tests/images/*.iso
```
