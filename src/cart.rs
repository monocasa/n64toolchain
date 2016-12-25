use std::io::{Cursor, Error, ErrorKind};

use std::fmt;

use byteorder::{ReadBytesExt, BigEndian};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ByteSwapping {
	Native,
	U16LittleEndian,
}

impl fmt::Display for ByteSwapping {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ByteSwapping::Native          => write!(f, "Native"),
			ByteSwapping::U16LittleEndian => write!(f, "U16 Little Endian"),
		}
	}
}

#[derive(Debug, Eq, PartialEq)]
pub enum ChecksumError {
	NotLongEnough,
	ErrorReadingBuffer,
}

pub fn detect_swapping(buffer: &[u8]) -> Option<ByteSwapping> {
	if buffer.len() < 4 {
		return None;
	}

	return match (buffer[0], buffer[1], buffer[2], buffer[3]) {
		(0x80, 0x37, 0x12, 0x40) => Some(ByteSwapping::Native),
		(0x37, 0x80, 0x40, 0x12) => Some(ByteSwapping::U16LittleEndian),
		(   _,    _,    _,    _) => None,
	};
}

pub fn swap_cart_to(new_swapping: ByteSwapping, buffer: &mut [u8]) -> Result<(), Error> {
	let original_swapping = match detect_swapping(buffer) {
		Some(swapping) => swapping,
		None => {
			return Err(Error::new(ErrorKind::Other, "Unknown original byte swapping"));
		},
	};

	if (buffer.len() % 2) != 0 {
		return Err(Error::new(ErrorKind::Other, "Not an even length for swapping"));
	}

	if original_swapping == new_swapping {
		return Ok(());
	}

	for ii in 0..(buffer.len() / 2) {
		let cur_base = (ii * 2) as usize;
		let temp = buffer[cur_base];
		buffer[cur_base] = buffer[cur_base + 1];
		buffer[cur_base + 1] = temp;
	}

	Ok(())
}

const CHECKSUM_START:  usize =   0x1000;
const CHECKSUM_LENGTH: usize = 0x100000;
const CHECKSUM_END: usize = CHECKSUM_START + CHECKSUM_LENGTH;
const CHECKSUM_START_VALUE: u32 = 0xf8ca4ddc;

fn rol_u32(i: u32, b: u32) -> u32 {
	if b >= 32 {
		println!("rol_u32(i: {:#x}, b: {})", i, b);
	}

	let lhs = i << b;

	let rhs: u32 = if b == 0 {
		0
	} else {
		i >> (32 - b)
	};

	return lhs | rhs;
}

pub fn calculate_cart_checksum(buffer: &[u8]) -> Result<(u32, u32), ChecksumError> {
	if buffer.len() < CHECKSUM_END {
		return Err(ChecksumError::NotLongEnough);
	}

	let checksum_slice = &buffer[CHECKSUM_START..CHECKSUM_END];

	let mut reader = Cursor::new(checksum_slice);

	let mut c1: u32;
	let mut k1: u32;
	let mut k2: u32;

	let mut t1 = CHECKSUM_START_VALUE;
	let mut t2 = CHECKSUM_START_VALUE;
	let mut t3 = CHECKSUM_START_VALUE;
	let mut t4 = CHECKSUM_START_VALUE;
	let mut t5 = CHECKSUM_START_VALUE;
	let mut t6 = CHECKSUM_START_VALUE;

	for _ in 0..(CHECKSUM_LENGTH / 4) {
		c1 = match reader.read_u32::<BigEndian>() {
			Ok(value) => value,
			Err(_) => {
				return Err(ChecksumError::ErrorReadingBuffer);
			},
		};

		k1 = t6.wrapping_add(c1);
		if k1 < t6 {
			t4 += 1;
		}
		t6 = k1;
		t3 ^= c1;
		k2 = c1 & 0x1f;
		k1 = rol_u32(c1, k2);
		t5 = t5.wrapping_add(k1);
		if c1 < t2 {
			t2 ^= k1;
		} else {
			t2 ^= t6 ^ c1;
		}
		t1 = t1.wrapping_add(c1 ^ t5);
	}

	return Ok((
		t6 ^ t4 ^ t3, 
		t5 ^ t2 ^ t1))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}

