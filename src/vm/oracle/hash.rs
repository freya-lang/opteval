pub(crate) fn hash(source_data: impl IntoIterator<Item = u8>) -> [u8; 32] {
	let mut source_data = source_data.into_iter();
	let mut state = [0x0000080100cc0002, 0, 0, 0, 0];

	ascon_12(&mut state);

	loop {
		let mut buffer = [0; 8];
		let mut i = 0;

		for byte in &mut source_data {
			buffer[i] = byte;
			i += 1;

			if i == 8 {
				break;
			}
		}

		let mut ending = false;
		if i < 8 {
			buffer[i] = 0x01;
			ending = true;
		}

		state[0] ^= u64::from_le_bytes(buffer);
		ascon_12(&mut state);

		if ending {
			break;
		}
	}

	let h0 = state[0];
	ascon_12(&mut state);
	let h1 = state[0];
	ascon_12(&mut state);
	let h2 = state[0];
	ascon_12(&mut state);
	let h3 = state[0];

	let mut out = [0; 32];

	out[0 .. 8].copy_from_slice(&h0.to_le_bytes());
	out[8 .. 16].copy_from_slice(&h1.to_le_bytes());
	out[16 .. 24].copy_from_slice(&h2.to_le_bytes());
	out[24 .. 32].copy_from_slice(&h3.to_le_bytes());

	out
}

fn get_const(i: u8) -> u64 {
	u64::from(0xf0 - 0x0f * i)
}

fn substitution(state: &mut [u64; 5]) {
	state[0] ^= state[4];

	state[2] ^= state[1];
	state[4] ^= state[3];

	let t0 = !state[0] & state[1];
	let t1 = !state[1] & state[2];
	let t2 = !state[2] & state[3];
	let t3 = !state[3] & state[4];
	let t4 = !state[4] & state[0];

	state[0] ^= t1;
	state[1] ^= t2;
	state[2] ^= t3;
	state[3] ^= t4;
	state[4] ^= t0;

	state[1] ^= state[0];
	state[3] ^= state[2];

	state[0] ^= state[4];

	state[2] ^= u64::MAX;
}

fn diffusion(state: &mut [u64; 5]) {
	state[0] ^= state[0].rotate_right(19) ^ state[0].rotate_right(28);
	state[1] ^= state[1].rotate_right(61) ^ state[1].rotate_right(39);
	state[2] ^= state[2].rotate_right(1) ^ state[2].rotate_right(6);
	state[3] ^= state[3].rotate_right(10) ^ state[3].rotate_right(17);
	state[4] ^= state[4].rotate_right(7) ^ state[4].rotate_right(41);
}

fn ascon_12(state: &mut [u64; 5]) {
	for i in 0 .. 12 {
		state[2] ^= get_const(i);
		substitution(state);
		diffusion(state);
	}
}
