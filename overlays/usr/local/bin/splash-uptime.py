#!/usr/bin/env python3

from PIL import Image, ImageDraw, ImageFont
from displayhatmini import DisplayHATMini
import signal
import sys


def initialize_display():
    """Initialize the DisplayHATMini and the drawing buffer."""
    width = DisplayHATMini.WIDTH
    height = DisplayHATMini.HEIGHT
    buffer = Image.new("RGB", (width, height))
    draw = ImageDraw.Draw(buffer)
    display = DisplayHATMini(buffer)
    return width, height, buffer, draw, display

def load_fonts():
    """Load fonts for display."""
    try:
        font_large = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 24)
        font_small = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 16)
    except:
        font_large = ImageFont.load_default()
        font_small = ImageFont.load_default()
    return font_large, font_small

def wipe_screen():
    draw.rectangle((0, 0, width, height), fill=(0, 0, 0))

def draw_title(title):
    draw.text((width // 2, 30), title, fill=(255, 255, 255), font=font_large, anchor="mm")

def draw_sub_title(sub_title):
    draw.text((width // 2, 80), sub_title, fill=(0, 255, 0), font=font_large, anchor="mm")

def draw_text(text):
    draw.text((width // 2, 120), text, fill=(100, 100, 255), font=font_small, anchor="mm")
    
def read_uptime():
    """Read system uptime in seconds from /proc/uptime."""
    with open('/proc/uptime', 'r') as f:
        uptime_seconds = int(float(f.readline().split()[0]))
    hours = uptime_seconds // 3600
    minutes = (uptime_seconds % 3600) // 60
    seconds = uptime_seconds % 60
    return hours, minutes, seconds, uptime_seconds

width, height, buffer, draw, displayhatmini = initialize_display()
font_large, font_small = load_fonts()
hours, minutes, seconds, uptime_seconds = read_uptime()

wipe_screen()
draw_title("UPTIME")
draw_sub_title(f"{hours:02d}:{minutes:02d}:{seconds:02d}")
draw_text(f"{uptime_seconds} seconds")

displayhatmini.display()

print(f"Displayed uptime: {hours:02d}:{minutes:02d}:{seconds:02d} ({uptime_seconds} seconds)")