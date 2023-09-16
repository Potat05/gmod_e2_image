
use std::{fs::File, io::Write};

use base64::{engine::general_purpose, Engine};
use image::EncodableLayout;


fn main() -> std::io::Result<()> {

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 5 {
        println!("Usage:");
        println!("gmod_e2_image.exe RESIZE_WIDTH RESIZE_HEIGHT FORMAT FILE_PATH");

        return Ok(());
    }



    let resize_width = &args[1].parse::<u32>().unwrap();
    let resize_height = &args[2].parse::<u32>().unwrap();
    let format = &args[3];
    let filepath = &args[4];



    let mut img = image::open(filepath).unwrap();

    if img.width() > 0xFFFF || img.height() > 0xFFFF {
        println!("Image is too big.");

        return Ok(());
    }



    // I don't know what filtering method to use, I just use Lanczos3 because it sounds the most fancy.
    img = img.resize(resize_width.to_owned(), resize_height.to_owned(), image::imageops::FilterType::Lanczos3);
    let rgba8888 = img.into_rgba8();



    let encoded = (match format.as_str() {
        "bc1" => {
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
            Ok(buf)
        },
        "rgb888" => {
            let buf = Vec::from(rgba8888.as_bytes());
            Ok(buf)
        }
        _ => {
            println!("Unknown format \"{}\"", format);
            Err(())
        },
    }).unwrap();

    let header = vec![
        u8::from((rgba8888.width() & 0xFF) as u8),
        u8::from(((rgba8888.width() >> 8) & 0xFF) as u8),
        u8::from((rgba8888.height() & 0xFF) as u8),
        u8::from(((rgba8888.height() >> 8) & 0xFF) as u8),
        (match format.as_str() {
            "bc1" => Ok(4),
            "rgb888" => Ok(5),
            _ => {
                println!("Unknown format \"{}\"", format);
                Err(())
            },
        }).unwrap()
    ];



    static OUTPUTFILE: &str = "output.txt";

    println!("Output to file \"{}\"", OUTPUTFILE);

    let mut file = File::create(OUTPUTFILE)?;

    let base64_stream = general_purpose::STANDARD.encode([ header, encoded ].concat());

    file.write_all(format!("Base64Stream = \"{}\"", base64_stream).as_bytes())?;



    return Ok(());

}


