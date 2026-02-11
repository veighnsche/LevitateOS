# LevitateOS development commands

# QEMU tools environment
tools_prefix := join(justfile_directory(), "leviso/downloads/.tools")
export PATH := tools_prefix / "usr/bin" + ":" + tools_prefix / "usr/libexec" + ":" + env("PATH")
export LD_LIBRARY_PATH := tools_prefix / "usr/lib64"
export OVMF_PATH := tools_prefix / "usr/share/edk2/ovmf/OVMF_CODE.fd"

ovmf := tools_prefix / "usr/share/edk2/ovmf/OVMF_CODE.fd"

# Boot into a checkpoint stage (interactive serial, Ctrl-A X to exit)
[no-exit-message]
checkpoint n distro="leviso":
    #!/usr/bin/env bash
    set -euo pipefail

    # Determine ISO and disk paths based on distro
    if [ "{{distro}}" = "leviso" ]; then
        iso="{{justfile_directory()}}/leviso/output/levitateos-x86_64.iso"
        disk_dir="{{justfile_directory()}}/leviso/output"
        disk_name="levitate-test.qcow2"
        vars_name="levitate-ovmf-vars.fd"
        pretty_name="LevitateOS"
    elif [ "{{distro}}" = "acorn" ]; then
        iso="{{justfile_directory()}}/AcornOS/output/acornos.iso"
        disk_dir="{{justfile_directory()}}/AcornOS/output"
        disk_name="acorn-test.qcow2"
        vars_name="acorn-ovmf-vars.fd"
        pretty_name="AcornOS"
    elif [ "{{distro}}" = "iuppiter" ]; then
        iso="{{justfile_directory()}}/IuppiterOS/output/iuppiteros.iso"
        disk_dir="{{justfile_directory()}}/IuppiterOS/output"
        disk_name="iuppiter-test.qcow2"
        vars_name="iuppiter-ovmf-vars.fd"
        pretty_name="IuppiterOS"
    else
        echo "Unknown distro: {{distro}}"
        echo "Valid options: leviso, acorn, iuppiter"
        exit 1
    fi

    if [ "{{n}}" = "1" ]; then
        echo "Booting $pretty_name live ISO... (Ctrl-A X to exit)"
        qemu-system-x86_64 \
            -enable-kvm \
            -cpu host \
            -smp 4 \
            -m 4G \
            -device virtio-scsi-pci,id=scsi0 \
            -device scsi-cd,drive=cdrom0,bus=scsi0.0 \
            -drive id=cdrom0,if=none,format=raw,readonly=on,file="$iso" \
            -drive if=pflash,format=raw,readonly=on,file={{ovmf}} \
            -vga none \
            -nographic \
            -serial mon:stdio \
            -no-reboot
    elif [ "{{n}}" = "4" ]; then
        echo "Booting installed $pretty_name... (Ctrl-A X to exit)"
        qemu-system-x86_64 \
            -enable-kvm \
            -cpu host \
            -smp 4 \
            -m 4G \
            -drive file="$disk_dir/$disk_name",format=qcow2,if=virtio \
            -drive if=pflash,format=raw,readonly=on,file={{ovmf}} \
            -drive if=pflash,format=raw,file="$disk_dir/$vars_name" \
            -boot c \
            -netdev user,id=net0 \
            -device virtio-net-pci,netdev=net0 \
            -vga none \
            -nographic \
            -serial mon:stdio \
            -no-reboot
    else
        echo "Checkpoint {{n}} is automated — use 'just test {{n}} {{distro}}' instead"
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
