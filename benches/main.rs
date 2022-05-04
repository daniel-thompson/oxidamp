// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

#[macro_use]
extern crate bencher;

use bencher::Bencher;
use oxidamp::*;
use rand::prelude::*;

fn randbuf<const L: usize>() -> [f32; L] {
    let mut rng = thread_rng();
    let mut inbuf = [0.0_f32; L];
    for i in &mut inbuf {
        *i = rng.gen_range(-1.0..1.0);
    }
    inbuf
}

fn amplifier<const L: usize>(bench: &mut Bencher) {
    let inbuf = randbuf::<L>();
    let mut outbuf = [0.0; L];
    let ctx = AudioContext::new(48000);
    let mut amp = Amplifier::default();
    amp.setup(&ctx);

    bench.iter(|| {
        amp.process(&inbuf, &mut outbuf);
    });
}

fn biquad<const L: usize>(bench: &mut Bencher) {
    let inbuf = randbuf::<L>();
    let mut outbuf = [0.0; L];
    let ctx = AudioContext::new(48000);
    let mut bq = Biquad::default();
    bq.lowpass(&ctx, 12000, 0.7);

    bench.iter(|| {
        bq.process(&inbuf, &mut outbuf);
    });
}

fn cabsim<const L: usize>(bench: &mut Bencher) {
    let inbuf = randbuf::<L>();
    let mut outbuf = [0.0; L];
    let ctx = AudioContext::new(48000);
    let mut cabsim = CabinetSimulator::default();
    cabsim.setup(&ctx);

    bench.iter(|| {
        cabsim.process(&inbuf, &mut outbuf);
    });
}

fn fir16<const L: usize>(bench: &mut Bencher) {
    let inbuf = randbuf::<L>();
    let mut outbuf = [0.0; L];
    let mut fir = fir16_halfband();

    bench.iter(|| {
        fir.process(&inbuf, &mut outbuf);
    });
}

fn fir64<const L: usize>(bench: &mut Bencher) {
    let inbuf = randbuf::<L>();
    let mut outbuf = [0.0; L];
    let mut fir = fir64_halfband();

    bench.iter(|| {
        fir.process(&inbuf, &mut outbuf);
    });
}

benchmark_group!(
    benches,
    // This is a very special case! 480 is both a sensibly
    // small chunk size *and* represents ~10ms of samples
    // at 48000Hz. Dividing the result of this test by 10000
    // yields the cost in milliseconds to process a seconds
    // worth of samples.
    amplifier::<480>,

    amplifier::<256>,
    amplifier::<1024>,
    amplifier::<4096>,
    biquad::<256>,
    biquad::<1024>,
    biquad::<4096>,
    cabsim::<256>,
    cabsim::<1024>,
    cabsim::<4096>,
    fir16::<256>,
    fir16::<1024>,
    fir16::<4096>,
    fir64::<256>,
    fir64::<1024>,
    fir64::<4096>
);
benchmark_main!(benches);
