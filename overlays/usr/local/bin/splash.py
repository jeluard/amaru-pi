#!/usr/bin/env python3

import time
from PIL import Image, ImageDraw
from displayhatmini import DisplayHATMini

width = DisplayHATMini.WIDTH
height = DisplayHATMini.HEIGHT
buffer = Image.new("RGB", (width, height))
hat = DisplayHATMini(buffer)
hat.display()
image = Image.open("/home/pi/logo.png").convert("RGB")
image = image.resize((width, height))
buffer.paste(image, (0, 0))
hat.set_led(0, 0, 0)

hat.display()

time.sleep(5)