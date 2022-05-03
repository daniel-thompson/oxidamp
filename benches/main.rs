// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

#[macro_use]
extern crate bencher;

use bencher::Bencher;
use oxidamp::*;
use rand::prelude::*;

fn randbuf() -> [f32; 1024] {
    let mut rng = thread_rng();
    let mut inbuf = [0.0_f32; 1024];
    for i in &mut inbuf {
        *i = rng.gen_range(-1.0..1.0);
    }
    inbuf
}

fn fir16(bench: &mut Bencher) {
    let inbuf = randbuf();
    let mut outbuf = [0.0; 1024];
    let mut fir = fir16_halfband();

    bench.iter(|| {
        fir.process(&inbuf, &mut outbuf);
    });
}

fn fir64(bench: &mut Bencher) {
    let inbuf = randbuf();
    let mut outbuf = [0.0; 1024];
    let mut fir = fir64_halfband();

    bench.iter(|| {
        fir.process(&inbuf, &mut outbuf);
    });
}

benchmark_group!(benches, fir16, fir64);
benchmark_main!(benches);
