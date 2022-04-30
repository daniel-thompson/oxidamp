// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;

struct TubeTable {
    min_vi: i32,
    max_vi: i32,
    _mapping: u32,
    table: [f32; 1001],
}

include!("12AX7.rs");

pub enum Tube {
    Tube12AX7Ri68K,
    Tube12AX7Ri250K,
}

#[derive(Debug, Default)]
pub struct TubeStage {
    vp: f32,
    predivide: f32,

    gain: f32,
    input_filter: Biquad,
    tube: WaveShaper<'static>,
    feedback_filter: Biquad,
    output_filter: DCBlocker,

    bias: f32,
}

impl TubeStage {
    pub fn setup(
        &mut self,
        ctx: &AudioContext,
        tube: Tube,
        dbgain: f32,
        rk: f32,
        input_cutoff: i32,
        feedback_cutoff: i32,
    ) {
        let table = match tube {
            Tube::Tube12AX7Ri68K => &TUBE_12AX7_RI68K,
            Tube::Tube12AX7Ri250K => &TUBE_12AX7_RI250K,
        };

        // TODO: these should be looked up from the tube model
        self.vp = 250.0;
        let rp = 100000.0;
        self.tube
            .setup(ctx, table.min_vi as f32, table.max_vi as f32, &table.table);

        self.predivide = rk / rp;
        self.gain = db2linear(dbgain);

        self.input_filter.lowpass(ctx, input_cutoff, 0.7);
        self.feedback_filter.lowpass(ctx, feedback_cutoff, 0.7);
        self.output_filter.setup(ctx);
    }

    pub fn set_gain(&mut self, dbgain: f32) {
        self.gain = db2linear(dbgain);
    }
}

impl Filter for TubeStage {
    fn step(&mut self, spl: f32) -> f32 {
        let mut spl = spl;

        spl *= self.gain;
        spl = self.input_filter.step(spl);
        spl += self.bias;
        spl = self.tube.step(spl);

        let mut feedback = self.vp - spl;
        feedback *= self.predivide;
        feedback = self.feedback_filter.step(feedback);
        self.bias = feedback;

        self.output_filter.step(spl)
    }

    fn flush(&mut self) {
        self.input_filter.flush();
        self.feedback_filter.flush();
        self.output_filter.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settle(ts: &mut TubeStage, outbuf: &mut [f32]) {
        let inbuf = [0.0; 256];

        let _ = ts.step(1.0);
        for _ in 0..25 {
            ts.process(&inbuf, outbuf);
        }

        let peak = linear2db(outbuf.analyse_peak());
        let rectify = linear2db(outbuf.analyse_rectify());
        println!("peak {:6.2}db  rectify {:6.2}db", peak, rectify);
        assert!(peak < -95.0); // -96.0?
        assert!(rectify < -96.0);
    }

    fn stimulate(ctx: &AudioContext, ts: &mut TubeStage, outbuf: &mut [f32]) {
        let mut inbuf = [0.0; 256];
        let mut sg = SineGenerator::default();
        sg.setup(ctx, 200, db2linear(-12.0));
        for _ in 0..2 {
            sg.process(&mut inbuf);
            ts.process(&inbuf, outbuf);
        }
    }

    #[test]
    fn test_stage1() {
        let ctx = AudioContext::new(48000);
        let mut outbuf = [0.0; 256];
        let mut ts = TubeStage::default();
        ts.setup(&ctx, Tube::Tube12AX7Ri68K, 0.0, 2700.0, 22570, 86);

        settle(&mut ts, &mut outbuf);
        stimulate(&ctx, &mut ts, &mut outbuf);
        let peak = linear2db(outbuf.analyse_peak());
        let rectify = linear2db(outbuf.analyse_rectify());
        println!("peak {:7.2}db  rectify {:7.2}db\n", peak, rectify,);
        assert!(peak > -12.0);
        assert!(rectify > -18.0);
    }

    #[test]
    fn test_stage2() {
        let ctx = AudioContext::new(48000);
        let mut outbuf = [0.0; 256];
        let mut ts = TubeStage::default();
        ts.setup(&ctx, Tube::Tube12AX7Ri250K, 0.0, 1500.0, 6531, 132);

        settle(&mut ts, &mut outbuf);
        stimulate(&ctx, &mut ts, &mut outbuf);
        let peak = linear2db(outbuf.analyse_peak());
        let rectify = linear2db(outbuf.analyse_rectify());
        println!("peak {:7.2}db  rectify {:7.2}db\n", peak, rectify,);
        assert!(peak > -12.0);
        assert!(rectify > -12.0);
    }

    #[test]
    fn test_stage3() {
        let ctx = AudioContext::new(48000);
        let mut outbuf = [0.0; 256];
        let mut ts = TubeStage::default();
        ts.setup(&ctx, Tube::Tube12AX7Ri250K, 0.0, 820.0, 6531, 194);

        settle(&mut ts, &mut outbuf);
        stimulate(&ctx, &mut ts, &mut outbuf);
        let peak = linear2db(outbuf.analyse_peak());
        let rectify = linear2db(outbuf.analyse_rectify());
        println!("peak {:7.2}db  rectify {:7.2}db\n", peak, rectify,);
        assert!(peak > -12.0);
        assert!(rectify > -12.0);
    }
}
