How to build an image with [buildroot](https://buildroot.org/):

1. Install [buildroot](https://buildroot.org/)
2. Configure buildroot for your target infrastructure with `make menuconfig`
3. Configure buildroot to install the amaru package (you'll need to define the `BR2_EXTERNAL` env variable to point to this directory/pragma-org)
4. Run `make` to build the image

The following exemple considers you're running buildroot in docker.

## Using docker

### Build the image with docker

Start the buildroot container, from this directory:

```bash
docker run --rm --name buildroot -it -v buildroot:/home/br-user -v $PWD/pragma-org:/srv registry.gitlab.com/buildroot.org/buildroot/base:20250218.2110 bash
```

Inside the container, install buildroot:

```bash
git clone https://git.busybox.net/buildroot
cd buildroot/
```

Setup the target architecture, for instance for qemu with arm64 emulation:

```bash
make qemu_aarch64_virt_defconfig
```

Or if you prefer building for raspberry pi5:

```bash
make raspberrypi5_defconfig
```

Configure buildroot to install the amaru package:

```bash
make BR2_EXTERNAL=/srv menuconfig
```

In the menu `External options` activate `amaru`.

Then build the image:

```bash
make
```

### Run the image with qemu

We suppose you built the image for qemu with arm64 emulation. Let's fetch the image from the container:

```bash
docker cp buildroot:/home/br-user/buildroot/output/images .
```

Start the image:

```bash
qemu-system-aarch64 \
  -M virt \
  -cpu cortex-a76 \
  -m 2048 \
  -kernel images/Image \
  -append "root=/dev/vda console=ttyAMA0 rootwait" \
  -drive file=images/rootfs.ext4,format=raw,if=virtio \
  -serial mon:stdio \
  -net nic -net user
```

You can login as root without password.
