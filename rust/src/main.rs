use clap::{App, Arg};
use image::{ImageBuffer, Rgb};
use plotters::backend::RGBPixel;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use plotters::prelude::*;


const IMAGE_WIDTH: u32 = 300;
const IMAGE_HEIGHT: u32 = 2000;
const LEGEND_WIDTH: u32 = 150;

#[derive(Debug, PartialEq, Clone)]
struct MemoryAttributes {
    readable: bool,
    writable: bool,
    executable: bool,
    private: bool,
    allocated: bool,
}

#[derive(Debug, Clone)]
struct MemoryRegion {
    start: usize,
    end: usize,
    size: usize,
    attributes: MemoryAttributes,
    file_name: Option<String>,
}

impl FromStr for MemoryRegion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let fields: Vec<&str> = s.split_whitespace().collect();
        if fields.len() < 2 {
            return Err("Invalid input format".to_string());
        }

        let range: Vec<&str> = fields[0].split('-').collect();
        let start = usize::from_str_radix(range[0], 16).map_err(|_| "Invalid start address".to_string())?;
        let end = usize::from_str_radix(range[1], 16).map_err(|_| "Invalid end address".to_string())?;

        let attributes = fields[1];
        if attributes.len() != 4 {
            return Err("Invalid memory attributes".to_string());
        }

        let readable = attributes.chars().nth(0).unwrap() == 'r';
        let writable = attributes.chars().nth(1).unwrap() == 'w';
        let executable = attributes.chars().nth(2).unwrap() == 'x';
        let private = attributes.chars().nth(3).unwrap() == 'p';

        let size = end - start;
        let file_name = fields.get(5).map(|s| s.to_string());

        Ok(MemoryRegion {
            start,
            end,
            size,
            attributes: MemoryAttributes {
                readable,
                writable,
                executable,
                private,
                allocated: true,
            },
            file_name,
        })
    }
}

fn read_memory_regions(pid: u32) -> Vec<MemoryRegion> {
    let path = format!("/proc/{}/maps", pid);
    let file = File::open(path).expect("Unable to open the maps file");
    let reader = BufReader::new(file);

    let mut memory_regions = Vec::new();

    for line in reader.lines() {
        if let Ok(l) = line {
            if let Ok(region) = l.parse::<MemoryRegion>() {
                memory_regions.push(region);
            }
        }
    }

    memory_regions.sort_by(|a, b| a.start.cmp(&b.start));
    memory_regions
}

fn insert_gap_memory_regions(memory_regions: &[MemoryRegion]) -> Vec<MemoryRegion> {
    let mut regions_with_gaps = Vec::new();
    let mut prev_end: usize = 0;

    for region in memory_regions {
        if region.start > prev_end {
            let gap_region = MemoryRegion {
                start: prev_end,
                end: region.start,
                size: region.start - prev_end,
                attributes: MemoryAttributes {
                    readable: false,
                    writable: false,
                    executable: false,
                    private: false,
                    allocated: false,},
      
                file_name: None,
            };
            regions_with_gaps.push(gap_region);
        }
        regions_with_gaps.push(region.clone());
        prev_end = region.end;
    }

    regions_with_gaps
}


fn memory_type_color(attributes: &MemoryAttributes) -> Rgb<u8> {
    if attributes.allocated == false {
        return Rgb([0, 0, 0]);
    }

    let r: u8 = if attributes.readable { 255 } else { 0 };
    let g: u8 = if attributes.writable { 255 } else { 0 };
    let b: u8 = if attributes.executable { 255 } else { 0 };
    
    if r == 0 && g == 0 && b == 0 {
        return Rgb([128, 128, 128]);
    } else {
        return Rgb([r, g, b]);
    }
}

use plotters::prelude::*;
use plotters::style::{FontDesc, FontStyle, FontFamily};

fn create_memory_map_image(memory_regions: &[MemoryRegion], image_width: u32, image_height: u32) -> Result<image::RgbImage, Box<dyn std::error::Error>> {
    let mut imgbuf = image::ImageBuffer::new(image_width, image_height);
    {
        let backend = BitMapBackend::with_buffer(&mut imgbuf, (image_width, image_height));
        let mut root: DrawingArea<BitMapBackend, plotters::coord::Shift> = backend.into_drawing_area();
        root.fill(&WHITE)?;

        let mut total_img_height: f64 = 0.0;
        for region in memory_regions {
            total_img_height += (region.size as f64).log2().powi(3);
        }

        let mut current_y: i32 = 0;
        for region in memory_regions {
            let region_height = (region.size as f64).log2().powi(3);
            let region_height_in_pixels: i32 = ((region_height / total_img_height) * (image_height as f64)) as i32;
            let region_color = memory_type_color(&region.attributes);

            let bar = Rectangle::new(
                [(LEGEND_WIDTH as i32, current_y), (image_width as i32, current_y + region_height_in_pixels)],
                ShapeStyle::from(RGBColor(region_color[0], region_color[1], region_color[2]).filled().stroke_width(0)),
            );
            root.draw(&bar)?;

            let font = FontDesc::new(FontFamily::SansSerif, 10.0, FontStyle::Normal);
            let address_text = Text::new(format!("{:#x} ({:#x})", region.start, region.size), (25, current_y), font.clone());
            root.draw(&address_text)?;

            current_y += region_height_in_pixels;
        }

        draw_legend(&mut root, image_width as i32, image_height as i32)?;
        root.present()?;
    }

    Ok(imgbuf)
}

fn draw_legend(root: &mut DrawingArea<BitMapBackend, plotters::coord::Shift>, image_width: i32, image_height: i32) -> Result<(), Box<dyn std::error::Error>> {
    let font = FontDesc::new(FontFamily::SansSerif, 10.0, FontStyle::Normal);
    let memory_types = vec![
        ("Free", &GREEN),
        ("Used", &RED),
        ("Reserved", &YELLOW),
        ("NVS", &BLUE),
    ];

    let legend_x: i32 = 5;
    let mut legend_y: i32 = image_height - 20 * memory_types.len() as i32;

    for (name, color) in memory_types {
        let legend_entry = Rectangle::new(
            [(legend_x, legend_y), (legend_x + 10, legend_y + 10)],
            ShapeStyle::from(color).filled().stroke_width(0),
        );
        root.draw(&legend_entry)?;

        let legend_text = Text::new(name, (legend_x + 15, legend_y - 2), font.clone());
        root.draw(&legend_text)?;

        legend_y += 20;
    }

    Ok(())
}



fn main() {
    let matches = App::new("Memory Map Visualizer")
        .version("1.0")
        .author("Your Name <your@email.com>")
        .about("Visualizes the memory layout of a process")
        .arg(
            Arg::with_name("PID")
                .help("Process ID to visualize")
                .required(true)
                .index(1),
        )
        .get_matches();

    let pid = matches
        .value_of("PID")
        .unwrap()
        .parse::<u32>()
        .expect("Invalid PID");

    let memory_regions = read_memory_regions(pid);
    let memory_regions = insert_gap_memory_regions(&memory_regions);

    let img = create_memory_map_image(&memory_regions, IMAGE_WIDTH, IMAGE_HEIGHT)
        .expect("Unable to create memory map image");
    img.save("memory_map.png").expect("Unable to save image");

}
