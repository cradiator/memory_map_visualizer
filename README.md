# Memory Map Visualizer

Memory Map Visualizer is an experimental project that demonstrates the use of AI for programming. This repository is accompanied by a blog post https://sysdbg.blogspot.com/2023/04/gpt-4.html, which explains the process and results in more detail. The repository includes both Python and Rust versions of the memory map visualizer, as well as the chat history from the AI-guided programming session.

## Overview

The Memory Map Visualizer is a tool that helps visualize the memory layout of a process. It reads memory regions from an input file and displays them graphically. Each memory region is color-coded based on its attributes, such as read/write/execute permissions.

## Python Version

The Python version of the Memory Map Visualizer is built using the `matplotlib` library for creating the graphical visualization of the memory layout.

### Prerequisites

- Python 3.7 or later
- matplotlib library

You can install the required library using pip:

```bash
pip install matplotlib
```

### Usage

To use the Python version, first ensure you have the required dependencies installed. Then, you can run the script with command-line options to control the image width, height, and number of columns. For example:

```bash
python memory_map_visualizer.py input_file.txt --width 1000 --height 1200 --column 4
```

This command will create an image with a width of 1000 pixels, a height of 1200 pixels, and 4 columns for memory regions. If you don't provide these options, the script will use the default values (width: 800, height: 1000, columns: 5).


## Rust Version

The Rust version of the Memory Map Visualizer can be found in the `rust` directory. 

## Chat History

The chat history from the AI-guided programming session is included in the `chat_history.pdf` file. 
