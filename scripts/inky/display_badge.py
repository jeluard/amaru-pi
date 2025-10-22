#!/usr/bin/env python

from create_badge import create_badge
from display import display_bitmap
from inky.auto import auto

import argparse

def main():
    parser = argparse.ArgumentParser(description="Create a badge for a given epoch and block number.")
    parser.add_argument("epoch", help="Epoch number", type=int)
    parser.add_argument("block", help="Block number", type=int)
    args = parser.parse_args()
    inky_display = auto()
    path = "badge.png"
    color = "red"
    create_badge(path, args.epoch, args.block, color, inky_display.WIDTH, inky_display.HEIGHT)
    display_bitmap(path)

if __name__ == "__main__":
    main()