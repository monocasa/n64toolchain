extern crate byteorder;
extern crate n64toolchain;

use std::fs;
use std::io;

use std::io::Read;
use std::io::Write;

use byteorder::{WriteBytesExt, BigEndian};

const DEFAULT_CART_TIMING: u32 = 0x80371240;
const DEFAULT_CLOCK_RATE: u32  = 0x0000000f;

const NAME_LEN: usize = 20;

const HEADER_START: usize = 0;
const HEADER_LEN: u64 = 64;
const HEADER_END: usize = HEADER_START + (HEADER_LEN as usize);

const BOOTCODE_START: usize = HEADER_LEN as usize;
const BOOTCODE_LEN: u64 = 4096 - HEADER_LEN;
const BOOTCODE_END: usize = BOOTCODE_START + (BOOTCODE_LEN as usize);

const LOAD_START: usize = (HEADER_LEN + BOOTCODE_LEN) as usize;
const LOAD_LEN: u64 = 0x100000;

const ROM_LEN: usize = (HEADER_LEN + BOOTCODE_LEN + LOAD_LEN) as usize;

#[allow(dead_code)]
struct RomHeader {
	cart_timing: u32,
	clock_rate: u32,
	load_addr: u32,
	release: u32,
	crc1: u32,
	crc2: u32,
	rsvd_18: u32,
	rsvd_1c: u32,
	name: [u8; NAME_LEN],
	rsvd_34: u32,
	manuf_id: u32,
	cart_id: u16,
	country_code: u16,
}

impl RomHeader {
	fn new() -> RomHeader {
		RomHeader {
			cart_timing: DEFAULT_CART_TIMING,
			clock_rate: DEFAULT_CLOCK_RATE,
			load_addr: 0,
			release: 0,
			crc1: 0,
			crc2: 0,
			rsvd_18: 0,
			rsvd_1c: 0,
			name: [0u8; NAME_LEN],
			rsvd_34: 0,
			manuf_id: 0,
			cart_id: 0,
			country_code: 0,
		}
	}

	fn write(&self, writer: &mut io::Write) -> Result<(), io::Error> {
		writer.write_u32::<BigEndian>(self.cart_timing)?;
		writer.write_u32::<BigEndian>(self.clock_rate)?;
		writer.write_u32::<BigEndian>(self.load_addr)?;
		writer.write_u32::<BigEndian>(self.release)?;
		writer.write_u32::<BigEndian>(self.crc1)?;
		writer.write_u32::<BigEndian>(self.crc2)?;
		writer.write_u32::<BigEndian>(self.rsvd_18)?;
		writer.write_u32::<BigEndian>(self.rsvd_1c)?;
		for name_char in self.name.iter() {
			writer.write_u8(*name_char)?;
		}
		writer.write_u32::<BigEndian>(self.rsvd_34)?;
		writer.write_u32::<BigEndian>(self.manuf_id)?;
		writer.write_u16::<BigEndian>(self.cart_id)?;
		writer.write_u16::<BigEndian>(self.country_code)?;

		Ok(())
	}
}

fn usage() {
	println!("Usage:  buildrom64 BOOTCODE_FILE LOAD_BASE_ADDR LOAD_IMAGE");
	std::process::exit(1);
}

fn parse_u32(input_string: &str) -> Result<u32, std::num::ParseIntError> {
	let (base, string) = if input_string.starts_with("0x") {
		(16, &input_string[2..])
	} else {
		(10, input_string)
	};

	u32::from_str_radix(string, base)
}

fn open_input_file(filename: &str) -> Result<(fs::File, u64), io::Error> {
	let file = fs::File::open(filename)?;
	let len = file.metadata()?.len();

	Ok((file, len))
}

fn main() {
	let args: Vec<_> = std::env::args().collect();

	if args.len() != 5 {
		println!("{}", args.len());
		usage();
	}

	let (mut bootcode_file, bootcode_file_len) = open_input_file(&args[1]).unwrap();
	if bootcode_file_len != BOOTCODE_LEN {
		println!("ERROR:  Length of bootcode_file {} != {} bytes", &args[1], BOOTCODE_LEN);
		std::process::exit(1);
	}

	let base_addr = parse_u32(&args[2]).unwrap();

	let (mut load_file, load_file_len) = open_input_file(&args[3]).unwrap();
	if load_file_len > LOAD_LEN {
		println!("ERROR:  Length of load file {} > {} bytes", &args[1], LOAD_LEN);
		std::process::exit(1);
	}

	let mut rom = vec![0xFFu8; ROM_LEN];

	bootcode_file.read(&mut rom[BOOTCODE_START..BOOTCODE_END]).unwrap();
	load_file.read(&mut rom[LOAD_START..load_file_len as usize]).unwrap();

	let (crc1, crc2) = n64toolchain::cart::calculate_cart_checksum(&rom).unwrap();

	let mut rom_header = RomHeader::new();
	rom_header.load_addr = base_addr;
	rom_header.crc1 = crc1;
	rom_header.crc2 = crc2;

	{
		let mut cursor = io::Cursor::new(&mut rom[HEADER_START..HEADER_END]);

		rom_header.write(&mut cursor).unwrap();
	}

	let mut output_file = fs::File::create(&args[4]).unwrap();

	output_file.write(&rom).unwrap();
}

