use std::env;

use lens_standalone::LensClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let copy_to_clipboard = args.iter().any(|arg| arg == "--clip");
    let filtered_args: Vec<&String> = args.iter().skip(1).filter(|arg| *arg != "--clip").collect();

    if filtered_args.is_empty() {
        eprintln!("Usage: {} <path_to_image> [--clip]", args[0]);
        return Ok(());
    }

    let image_path = filtered_args[0];

    let client = LensClient::new(None);

    println!("Processing image: {}", image_path);
    match client.process_image_path(image_path, Some("en")).await {
        Ok(result) => {
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

            if let Some(trans) = result.translation {
                println!("\n--- Translation ---");
                println!("{}", trans);
            }
            println!("------------------");

            if copy_to_clipboard {
                match arboard::Clipboard::new() {
                    Ok(mut ctx) => {
                        if let Err(e) = ctx.set_text(result.full_text.clone()) {
                            eprintln!("Error copying to clipboard: {}", e);
                        } else {
                            println!("Result successfully copied to clipboard!");
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to initialize clipboard context: {}", e);
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
