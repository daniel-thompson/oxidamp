// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;

#[derive(Debug, Default)]
pub struct Amplifier {
    pub controls: ToneStackControls,

    preamp: Preamp,
    tonestack: ToneStack,
    cabsim: CabinetSimulator,
}

impl Amplifier {
    pub fn setup(&mut self, ctx: &AudioContext) {
        self.preamp.setup(ctx);
        self.tonestack.setup(ctx);
        self.cabsim.setup(ctx);
    }

    pub fn update(&mut self) {
        self.tonestack.controls = self.controls;
        self.tonestack.update();
    }
}

impl Filter for Amplifier {
    fn step(&mut self, spl: f32) -> f32 {
        let mut spl = spl;
        spl = self.preamp.step(spl);
        spl = self.tonestack.step(spl);
        self.cabsim.step(spl)
    }

    fn flush(&mut self) {
        self.preamp.flush();
        self.tonestack.flush();
        self.cabsim.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settle(amp: &mut Amplifier, outbuf: &mut [f32]) {
        let inbuf = [0.0; 256];

        let _ = amp.step(1.0);
        for _ in 0..25 {
            amp.process(&inbuf, outbuf);
        }

        let peak = linear2db(outbuf.analyse_peak());
        let rectify = linear2db(outbuf.analyse_rectify());
        println!("Settled at peak {:6.2}db  rectify {:6.2}db", peak, rectify);
        assert!(peak < -90.0); // TODO: -96dB
        assert!(rectify < -96.0);
    }

    fn stimulate(ctx: &AudioContext, amp: &mut Amplifier, freq: i32, outbuf: &mut [f32]) {
        let mut inbuf = [0.0; 256];
        let mut sg = SineGenerator::default();
        sg.setup(ctx, freq, db2linear(-12.0));
        for _ in 0..10 {
            sg.process(&mut inbuf);
            amp.process(&inbuf, outbuf);
        }
    }

    #[test]
    fn test_defaults() {
        let ctx = AudioContext::new(48000);
        let mut outbuf = [0.0; 256];
        let mut amp = Amplifier::default();
        amp.setup(&ctx);

        settle(&mut amp, &mut outbuf);

        let mut freq = 100;
        while freq < 10000 {
            amp.flush();
            stimulate(&ctx, &mut amp, freq, &mut outbuf);
            let peak = linear2db(outbuf.analyse_peak());
            let rectify = linear2db(outbuf.analyse_rectify());
            println!(
                "{}Hz response at peak {:7.2}db  rectify {:7.2}db\n",
                freq, peak, rectify,
            );

            // no clipping
            assert!(peak < 9.0);
            assert!(rectify < -3.0);

            // not excessive volume loss
            assert!(rectify > -24.0);

            freq *= 2;
        }
    }
}
