PI_IMG ?=
RPI_WORKDIR ?= .rpi-qemu
SSH_FORWARD_PORT ?= 2222
HTTP_FORWARD_PORT ?= 8080

.PHONY: run-pi ssh ssh-pi stop-pi

run-pi: ## &pi Boot a Raspberry Pi image under QEMU with systemd and SSH forwarding
	@if [ -z "$(PI_IMG)" ]; then \
		echo "PI_IMG is required, e.g. PI_IMG=../amaru-rpi/sdcard-summit.img.gz make $@"; \
		exit 1; \
	fi
	WORKDIR="$(RPI_WORKDIR)" SSH_FORWARD_PORT="$(SSH_FORWARD_PORT)" HTTP_FORWARD_PORT="$(HTTP_FORWARD_PORT)" ./scripts/run-pi.sh "$(PI_IMG)"

ssh-pi: ## &pi Open an SSH shell to the local Raspberry Pi VM on localhost:$SSH_FORWARD_PORT
	@if [ ! -f "$(RPI_WORKDIR)/id_ed25519" ]; then \
		echo "Missing SSH key at $(RPI_WORKDIR)/id_ed25519; run 'make PI_IMG=/path/to/image run-pi' first."; \
		exit 1; \
	fi
	ssh -F /dev/null -i "$(RPI_WORKDIR)/id_ed25519" \
		-o IdentitiesOnly=yes \
		-o StrictHostKeyChecking=no \
		-o UserKnownHostsFile=/dev/null \
		-p "$(SSH_FORWARD_PORT)" \
		root@localhost

stop-pi: ## &pi Stop the local Raspberry Pi QEMU VM
	@if pgrep -f qemu-system-aarch64 >/dev/null 2>&1; then \
		pkill -f qemu-system-aarch64; \
		echo "Stopped qemu-system-aarch64"; \
	else \
		echo "No qemu-system-aarch64 process found"; \
	fi
