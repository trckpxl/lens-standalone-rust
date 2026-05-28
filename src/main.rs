use std::env;

use lens_standalone::LensClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let copy_to_clipboard = args.iter().any(|arg| arg == "--clip");
    let save_to_text = args.iter().any(|arg| arg == "--text");
    let copy_translation = args.iter().any(|arg| arg == "--tl");
    let filtered_args: Vec<&String> = args.iter().skip(1)
        .filter(|arg| *arg != "--clip" && *arg != "--text" && *arg != "--tl")
        .collect();

    if filtered_args.is_empty() {
        eprintln!("Usage: {} <path_to_image> [--clip] [--text] [--tl]", args[0]);
        return Ok(());
    }

    let image_path = filtered_args[0];

    let client = LensClient::new(None);

    let silent = copy_to_clipboard || save_to_text || copy_translation;

    if !silent {
        println!("Processing image: {}", image_path);
    }
    match client.process_image_path(image_path, Some("en")).await {
        Ok(result) => {
            if !silent {
                println!("--- Full Text ---");
                println!("{}", result.full_text);

                println!("\n--- Detailed Structure ---");
                println!("Found {} paragraphs.", result.paragraphs.len());
                for (i, para) in result.paragraphs.iter().enumerate() {
                    println!("Paragraph {}: {} lines", i + 1, para.lines.len());
                    if let Some(geom) = para.lines.first().and_then(|line| line.geometry.as_ref()) {
                        println!(
                            "  -> First line pos: x={:.2}, y={:.2}, w={:.2}",
                            geom.center_x, geom.center_y, geom.width
                        );
                    }
                }

                if let Some(trans) = &result.translation {
                    println!("\n--- Translation ---");
                    println!("{}", trans);
                }
                println!("------------------");
            }

            if copy_to_clipboard {
                match arboard::Clipboard::new() {
                    Ok(mut ctx) => {
                        if let Err(e) = ctx.set_text(result.full_text.clone()) {
                            eprintln!("Error copying to clipboard: {}", e);
                        } else if !silent {
                            println!("Result successfully copied to clipboard!");
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to initialize clipboard context: {}", e);
                    }
                }
            }

            if copy_translation {
                match &result.translation {
                    Some(trans) => {
                        match arboard::Clipboard::new() {
                            Ok(mut ctx) => {
                                if let Err(e) = ctx.set_text(trans.clone()) {
                                    eprintln!("Error copying translation to clipboard: {}", e);
                                } else if !silent {
                                    println!("Translation successfully copied to clipboard!");
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to initialize clipboard context: {}", e);
                            }
                        }
                    }
                    None => {
                        eprintln!("Warning: No translation was found to copy to clipboard.");
                    }
                }
            }

            if save_to_text {
                use std::fs::File;
                use std::io::Write;
                use std::path::Path;

                let output_path = Path::new(image_path).with_extension("txt");
                match File::create(&output_path) {
                    Ok(mut file) => {
                        if let Err(e) = file.write_all(result.full_text.as_bytes()) {
                            eprintln!("Error writing to file: {}", e);
                        } else if !silent {
                            println!("Result successfully saved to {:?}", output_path);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error creating file: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }

    Ok(())
}
