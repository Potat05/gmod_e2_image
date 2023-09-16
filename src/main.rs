
use std::{fs::File, io::Write, path::PathBuf};

use base64::{engine::general_purpose, Engine};
use image::EncodableLayout;
use clap::{Parser, ValueEnum};



#[derive(ValueEnum, Debug, Clone)]
enum EncodingMethod {
    BC1 = 4,
    RGB888 = 5
}

// // TODO: Get this to work
// #[derive(Args, Debug, Clone)]
// struct Resize {
//     width: u32,
//     height: u32,
// }

/// Encode an image that gmod_e2_image.txt can decode in-game.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Command {

    /// Image path to encode
    img_path: PathBuf,

    /// The image encoding method to use
    #[arg(short, long, value_enum, default_value_t = EncodingMethod::BC1)]
    encoding: EncodingMethod,

    // /// Resize image
    // #[arg(short, long)] 
    // resize: Option<Resize>,

}




fn main() -> std::io::Result<()> {

    let cli = Command::parse();



    let img = image::open(cli.img_path).unwrap();

    if img.width() > 0xFFFF || img.height() > 0xFFFF {
        println!("Image is too big.");

        return Ok(());
    }



    // if Option::is_some(&cli.resize) {
    //     let resize = cli.resize.unwrap();
    //     // I don't know what filtering method to use, I just use Lanczos3 because it sounds the most fancy.
    //     img = img.resize(resize.width, resize.height, image::imageops::FilterType::Lanczos3);
    // }
    let rgba8888 = img.into_rgba8();



    let encoded = match cli.encoding {
        EncodingMethod::BC1 => {
            let size = texpresso::Format::Bc1.compressed_size(rgba8888.width().try_into().unwrap(), rgba8888.height().try_into().unwrap());
            let mut buf = vec![0u8; size];
            texpresso::Format::Bc1.compress(
                rgba8888.as_bytes(),
                rgba8888.width().try_into().unwrap(),
                rgba8888.height().try_into().unwrap(),
                texpresso::Params {
                    algorithm: texpresso::Algorithm::IterativeClusterFit,
                    weights: [ 0.2126, 0.7152, 0.0722 ],
                    weigh_colour_by_alpha: false
                },
                &mut buf
            );
            buf
        },
        EncodingMethod::RGB888 => {
            let buf = Vec::from(rgba8888.as_bytes());
            buf
        },
    };

    let header = vec![
        u8::from((rgba8888.width() & 0xFF) as u8),
        u8::from(((rgba8888.width() >> 8) & 0xFF) as u8),
        u8::from((rgba8888.height() & 0xFF) as u8),
        u8::from(((rgba8888.height() >> 8) & 0xFF) as u8),
        u8::from(cli.encoding as u8),
    ];



    static OUTPUTFILE: &str = "output.txt";

    println!("Output to file \"{}\"", OUTPUTFILE);

    let mut file = File::create(OUTPUTFILE)?;

    let base64_stream = general_purpose::STANDARD.encode([ header, encoded ].concat());

    file.write_all(format!("Base64Stream = \"{}\"", base64_stream).as_bytes())?;



    return Ok(());

}


