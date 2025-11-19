#!/usr/bin/env python3

from displayhatmini import DisplayHATMini
import sys

def detect_display_hat_mini():
    try:
        # Create a buffer: PIL Image, matching the HAT resolution.
        from PIL import Image
        width = DisplayHATMini.WIDTH
        height = DisplayHATMini.HEIGHT
        buffer = Image.new("RGB", (DisplayHATMini.WIDTH, DisplayHATMini.HEIGHT), (0, 0, 0))
        disp = DisplayHATMini(buffer=buffer, backlight_pwm=True)

        disp.set_backlight(0.0)
        disp.display()
        disp.display()
        # Try to turn on backlight to half
        disp.set_backlight(0.5)

        # If everything above didn't throw, we assume it's present
        return True
    except Exception as e:
        print("Error detecting Display HAT Mini:", e, file=sys.stderr)
        return False

if __name__ == "__main__":
    if detect_display_hat_mini():
        print("Display HAT Mini seems present!")
        sys.exit(0)
    else:
        print("No Display HAT Mini detected.")
        sys.exit(1)