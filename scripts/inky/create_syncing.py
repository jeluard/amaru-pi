#!/usr/bin/env python

from PIL import Image, ImageDraw, ImageFont, ImageOps
from font_fredoka_one import FredokaOne

import argparse

def create_syncing(path: str, epoch: int, text_color: str, width: int, height: int):
    img = Image.new("P", (width, height), color="white")
    draw = ImageDraw.Draw(img)
    font = ImageFont.truetype(FredokaOne, 22)

    # Draw label 'syncing'
    label_x, label_y = 70, 10
    draw.text((label_x, label_y), "SYNCING", fill=text_color, font=font)

    # Draw epoch number
    draw.text((label_x, 35), f"Epoch {epoch}", fill="black", font=font)

    img.save(path)

def main():
    parser = argparse.ArgumentParser(description="Create a syncing sign for a given epoch.")
    parser.add_argument("path", help="Path to the image", type=str)
    parser.add_argument("epoch", help="Epoch number", type=int)
    parser.add_argument("width", help="Width of the image", type=int)
    parser.add_argument("height", help="Height of the image", type=int)
    parser.add_argument("color", help="Color of the image", type=str)
    args = parser.parse_args()
    create_syncing(args.path, args.epoch, args.color, args.width, args.height)

if __name__ == "__main__":
    main()