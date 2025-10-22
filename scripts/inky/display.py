#!/usr/bin/env python

from PIL import Image
from inky.auto import auto

import argparse

def display_bitmap(path: str):
    # Auto-detect Inky display
    inky_display = auto()

    image = Image.open(path)

    # Resize to this Inky pHAT dimensions
    if image.size != inky_display.WIDTH or image.width != inky_display.HEIGHT:
        image = image.resize((inky_display.WIDTH, inky_display.HEIGHT))

    # Convert to display palette
    image = image.convert("P")

    # Display the image
    inky_display.set_image(image)
    inky_display.show()

def main():
    parser = argparse.ArgumentParser(description="Display an image on an Inky display.")
    parser.add_argument("path", help="Path to the image")
    args = parser.parse_args()
    display_bitmap(args.path)

if __name__ == "__main__":
    main()