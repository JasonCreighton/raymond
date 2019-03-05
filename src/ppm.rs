use std::fs::File;
use std::io;
use std::io::Write;

pub struct PPMWriter {
    file_handle: io::BufWriter<File>,
}

impl PPMWriter {
    pub fn new(output_filename: &str, width: i32, height: i32) -> io::Result<PPMWriter> {
        let f = File::create(output_filename)?;
        let mut buffered = io::BufWriter::new(f);
        let max_value = 255;

        // The trailing space is important, there should only be a single whitespace
        // between the header and the binary image data
        write!(&mut buffered, "P6\n{} {}\n{} ", width, height, max_value)?;

        Ok(PPMWriter {
            file_handle: buffered,
        })
    }

    pub fn write(&mut self, red: u8, green: u8, blue: u8) -> io::Result<()> {
        self.file_handle.write_all(&[red, green, blue])
    }
}
