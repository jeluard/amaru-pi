#!/usr/bin/env python

from create_logo import create_logo
from display import display_bitmap
from inky.auto import auto

import argparse

def main():
    parser = argparse.ArgumentParser(description="Create an amaru logo.")
    args = parser.parse_args()
    inky_display = auto()
    path = "logo.png"
    color = "red"
    create_logo(path, color, inky_display.WIDTH, inky_display.HEIGHT)
    display_bitmap(path)

if __name__ == "__main__":
    main()