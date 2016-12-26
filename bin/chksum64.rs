extern crate n64toolchain;

use std::io::Read;
use std::io::Write;

fn usage() {
	let args: Vec<_> = std::env::args().collect();

	println!("Usage: {} INPUT_FILE OUTPUT_FILE", args[0]);
	println!("");
	println!("The program calculates the ROM checksum for Nintendo64 ROM images.");
	println!("Checksum code reverse engineered from Nagra's program");

	std::process::exit(1);
}

fn main() {
	let args: Vec<_> = std::env::args().collect();
	let verbose = true;

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

	let original_byte_swapping = match n64toolchain::cart::detect_swapping(buffer.as_slice()) {
		Some(byte_swapping) => byte_swapping,
		None => {
			println!("Error:  Unable to detect byte ordering dynamically from file");
			std::process::exit(1);
		}
	};

	match n64toolchain::cart::swap_cart_to(n64toolchain::cart::ByteSwapping::Native, buffer.as_mut_slice()) {
		Ok(_) => {},
		Err(err) => {
			println!("Error:  Unable to swap binary to native:  {}", err);
			std::process::exit(1);
		}
	}

	let (checksum1, checksum2) = match n64toolchain::cart::calculate_cart_checksum(buffer.as_slice()) {
		Ok(checksum) => checksum,
		Err(err) => {
			println!("Error:  Unable to calculate checksum on file{}:  {:?}", input_filename, err);
			std::process::exit(1);
		}
	};

	if verbose {
		println!("Old Checksum:        {:02x} {:02x} {:02x} {:02x}  {:02x} {:02x} {:02x} {:02x}",
				buffer[0x10], buffer[0x11], buffer[0x12], buffer[0x13],
				buffer[0x14], buffer[0x15], buffer[0x16], buffer[0x17]);
	}

	buffer[0x10] = (checksum1 >> 24) as u8;
	buffer[0x11] = (checksum1 >> 16) as u8;
	buffer[0x12] = (checksum1 >>  8) as u8;
	buffer[0x13] = (checksum1 >>  0) as u8;
	buffer[0x14] = (checksum2 >> 24) as u8;
	buffer[0x15] = (checksum2 >> 16) as u8;
	buffer[0x16] = (checksum2 >>  8) as u8;
	buffer[0x17] = (checksum2 >>  0) as u8;

	if verbose {
		println!("New Checksum:        {:02x} {:02x} {:02x} {:02x}  {:02x} {:02x} {:02x} {:02x}",
				buffer[0x10], buffer[0x11], buffer[0x12], buffer[0x13],
				buffer[0x14], buffer[0x15], buffer[0x16], buffer[0x17]);
	}

	match n64toolchain::cart::swap_cart_to(original_byte_swapping.clone(), buffer.as_mut_slice()) {
		Ok(_) => {},
		Err(err) => {
			println!("Error:  Unable to swap binary to original({}):  {}", original_byte_swapping, err);
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

