
use std::{fs::File, io::Write, path::PathBuf};

use base64::{engine::general_purpose, Engine};
use image::{EncodableLayout, DynamicImage};
use clap::{Parser, ValueEnum};



#[derive(ValueEnum, Debug, Clone, Copy)]
enum EncodingMethod {
    BC1 = 4,
    RGB888 = 5
}

impl EncodingMethod {

    pub fn byte_size(self, width: usize, height: usize) -> usize {
        match self {
            EncodingMethod::BC1 => texpresso::Format::Bc1.compressed_size(width, height),
            EncodingMethod::RGB888 => width * height * 3,
        }
    }

    pub fn encode(self, img: &DynamicImage) -> Vec<u8> {

        let width: usize = img.width().try_into().unwrap();
        let height: usize = img.height().try_into().unwrap();

        match self {
            EncodingMethod::BC1 => {

                let size = texpresso::Format::Bc1.compressed_size(width, height);
                let mut encoded = vec![0u8; size];

                texpresso::Format::Bc1.compress(
                    img.to_rgba8().as_bytes(),
                    width,
                    height,
                    texpresso::Params {
                        algorithm: texpresso::Algorithm::IterativeClusterFit,
                        weights: [ 0.2126, 0.7152, 0.0722 ],
                        weigh_colour_by_alpha: false
                    },
                    &mut encoded
                );

                encoded

            },
            EncodingMethod::RGB888 => {

                img.to_rgb8().as_bytes().to_vec()

            }
        }
    }

}







/// Encode an image that gmod_e2_image.txt can decode in-game.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Command {

    /// Image path to encode
    img_path: PathBuf,

    /// The image encoding method to use
    #[arg(short, long, value_enum, default_value_t = EncodingMethod::BC1)]
    encoding: EncodingMethod,

}




fn main() -> std::io::Result<()> {

    let cli = Command::parse();



    println!("Loading image {:?}", cli.img_path);

    let img = image::open(cli.img_path).unwrap();

    if img.width() > 0xFFFF || img.height() > 0xFFFF {
        println!("Image is too big.");

        return Ok(());
    }



    // // I don't know what filtering method to use, I just use Lanczos3 because it sounds the most fancy.
    // img = img.resize(resize.width, resize.height, image::imageops::FilterType::Lanczos3);



    let header = vec![
        u8::from((img.width() & 0xFF) as u8),
        u8::from(((img.width() >> 8) & 0xFF) as u8),
        u8::from((img.height() & 0xFF) as u8),
        u8::from(((img.height() >> 8) & 0xFF) as u8),
        u8::from(cli.encoding as u8),
    ];



    println!("Encoding image to {:?} format", cli.encoding);

    let encoded = cli.encoding.encode(&img);



    let output_file: PathBuf = PathBuf::from("output.txt");

    println!("Writing to file {:?}", output_file);

    let mut file = File::create(output_file)?;

    let base64_stream = general_purpose::STANDARD.encode([ header, encoded ].concat());

    if base64_stream.len() > 200_000 {
        println!("WARNING: Image stream is large, pasting into e2 may take a while.");
    }

    file.write_all(format!("Base64Stream = \"{}\"", base64_stream).as_bytes())?;



    return Ok(());

}


