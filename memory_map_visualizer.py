#!/usr/bin/env python3

import argparse
import sys
import math
import platform
import matplotlib
import matplotlib.pyplot as plt
import matplotlib.patches as patches
from matplotlib.ticker import NullFormatter

IMAGE_WIDTH = 800
IMAGE_HEIGHT = 1000
LEGEND_WIDTH = 70
COLUMN_NUM = 5

if platform.system() == 'Windows':
    plt.switch_backend('Qt5Agg')
elif platform.system() == 'Darwin':
    plt.switch_backend('MacOSX')
else:
    plt.switch_backend('TkAgg')

class MemoryAttributes:
    def __init__(self, readable, writable, executable, private, allocated):
        self.readable = readable
        self.writable = writable
        self.executable = executable
        self.private = private
        self.allocated = allocated

    def to_str(self):
        if self.allocated is False:
            return "Free"

        return f"{'r' if self.readable else '-'}{'w' if self.writable else '-'}{'x' if self.executable else '-'}"

    def to_color(self):
        if self.allocated is False:
            return (0, 0, 0)

        r = 1 if self.readable else 0
        g = 1 if self.writable else 0
        b = 1 if self.executable else 0

        return (r, g, b) if (r, g, b) != (0, 0, 0) else (128 / 255, 128 / 255, 128 / 255)


class MemoryRegion:
    def __init__(self, start, end, size, attributes, file_name=None):
        self.start = start
        self.end = end
        self.size = size
        self.attributes = attributes
        self.file_name = file_name

    def to_str(self):
        return f"{self.start:#x}--{self.end:#x} ({self.size:#x}) {self.attributes.to_str()} {self.file_name})"


def parse_memory_region(line):
    fields = line.split()
    if len(fields) < 2:
        return None

    range_str = fields[0].split('-')
    start = int(range_str[0], 16)
    end = int(range_str[1], 16)

    attributes = fields[1]

    if len(attributes) != 4:
        return None

    readable = attributes[0] == 'r'
    writable = attributes[1] == 'w'
    executable = attributes[2] == 'x'
    private = attributes[3] == 'p'

    size = end - start
    file_name = fields[5] if len(fields) > 5 else None

    return MemoryRegion(
        start,
        end,
        size,
        MemoryAttributes(readable, writable, executable, private, True),
        file_name,
    )


def read_memory_regions(file):
    memory_regions = []

    for line in file:
        region = parse_memory_region(line)
        if region is not None:
            memory_regions.append(region)

    memory_regions.sort(key=lambda r: r.start)
    return memory_regions


def insert_gap_memory_regions(memory_regions):
    regions_with_gaps = []
    prev_end = 0

    for region in memory_regions:
        if region.start > prev_end:
            gap_region = MemoryRegion(
                prev_end,
                region.start,
                region.start - prev_end,
                MemoryAttributes(False, False, False, False, False),
            )
            regions_with_gaps.append(gap_region)

        regions_with_gaps.append(region)
        prev_end = region.end

    return regions_with_gaps


def create_memory_map_figure(memory_regions, image_width, image_height, column_num):
    fig, ax = plt.subplots(figsize=(image_width / 80, image_height / 80), dpi=80)
    ax.set_xlim(0, image_width)
    ax.set_ylim(0, image_height)
    ax.invert_yaxis()
    ax.xaxis.set_major_formatter(NullFormatter())
    ax.yaxis.set_major_formatter(NullFormatter())

    def format_custom_coord(x, y):
        # Find the memory region containing the current y-coordinate
        label = ""
        for bar in ax.patches:
            if bar.get_y() <= y < bar.get_y() + bar.get_height() and \
                bar.get_x() <= x < bar.get_x() + bar.get_width():
                label = bar.get_label()  # Retrieve the region attribute from the bar
                break

        # Return the formatted region information
        return label
    
    ax.format_coord = format_custom_coord

    total_img_height = sum(math.pow(math.log(region.size), 3)
                           for region in memory_regions)
    current_y = 0
    column = 0

    column_gap = 20
    column_width = (image_width - LEGEND_WIDTH - (column_num - 1) * column_gap) / column_num

    for region in memory_regions:
        region_height = math.pow(math.log(region.size), 3)
        region_height_in_pixels = (
            region_height / total_img_height) * image_height * column_num
        region_color = region.attributes.to_color()

        while region_height_in_pixels > 0:
            x = LEGEND_WIDTH + column * (column_width + column_gap)
            height = region_height_in_pixels
            if current_y + height > image_height:
                height = image_height - current_y
            bar = patches.Rectangle((x, current_y),
                                    column_width, height, edgecolor=None, facecolor=region_color)
            bar.set_label(region.to_str())
            ax.add_patch(bar)

            current_y += height
            region_height_in_pixels -= height
            if current_y >= image_height:
                current_y = 0
                column += 1

    draw_legend(ax, image_width, image_height)
    plt.tight_layout()
    plt.show()


def draw_legend(ax, image_width, image_height):
    memory_types = [
        MemoryAttributes(False, False, False, True, True),
        MemoryAttributes(True, False, False, True, True),
        MemoryAttributes(False, True, False, True, True),
        MemoryAttributes(True, True, False, True, True),
        MemoryAttributes(False, False, True, True, True),
        MemoryAttributes(True, False, True, True, True),
        MemoryAttributes(False, True, True, True, True),
        MemoryAttributes(True, True, True, True, True),
        MemoryAttributes(True, True, True, True, False),
    ]

    legend_x = 5
    legend_y = 5

    for this_type in memory_types:
        legend_entry = patches.Rectangle(
            (legend_x, legend_y), 10, 10, edgecolor=None, facecolor=this_type.to_color())
        ax.add_patch(legend_entry)
        ax.text(legend_x + 15, legend_y, this_type.to_str(),
                fontsize=10, verticalalignment='top')
        legend_y += 20


def main():
    parser = argparse.ArgumentParser(
        description="Visualizes the memory layout of a process")
    parser.add_argument("file", nargs="?", type=argparse.FileType(
        "r"), default=sys.stdin, help="File containing memory map data")
    parser.add_argument("--width", type=int, default=IMAGE_WIDTH,
                        help="Image width in pixels")
    parser.add_argument("--height", type=int, default=IMAGE_HEIGHT,
                        help="Image height in pixels")
    parser.add_argument("--column", type=int, default=COLUMN_NUM,
                        help="Number of columns for memory regions")
    args = parser.parse_args()

    memory_regions = read_memory_regions(args.file)
    memory_regions = insert_gap_memory_regions(memory_regions)
    create_memory_map_figure(memory_regions, args.width, args.height, args.column)


if __name__ == "__main__":
    main()
