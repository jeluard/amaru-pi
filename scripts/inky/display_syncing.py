#!/usr/bin/env python

from create_syncing import create_syncing
from display import display_bitmap
from inky.auto import auto

import argparse

def main():
    parser = argparse.ArgumentParser(description="Create a syncing sign for a given epoch.")
    parser.add_argument("epoch", help="Epoch number", type=int)
    args = parser.parse_args()
    inky_display = auto()
    path = "syncing.png"
    color = "red"
    create_syncing(path, args.epoch, color, inky_display.WIDTH, inky_display.HEIGHT)
    display_bitmap(path)

if __name__ == "__main__":
    main()