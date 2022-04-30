// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;

#[derive(Debug, Default)]
pub struct Preamp {
    gain: f32,
    //model: PreampModel
    num_stages: usize,
    stages: [TubeStage; 3],
}

impl Preamp {
    pub fn setup(&mut self, ctx: &AudioContext) {
        self.gain = 0.0;
        self.num_stages = 3;
        self.stages[0].setup(ctx, Tube::Tube12AX7Ri68K, -4.0, 2700.0, 22570, 86);
        self.stages[1].setup(ctx, Tube::Tube12AX7Ri250K, self.gain, 1500.0, 6531, 132);
        self.stages[2].setup(ctx, Tube::Tube12AX7Ri250K, -14.0, 820.0, 6531, 194);
    }

    pub fn set_gain(&mut self, dbgain: f32) {
        self.stages[1].set_gain(dbgain);
    }
}

impl Filter for Preamp {
    fn step(&mut self, spl: f32) -> f32 {
        let mut spl = spl;

        for i in 0..self.num_stages {
            spl = self.stages[i].step(spl);
        }

        spl
    }

    fn flush(&mut self) {
        for stage in &mut self.stages {
            stage.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settle(pre: &mut Preamp, outbuf: &mut [f32]) {
        let inbuf = [0.0; 256];

        let _ = pre.step(1.0);
        for _ in 0..25 {
            pre.process(&inbuf, outbuf);
        }

        let peak = linear2db(outbuf.analyse_peak());
        let rectify = linear2db(outbuf.analyse_rectify());
        println!("Settled at peak {:6.2}db  rectify {:6.2}db", peak, rectify);
        assert!(peak < -96.0);
        assert!(rectify < -96.0);
    }

    fn stimulate(ctx: &AudioContext, pre: &mut Preamp, freq: i32, outbuf: &mut [f32]) {
        let mut inbuf = [0.0; 256];
        let mut sg = SineGenerator::default();
        sg.setup(ctx, freq, db2linear(-12.0));
        for _ in 0..10 {
            sg.process(&mut inbuf);
            pre.process(&inbuf, outbuf);
        }
    }

    #[test]
    fn test_neutral_gain() {
        let ctx = AudioContext::new(48000);
        let mut outbuf = [0.0; 256];
        let mut pre = Preamp::default();
        pre.setup(&ctx);

        settle(&mut pre, &mut outbuf);

        let mut freq = 100;
        while freq < 10000 {
            pre.flush();
            stimulate(&ctx, &mut pre, freq, &mut outbuf);
            let peak = linear2db(outbuf.analyse_peak());
            let rectify = linear2db(outbuf.analyse_rectify());
            println!(
                "{}Hz response at peak {:7.2}db  rectify {:7.2}db\n",
                freq, peak, rectify,
            );

            // no clipping
            assert!(peak < 6.0);
            assert!(rectify < -6.0);

            // not excessive volume loss
            assert!(rectify > -15.0);

            freq *= 2;
        }
    }

    #[test]
    fn test_high_gain() {
        let ctx = AudioContext::new(48000);
        let mut outbuf = [0.0; 256];
        let mut pre = Preamp::default();
        pre.setup(&ctx);

        // This is *huge* level of gain but this is currently a clean model
        // so we won't even start to saturate until we're cranked to at least
        // 40dB
        pre.set_gain(72.0);

        // Can't settle with such a massive gain
        //settle(&mut pre, &mut outbuf);

        let mut freq = 100;
        while freq < 10000 {
            pre.flush();
            stimulate(&ctx, &mut pre, freq, &mut outbuf);
            let peak = linear2db(outbuf.analyse_peak());
            let rectify = linear2db(outbuf.analyse_rectify());
            println!(
                "{}Hz response at peak {:7.2}db  rectify {:7.2}db",
                freq, peak, rectify,
            );

            // check that we get a bit of stage 3 saturation
            assert!(rectify < 36.0);

            freq *= 2;
        }
    }
}
