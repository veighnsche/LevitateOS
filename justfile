# LevitateOS development commands

# QEMU tools environment
tools_prefix := join(justfile_directory(), "leviso/downloads/.tools")
export PATH := tools_prefix / "usr/bin" + ":" + tools_prefix / "usr/libexec" + ":" + env("PATH")
export LD_LIBRARY_PATH := tools_prefix / "usr/lib64"
export OVMF_PATH := tools_prefix / "usr/share/edk2/ovmf/OVMF_CODE.fd"

iso := join(justfile_directory(), "leviso/output/levitateos-x86_64.iso")
ovmf := tools_prefix / "usr/share/edk2/ovmf/OVMF_CODE.fd"
disk_dir := join(justfile_directory(), "leviso/output")

# Boot into a checkpoint stage (interactive serial, Ctrl-A X to exit)
[no-exit-message]
checkpoint n:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "{{n}}" = "1" ]; then
        echo "Booting LevitateOS live ISO... (Ctrl-A X to exit)"
        qemu-system-x86_64 \
            -enable-kvm \
            -cpu host \
            -smp 4 \
            -m 4G \
            -device virtio-scsi-pci,id=scsi0 \
            -device scsi-cd,drive=cdrom0,bus=scsi0.0 \
            -drive id=cdrom0,if=none,format=raw,readonly=on,file={{iso}} \
            -drive if=pflash,format=raw,readonly=on,file={{ovmf}} \
            -vga none \
            -nographic \
            -serial mon:stdio \
            -no-reboot
    elif [ "{{n}}" = "4" ]; then
        echo "Booting installed LevitateOS... (Ctrl-A X to exit)"
        qemu-system-x86_64 \
            -enable-kvm \
            -cpu host \
            -smp 4 \
            -m 4G \
            -drive file={{disk_dir}}/levitate-test.qcow2,format=qcow2,if=virtio \
            -drive if=pflash,format=raw,readonly=on,file={{ovmf}} \
            -drive if=pflash,format=raw,file={{disk_dir}}/levitate-ovmf-vars.fd \
            -boot c \
            -netdev user,id=net0 \
            -device virtio-net-pci,netdev=net0 \
            -vga none \
            -nographic \
            -serial mon:stdio \
            -no-reboot
    else
        echo "Checkpoint {{n}} is automated — use 'just test {{n}}' instead"
        echo "Interactive checkpoints: 1 (live), 4 (installed)"
        exit 1
    fi

# Run automated checkpoint test (pass/fail)
test n distro="levitate":
    cd testing/install-tests && cargo run --bin checkpoints -- --distro {{distro}} --checkpoint {{n}}

# Run all checkpoint tests up to N
test-up-to n distro="levitate":
    cd testing/install-tests && cargo run --bin checkpoints -- --distro {{distro}} --up-to {{n}}

# Show checkpoint test status
test-status distro="levitate":
    cd testing/install-tests && cargo run --bin checkpoints -- --distro {{distro}} --status

# Reset checkpoint test state
test-reset distro="levitate":
    cd testing/install-tests && cargo run --bin checkpoints -- --distro {{distro}} --reset

# Build ISO
build distro="leviso":
    cd {{distro}} && cargo run -- build
