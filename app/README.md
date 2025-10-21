To run the simulator:

```shell
# On OSX
# Install SDL2
brew install sdl2
# Then make sure the following is run in your terminal
export LIBRARY_PATH="$LIBRARY_PATH:$(brew --prefix)/lib"

make run-simulator
```

You can also take a screenshot using the simalator by setting `EG_SIMULATOR_DUMP=screenshot.png`.

To build and deploy on a pi, first install `cargo install --locked cargo-zigbuild`:

then:

```shell
export SCP_TARGET=pi@pi.local:.
make upload-pi
```

Then ssh to the machine and execute `./app`.

# PI optimizations

In `/boot/firmware/config.txt`

```ini
# Disable HDMI0
dtoverlay=vc4-kms-dpi-disable,display=0

# Disable HDMI1
dtoverlay=vc4-kms-dpi-disable,display=1

# Lower GPU memory used
gpu_mem=16 # Check using `vcgencmd get_mem gpu`

# Reduce CPU frequency
arm_freq=1500 # Check using `vcgencmd measure_clock arm`

# Slight undervolt for efficiency (-50mV)
over_voltage=-2 # Check with `vcgencmd get_config int | grep over_voltage` or `vcgencmd measure_volts`
```

In `/boot/firmware/cmdline.txt`

At the end of the line, add ` maxcpus=2` (including leading space).
Can be checked using `cat /sys/devices/system/cpu/online` (should show `0-1`)