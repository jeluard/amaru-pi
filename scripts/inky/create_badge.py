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

def create_badge(path: str, epoch: int, block: int, text_color: str, width: int, height: int):
    img = Image.new("P", (width, height), color="white")
    draw = ImageDraw.Draw(img)
    font = ImageFont.truetype(FredokaOne, 22)

    big_number = "#" + str(block)

    # Draw label 'amaru'
    label_x, label_y = 70, 10
    draw.text((label_x, label_y), "AMARU", fill=text_color, font=font)

    amaru_width = draw.textlength("amaru", font=font)

    # Draw epoch number
    draw.text((label_x, 35), f"Epoch {epoch}", fill="black", font=font)

    number_font = fit_font(draw, big_number, width - 10)

    # Draw block number
    num_w = draw.textlength(big_number, font=number_font)
    num_x = (width - num_w) // 2
    num_y = height - 18  - 10
    draw.text((num_x, num_y), big_number, fill=text_color, font=number_font)

    img.save(path)

def main():
    parser = argparse.ArgumentParser(description="Create a badge for a given epoch and block number.")
    parser.add_argument("path", help="Path to the badge image", type=str)
    parser.add_argument("epoch", help="Epoch number", type=int)
    parser.add_argument("block", help="Block number", type=int)
    parser.add_argument("width", help="Width of the badge", type=int)
    parser.add_argument("height", help="Height of the badge", type=int)
    parser.add_argument("color", help="Color of the badge", type=str)
    args = parser.parse_args()
    create_badge(args.path, args.epoch, args.block, args.color, args.width, args.height)

if __name__ == "__main__":
    main()