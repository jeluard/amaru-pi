#!/usr/bin/env python

from PIL import Image, ImageDraw, ImageFont, ImageOps
from font_fredoka_one import FredokaOne

import argparse

def fit_font(draw, text, max_width, base_size=22):
    size = base_size
    while size > 10:
        font = ImageFont.truetype(FredokaOne, size)
        if draw.textlength(text, font=font) <= max_width:
            return font
        size -= 1
    return font

def create_logo(path: str, text_color: str, width: int, height: int):
    img = Image.new("P", (width, height), color="white")
    draw = ImageDraw.Draw(img)
    font = ImageFont.truetype(FredokaOne, 28)

    # Draw label 'amaru'
    label_x, label_y = 70, 0
    draw.text((label_x, label_y), "AMARU", fill=text_color, font=font)

    amaru_width = draw.textlength("amaru", font=font)

    img.save(path)

def main():
    parser = argparse.ArgumentParser(description="Create a logo for amaru")
    parser.add_argument("path", help="Path to the badge image", type=str)
    parser.add_argument("width", help="Width of the badge", type=int)
    parser.add_argument("height", help="Height of the badge", type=int)
    parser.add_argument("color", help="Color of the badge", type=str)
    args = parser.parse_args()
    create_logo(args.path, args.color, args.width, args.height)

if __name__ == "__main__":
    main()