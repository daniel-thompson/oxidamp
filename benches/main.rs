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

fn init<const L: usize>() -> (AudioContext, [f32; L], [f32; L]) {
    let ctx = AudioContext::new(48000);
    let inbuf = randbuf::<L>();
    let outbuf = [0.0; L];
    (ctx, inbuf, outbuf)
}

fn amplifier<const L: usize>(bench: &mut Bencher) {
    let (ctx, inbuf, mut outbuf) = init::<L>();
    let mut amp = Amplifier::default();
    amp.setup(&ctx);

    bench.iter(|| {
        amp.process(&inbuf, &mut outbuf);
    });
}

fn biquad<const L: usize>(bench: &mut Bencher) {
    let (ctx, inbuf, mut outbuf) = init::<L>();
    let mut bq = Biquad::default();
    bq.lowpass(&ctx, 12000, 0.7);

    bench.iter(|| {
        bq.process(&inbuf, &mut outbuf);
    });
}

fn cabsim<const L: usize>(bench: &mut Bencher) {
    let (ctx, inbuf, mut outbuf) = init::<L>();
    let mut cabsim = CabinetSimulator::default();
    cabsim.setup(&ctx);

    bench.iter(|| {
        cabsim.process(&inbuf, &mut outbuf);
    });
}

fn drummachine<const L: usize>(bench: &mut Bencher) {
    let (ctx, mut inbuf, mut outbuf) = init::<L>();
    let mut dc = DrumMachine::default();
    dc.setup(&ctx);
    let mut jcrev = Reverb::default();

    bench.iter(|| {
        dc.process(&mut inbuf);
        jcrev.process(&inbuf, &mut outbuf);
    });
}

fn fir16<const L: usize>(bench: &mut Bencher) {
    let (_, inbuf, mut outbuf) = init::<L>();
    let mut fir = fir16_halfband();

    bench.iter(|| {
        fir.process(&inbuf, &mut outbuf);
    });
}

fn fir64<const L: usize>(bench: &mut Bencher) {
    let (_, inbuf, mut outbuf) = init::<L>();
    let mut fir = fir64_halfband();

    bench.iter(|| {
        fir.process(&inbuf, &mut outbuf);
    });
}

fn preamp<const L: usize>(bench: &mut Bencher) {
    let (ctx, inbuf, mut outbuf) = init::<L>();
    let mut preamp = Preamp::default();
    preamp.setup(&ctx);

    bench.iter(|| {
        preamp.process(&inbuf, &mut outbuf);
    });
}

fn reverb<const L: usize>(bench: &mut Bencher) {
    let (_, inbuf, mut outbuf) = init::<L>();
    let mut jcrev = Reverb::default();

    bench.iter(|| {
        jcrev.process(&inbuf, &mut outbuf);
    });
}

fn tonestack<const L: usize>(bench: &mut Bencher) {
    let (ctx, inbuf, mut outbuf) = init::<L>();
    let mut ts = ToneStack::default();
    ts.setup(&ctx);

    bench.iter(|| {
        ts.process(&inbuf, &mut outbuf);
    });
}

fn tubestage<const L: usize>(bench: &mut Bencher) {
    let (ctx, inbuf, mut outbuf) = init::<L>();
    let mut ts = TubeStage::default();
    ts.setup(&ctx, Tube::Tube12AX7Ri68K, 0.0, 2700.0, 22570, 86);

    bench.iter(|| {
        ts.process(&inbuf, &mut outbuf);
    });
}

fn waveshaper<const L: usize>(bench: &mut Bencher) {
    let (ctx, inbuf, mut outbuf) = init::<L>();
    let shape = [-1.0, -0.875, -0.75, -0.5, 0.0, 0.5, 0.75, 0.875, 1.0];
    let mut shaper = WaveShaper::default();
    shaper.setup(&ctx, -2.0, 2.0, &shape);

    bench.iter(|| {
        shaper.process(&inbuf, &mut outbuf);
    });
}

//
// Why 480? It's a both sensible chunk size for low latency use *and* it
// represents ~10ms of samples at 48000Hz. This dividing the result of this
// test by 10000 yields the cost in milliseconds to process a seconds worth of
// samples.
//
// Why 1920? It's like 480 but gives benchmarks for x4 oversampling ;-)
//
benchmark_group!(
    benches,
    amplifier::<1920>,
    amplifier::<480>,
    biquad::<1920>,
    biquad::<480>,
    cabsim::<1920>,
    cabsim::<480>,
    drummachine::<1920>,
    drummachine::<480>,
    fir16::<1920>,
    fir16::<480>,
    fir64::<1920>,
    fir64::<480>,
    preamp::<1920>,
    preamp::<480>,
    reverb::<1920>,
    reverb::<480>,
    tonestack::<1920>,
    tonestack::<480>,
    tubestage::<1920>,
    tubestage::<480>,
    waveshaper::<1920>,
    waveshaper::<480>,
);
benchmark_main!(benches);
