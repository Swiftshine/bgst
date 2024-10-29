use anyhow::{Result, bail};
use std::fs;
use std::io::Cursor;
use image::*;
use gctex;
use byteorder::{ByteOrder, BigEndian};


const HEADER_SIZE: usize = 0x40;
const GRID_ENTRY_SIZE: usize = 0x10;
const COMPRESSED_IMAGE_SIZE: usize = 0x20000;

/// A list of layers, described as "scenes" ingame.
enum SceneIndex {
    Far05 = 0,
    Far04 = 1,
    Far03 = 2,
    Far02 = 3,
    Far01 = 4,
    Map = 5,
    Game = 6,
    Near01 = 7,
    Near02 = 8,
    Near03 = 9,
    Near04 = 10,
    Near05 = 11,
}

/// A stripped-down version of the header found
/// in BGST files. Unknown fields are named based
/// on the file offset.
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
struct Header {
    _unk_4: u32,
    image_width: u32,
    image_height: u32,
    _grid_width: u32,
    _grid_height: u32,
    image_count: u32,
    _layer_enabled: [bool; 12],
    info_offset: usize,
    image_data_offset: usize
}

impl Header {
    /// Creates a stripped-down header struct from a file
    /// that has had its header **pre-validated**.
    /// ### Parameters
    /// - `header_contents`: Data containing the raw header.
    /// ### Returns
    /// - a `Header` struct
    pub fn from_validated_header_bytes(
        header_contents: &Vec<u8>
    ) -> Header {
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
            _grid_width: grid_width,
            _grid_height: grid_height,
            image_count,
            _layer_enabled: layer_enabled,
            info_offset,
            image_data_offset
        }
    }
}

/// A structure containing information on entries in the grid.
/// Unknown fields are named based on the file offset.
/// 
/// ### Fields
/// - `enabled`: Indicates whether or not this cell should be shown.
/// - `scene_index`: Indicates which scene index this cell is to be shown on.
/// - `grid_x`: The row in which the cell is rendered.
/// - `grid_y`:The column in which the cell is rendered.
/// - `main_image_index`: The CMPR image this cell will render, if any
/// - `mask_image_index`: The I4 mask this cell will apply to the image, if any
/// - `_unk_c`: Currently an unknown value.
/// - `_unk_e`: Currently an unknown value.
struct GridEntry {
    enabled: i16,
    scene_index: i16,
    grid_x: i16,
    grid_y: i16,
    main_image_index: i16,
    mask_image_index: i16,
    _unk_c: i16,
    _unk_e: i16,
}

impl GridEntry {
    /// Returns if the entry is enabled.
    /// 
    /// ### Returns
    /// `true` if the entry is enabled.
    fn is_enabled(&self) -> bool {
        self.enabled != 0
    }
}



/// A list of compressed or uncompressed images.
    /// ### Fields
    /// - `image_width`: The width of every image, in pixels.
    /// - `image_height`: The height of every image, in pixels.
    /// - `images`: The images.
struct ImageList {
    image_width: u32,
    image_height: u32,
    grid_entries: Vec<GridEntry>,
    images: Vec<Vec<u8>>
} 



pub mod bgst_processing {
    use super::*;

    /// A list of compressed or uncompressed images.
    /// ### Fields
    /// - `image_width`: The width of every image, in pixels.
    /// - `image_height`: The height of every image, in pixels.
    /// - `images`: The images.
    pub struct ImageList {
        image_width: u32,
        image_height: u32,
        grid_entries: Vec<GridEntry>,
        images: Vec<Vec<u8>>
    }

    

    /// Validates a BGST header.
    /// ### Parameters
    /// - `file_contents`: The BGST file to be validated.
    /// ### Returns
    /// - `true` if the given header was valid
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

    pub fn apply_mask(
        main_image: &Vec<u8>,
        mask_image: &Vec<u8>,
        width: u32,
        height: u32
    ) -> Result<Vec<u8>> {
        if main_image.len() != mask_image.len() {
            bail!("the image sizes are not equal!");
        }
    
        // decode the main and mask images from raw rgba bytes

        let main_img: RgbaImage = ImageBuffer::from_raw(width, height, main_image.clone())
            .ok_or_else(|| anyhow::anyhow!("failed to decode main image"))?;
        let mask_img: RgbaImage = ImageBuffer::from_raw(width, height, mask_image.clone())
            .ok_or_else(|| anyhow::anyhow!("failed to decode mask image"))?;
    
        let mut output_img = RgbaImage::new(width, height);
    

        for (x, y, pixel) in output_img.enumerate_pixels_mut() {
            let main_pixel = main_img.get_pixel(x, y);
            let mask_pixel = mask_img.get_pixel(x, y);
    
            // if the mask pixel is black (r=0, g=0, b=0), set alpha of main image to 0
            if mask_pixel[0] == 0 && mask_pixel[1] == 0 && mask_pixel[2] == 0 {
                *pixel = Rgba([main_pixel[0], main_pixel[1], main_pixel[2], 0]); // make transparent
            } else {
                *pixel = *main_pixel; // keep original pixel
            }
        }
    
        let output_bytes = output_img.into_raw();
    
        Ok(output_bytes)
    }
    
    /// Attempts to return the RGBA of every image.
    /// ### Parameters
    /// - `bgst_contents`: The raw data of a bgst3 file.
    /// ### Returns
    /// - an `ImageList` struct
    pub fn get_raw_images(
        bgst_contents: &Vec<u8>
    ) -> Result<ImageList> {

        if !validate_header(&bgst_contents) {
            bail!("file is not a valid BGST file");
        }
        
        let header = Header::from_validated_header_bytes(&bgst_contents);

        let mut grid_entries = Vec::new();
        
        let mut current_offset = header.info_offset;

        while current_offset < header.image_data_offset {
            let enabled = BigEndian::read_i16(&bgst_contents[current_offset..current_offset + 2]);
            let scene_index = BigEndian::read_i16(&bgst_contents[current_offset + 2..current_offset + 4]);
            let grid_x = BigEndian::read_i16(&bgst_contents[current_offset + 4..current_offset + 6]);
            let grid_y = BigEndian::read_i16(&bgst_contents[current_offset + 6..current_offset + 8]);
            let main_image_index = BigEndian::read_i16(&bgst_contents[current_offset + 8..current_offset + 0xA]);
            let mask_image_index = BigEndian::read_i16(&bgst_contents[current_offset + 0xA..current_offset + 0xC]);
            let _unk_c = BigEndian::read_i16(&bgst_contents[current_offset + 0xC..current_offset + 0xE]);
            let _unk_e = BigEndian::read_i16(&bgst_contents[current_offset + 0xE..current_offset + 0x10]);

            let entry = GridEntry {
                enabled,
                scene_index,
                grid_x,
                grid_y,
                main_image_index,
                mask_image_index,
                _unk_c,
                _unk_e
            };

            grid_entries.push(entry);

            current_offset += GRID_ENTRY_SIZE;
        }

              
        let mut images = Vec::new();

        let image_data = Vec::from(&bgst_contents[header.image_data_offset..]);

        for i in 0..grid_entries.len() {
            let entry = &grid_entries[i];

            if entry.main_image_index > -1 && entry.main_image_index < header.image_count as i16 {
                let encoded = Vec::from(&image_data[entry.main_image_index as usize * COMPRESSED_IMAGE_SIZE..entry.main_image_index as usize * COMPRESSED_IMAGE_SIZE + COMPRESSED_IMAGE_SIZE]);
                let decoded = gctex::decode(
                    &encoded,
                    header.image_width,
                    header.image_height,
                    gctex::TextureFormat::CMPR,
                    &Vec::new(),
                    0
                );

                images.push(decoded);
            }

            if entry.mask_image_index > -1 && entry.mask_image_index < header.image_count as i16 {
                let encoded = Vec::from(&image_data[entry.mask_image_index as usize * COMPRESSED_IMAGE_SIZE..entry.mask_image_index as usize * COMPRESSED_IMAGE_SIZE + COMPRESSED_IMAGE_SIZE]);
                let decoded = gctex::decode(
                    &encoded,
                    header.image_width,
                    header.image_height,
                    gctex::TextureFormat::I4,
                    &Vec::new(),
                    0
                );

                images.push(decoded);
            }
        }

        let result = ImageList {
            image_width: header.image_width,
            image_height: header.image_height,
            grid_entries,
            images
        };

        Ok(result)
    } 

    fn get_png_images(
        raw_images: &ImageList
    ) -> Result<Vec<Vec<u8>>> {
        let mut result = Vec::new();

        for raw_image in &raw_images.images {
            if let Some(img) = RgbaImage::from_raw(
                raw_images.image_width,
                raw_images.image_height,
                raw_image.to_owned()
            ) {
                let mut buffer = Cursor::new(Vec::new());

                img.write_to(&mut buffer, ImageFormat::Png)?;

                result.push(buffer.into_inner());
            }
        }

        Ok(result)
    }

    pub fn extract_bgst(
        input_filename: &str
    ) -> Result<()> {

        println!("checking if file exists...");

        if !fs::exists(input_filename).unwrap() {
            bail!(format!("file {} does not exist", input_filename));
        }

        let file_contents = fs::read(input_filename)?;

        println!("validating header...");

        if !validate_header(&file_contents) {
            bail!(format!("file {} is not a valid BGST file", input_filename));
        }

        println!("extracting raw images...");
        let raw_image_list = get_raw_images(&file_contents)?;

        println!("converting to png...");

        let png_images = get_png_images(&raw_image_list)?;

        println!("writing files...");

        let folder_name = input_filename
            .strip_suffix(".bgst3")
            .unwrap()
            .to_string();

        match fs::exists(&folder_name) {
            Ok(folder_exists) => {
                if !folder_exists {
                    let _ = fs::create_dir(&folder_name);
                }
            }

            Err(_) => {
                let _ = fs::create_dir(&folder_name);
            }
        }


        for i in 0..png_images.len() {
            let filename = folder_name.to_owned() + "/" + i.to_string().as_str() + ".png";
            println!("\twriting file {filename}");
            let _ = fs::write(
                String::from(filename),
                &png_images[i]
            );
        }

        println!("done!");

        Ok(())
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
