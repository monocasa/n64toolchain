extern crate n64toolchain;

use std::io::Read;
use std::io::Write;

fn usage() {
	let args: Vec<_> = std::env::args().collect();

	println!("Usage: {} INPUT_FILE OUTPUT_FILE", args[0]);
	println!("");
	println!("This program converts ROMs into ");

	std::process::exit(1);
}

fn main() {
	let args: Vec<_> = std::env::args().collect();

	if args.len() != 3 {
		usage();
	}

	let input_filename = args[1].clone();
	let output_filename = args[2].clone();

	let mut input_file = match std::fs::File::open(&input_filename) {
		Ok(file) => file,
		Err(err) => {
			println!("Error:  Unable to open input file {}:  {}", input_filename, err);
			std::process::exit(1);
		}
	};

	let mut buffer: Vec<u8> = Vec::new();

	match input_file.read_to_end(&mut buffer) {
		Ok(_) => {},
		Err(err) => {
			println!("Error: Unable to read input file {}:  {}", input_filename, err);
			std::process::exit(1);
		}
	}

	match n64toolchain::cart::swap_cart_to(n64toolchain::cart::ByteSwapping::Native, buffer.as_mut_slice()) {
		Ok(_) => {},
		Err(err) => {
			println!("Error:  Unable to swap binary:  {}", err);
			std::process::exit(1);
		}
	}

	let mut output_file = match std::fs::File::create(&output_filename) {
		Ok(file) => file,
		Err(err) => {
			println!("Error:  Unable to open output file {}:  {}", output_filename, err);
			std::process::exit(1);
		}
	};

	match output_file.write(buffer.as_slice()) {
		Ok(_) => {},
		Err(err) => {
			println!("Error:  Unable to write output file {}:  {}", output_filename, err);
			std::process::exit(1);
		}
	}
}

