use ddc_hi::{Ddc, Display};
use std::env;
use std::process::exit;

// Print usage information for the program.
fn print_usage(program_name: &str) {
    println!("Usage:");
    println!(
        "  {} <brightness>    Set brightness level (0-100) on supported displays",
        program_name
    );
    println!("  {} --help          Print usage information", program_name);
    println!(
        "  {} --status        Check if displays support brightness adjustment",
        program_name
    );
    println!(
        "  {} --get           Get current brightness level",
        program_name
    );
    println!(
        "  {} +/-<number>     Adjust brightness by the specified value (0-100)",
        program_name
    );
}

fn main() {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("No command specified. Type -h or --help for help");
        // print_usage(&args[0]);
        exit(1);
    }

    /*
    env::args() returns an iterator over the command-line arguments passed to the program.
    .collect() converts this iterator into a Vec<String>, a vector containing the arguments as strings.
    args now holds all the command-line arguments, including the program name as the first element.

    if args.len() < 2 {

    This checks if the number of command-line arguments is less than 2.
    args.len() returns the number of elements in the args vector.
    */

    let mut brightness_value = None;
    let mut print_help = false;
    let mut print_status = false;
    let mut get_brightness = false;
    let mut adjust_brightness: Option<i16> = None;

    // Parse arguments
    for arg in &args[1..] {
        match arg.as_str() {
            "--help" => print_help = true,
            "-h" => print_help = true,
            "--status" => print_status = true,
            "-s" => print_status = true,
            "--get" => get_brightness = true,
            "-g" => get_brightness = true,
            value if value.starts_with('+') || value.starts_with('-') => {
                if let Ok(delta) = value.parse::<i16>() {
                    adjust_brightness = Some(delta);
                } else {
                    eprintln!("Invalid adjustment value '{}'.", value);
                    print_usage(&args[0]);
                    exit(1);
                }
            }
            value => {
                if brightness_value.is_some() {
                    eprintln!("Error: Unexpected argument '{}'.", value);
                    print_usage(&args[0]);
                    exit(1);
                }
                brightness_value = Some(value.to_string());
            }
        }
    }

    if print_help {
        print_usage(&args[0]);
        exit(0);
    }

    if print_status {
        // Check if displays support brightness adjustment via DDC/CI
        let displays = Display::enumerate();
        for mut display in displays {
            match display.handle.get_vcp_feature(0x10) {
                Ok(_) => println!(
                    "Display {:?} supports brightness adjustment via DDC/CI.",
                    display.info.model_name
                ),
                Err(_) => println!(
                    "Display {:?} does not support brightness adjustment via DDC/CI.",
                    display.info.model_name
                ),
            }
        }
        exit(0);
    }

    if get_brightness {
        // Retrieve and print the current brightness level
        let displays = Display::enumerate();
        for mut display in displays {
            match display.handle.get_vcp_feature(0x10) {
                Ok(value) => println!("{}", value.value()),
                Err(_) => println!(
                    "Failed to get brightness for display {:?}",
                    display.info.model_name
                ),
            }
        }
        exit(0);
    }

    if let Some(delta) = adjust_brightness {
        // Adjust brightness by the specified delta value
        let mut displays = Display::enumerate();
        for display in &mut displays {
            match display.handle.get_vcp_feature(0x10) {
                Ok(current_value) => {
                    let new_brightness =
                        (current_value.value() as i16 + delta).clamp(0, 100) as u16;
                    match display.handle.set_vcp_feature(0x10, new_brightness) {
                        Ok(_) => println!(
                            "Brightness adjusted to {} on display {:?}",
                            new_brightness, display.info.model_name
                        ),
                        Err(err) => eprintln!(
                            "Failed to set brightness on display {:?}: {:?}",
                            display.info.model_name, err
                        ),
                    }
                }
                Err(_) => eprintln!(
                    "Failed to get current brightness for display {:?}",
                    display.info.model_name
                ),
            }
        }
        exit(0);
    }

    // If neither --help nor --status nor --get was specified, handle brightness adjustment
    let new_brightness: u16 = match brightness_value.as_ref().unwrap().parse() {
        Ok(value) => value,
        Err(_) => {
            eprintln!(
                "Invalid brightness value: {}",
                brightness_value.as_ref().unwrap()
            );
            exit(1);
        }
    };

    // Ensure the brightness is within the valid range (0-100)
    if new_brightness > 100 {
        eprintln!("Brightness value must be between 0 and 100.");
        exit(1);
    }

    // Retrieve all connected displays that support DDC/CI
    let mut displays = Display::enumerate();

    if displays.is_empty() {
        eprintln!("No displays supporting DDC/CI found.");
        exit(1);
    }

    // Iterate through each display and set the brightness
    for display in &mut displays {
        match display.handle.set_vcp_feature(0x10, new_brightness) {
            Ok(_) => println!(
                "Brightness set to {} on display {:?}",
                new_brightness, display.info.model_name
            ),
            Err(err) => eprintln!(
                "Failed to set brightness on display {:?}: {:?}",
                display.info.model_name, err
            ),
        }
    }
}

