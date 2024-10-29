use gctex;
// use std::io;
// use std::fs;

pub mod image_processing {

}

pub mod bgst_processing {
    use anyhow::{Result, bail};
    use byteorder::{ByteOrder, BigEndian};

    const HEADER_SIZE: usize = 0x40;
    const COMPRESSED_IMAGE_SIZE: usize = 0x20000;

    /// A stripped-down version of the header found
    /// in BGST files. Unknown fields are named based
    /// on the file offset in a BGST file.
    /// 
    /// ### Fields
    /// - `_unk_4`: Currently an unknown value.
    /// - `image_width`: The width of every image in the grid, in pixels.
    /// - `image_height`: The height of every image in the grid, in pixels.
    /// - `grid_width`: The number of rows the grid has.
    /// - `grid_height`: The number of columns the grid has.
    /// - `image_count`: The number of images in the file.
    /// - `layer_enabled`: Indicates which of the 12 rendering layers from the game are available to entries.
    /// - `info_offset`: Offset to entry info.
    /// - `image_data_offset`: Offset to the array of compressed image data.
    pub struct Header {
        _unk_4: u32,
        image_width: u32,
        image_height: u32,
        grid_width: u32,
        grid_height: u32,
        image_count: u32,
        layer_enabled: [bool; 12],
        info_offset: usize,
        image_data_offset: usize
    }

    impl Header {
        /// Creates a stripped-down header struct from a file
        /// that has had its header **pre-validated**.
        /// ### Parameters
        /// - `header_contents`: Data containing the raw header.
        pub fn from_validated_header_bytes(
            header_contents: &Vec<u8>
        ) -> Header {

            // this value is often seen to be 0x11
            let _unk_4 = BigEndian::read_u32(&header_contents[4..8]);
            let image_width = BigEndian::read_u32(&header_contents[8..0xC]);
            let image_height = BigEndian::read_u32(&header_contents[0xC..0x10]);
            let grid_width = BigEndian::read_u32(&header_contents[0x10..0x14]);
            let grid_height = BigEndian::read_u32(&header_contents[0x14..0x18]);
            let image_count = BigEndian::read_u32(&header_contents[0x18..0x1C]);
            let mut layer_enabled = [false; 12];
        
            for i in 0..12 {
                layer_enabled[i] = header_contents
                    .get(0x1C + i)
                    .copied()
                    .unwrap_or(0) != 0;
            }

            let info_offset = BigEndian::read_u32(&header_contents[0x28..0x2C]) as usize;
            let image_data_offset = BigEndian::read_u32(&header_contents[0x2C..0x30]) as usize;

            Header {
                _unk_4,
                image_width,
                image_height,
                grid_width,
                grid_height,
                image_count,
                layer_enabled,
                info_offset,
                image_data_offset
            }
        }
    }

    /// Validates a BGST header.
    /// ### Parameters
    /// - `file_contents`: The BGST file to be validated.
    pub fn validate_header(
        file_contents: &Vec<u8>
    ) -> bool {

        if file_contents.len() < HEADER_SIZE {
            return false;
        } else if &file_contents[..4] != b"BGST" {
            return false;
        }

        true
    }

    /// Attempts to return the RGBA of every image.
    /// 
    /// ### Parameters
    /// - `bgst_contents`: The raw data of a bgst3 file.
    pub fn get_raw_images(
        bgst_contents: &Vec<u8>
    ) -> Result<Vec<Vec<u8>>> {

        if !validate_header(&bgst_contents) {
            bail!("file is not a valid BGST file");
        }
        
        let header = Header::from_validated_header_bytes(&bgst_contents);

        // for even numbered indices, the image is (most likely)
        // a CMPR image. for odd numbered indices, the image is
        // (most likely) an I4 mask.
        
        // if this is inaccurate, i guess i'll find out the hard way
        
        let mut result = Vec::new();

        let image_data = Vec::from(&bgst_contents[header.image_data_offset..]);

        for i in 0..header.image_count {
            let data = Vec::from(&image_data[i as usize * COMPRESSED_IMAGE_SIZE..i as usize * COMPRESSED_IMAGE_SIZE + COMPRESSED_IMAGE_SIZE]);
            
            if i % 2 == 0 {
                // even
                let decoded = gctex::decode(
                    &data,
                    header.image_width,
                    header.image_height,
                    gctex::TextureFormat::CMPR,
                    &Vec::new(),
                    0
                );

                result.push(decoded);
            } else {
                // odd
                let decoded = gctex::decode(
                    &data,
                    header.image_width,
                    header.image_height,
                    gctex::TextureFormat::I4,
                    &Vec::new(),
                    0
                );

                result.push(decoded);
            }
        }

        Ok(result)
    } 
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn invalid_bgst() {
        assert!(
            !bgst_processing::validate_header(&vec![])
        );
    }

    // #[test]
    // fn header_only() {
    //     let header = vec![
    //         0x42, 0x47, 0x53, 0x54, 0x00, 0x00, 0x00, 0x11,
    //         0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x00,
    //         0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x03,
    //         0x00, 0x00, 0x00, 0x42, 0x01, 0x00, 0x01, 0x00,
    //         0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x40, 0x00, 0x00, 0x04, 0x00, 0x3F, 0x80,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //     ];

    //     assert!(bgst_processing::get_raw_images(&header).is_ok());
    // }
}
