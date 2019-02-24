use std::fs::File;
use std::io::{BufWriter, Write};

pub struct PPMWriter {
	width: i32,
	x_position: i32,
	file_handle: BufWriter<File>,
}

impl PPMWriter {
	pub fn new(output_filename: &str, width: i32, height: i32) -> PPMWriter {
		let f = File::create(output_filename).unwrap();
		let mut buffered = BufWriter::new(f);
		let max_value = 255;
		
		// Note lack of a newline at the end, write() will add one
		write!(&mut buffered, "P3\n{} {}\n{}", width, height, max_value).unwrap();

		PPMWriter { width: width, x_position: 0, file_handle: buffered }
	}
	
	pub fn write(&mut self, red: u8, green: u8, blue: u8) {
		if self.x_position == 0 {
			// Just finished a line or first line
			write!(&mut self.file_handle, "\n").unwrap();
		}
		
		write!(&mut self.file_handle, "{} {} {} ", red, green, blue).unwrap();
		
		self.x_position = (self.x_position + 1) % self.width;
	}
}