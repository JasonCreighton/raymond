use std::fs::File;
use std::io::{BufWriter, Write};

pub struct PPMWriter {
	file_handle: BufWriter<File>,
}

impl PPMWriter {
	pub fn new(output_filename: &str, width: i32, height: i32) -> PPMWriter {
		let f = File::create(output_filename).unwrap();
		let mut buffered = BufWriter::new(f);
		let max_value = 255;
		
		// The trailing space is important, there should only be a single whitespace
		// between the header and the binary image data
		write!(&mut buffered, "P6\n{} {}\n{} ", width, height, max_value).unwrap();

		PPMWriter { file_handle: buffered }
	}
	
	pub fn write(&mut self, red: u8, green: u8, blue: u8) {
		self.file_handle.write(&[red, green, blue]).unwrap();
	}
}