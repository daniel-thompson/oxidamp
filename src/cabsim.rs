// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

/// Simple biquad based cabinet simulator.
///
/// Starting point was based on a frequency response graph of the condor from
/// runoffgroove. Nothing else comes from the condor; the "implementation"
/// is just curve fitting. Nevertheless I wanted to honour those who came
/// before by preserving the name.
///
/// ~~~
/// fiview 48000 -i \
///   PkBq/0.7/-16/400 x \
///   HsBq/0.7/6/400 x \
///   HpBq/0.7/60 x \
///   LpBq/0.7/4000 x LpBq/0.7/4000
/// ~~~
///
/// The result is five biquads:
///  * Partial notch filter at 400Hz (peaking EQ) (-16dB)
///  * High boosting shelf filter at 400Hz (6dB)
///  * High pass at 60Hz
///  * 2 x low pass at 4000Hz
#[derive(Debug, Default)]
pub struct CabinetSimulator {
    notch: Biquad,
    shelf: Biquad,
    hpf: Biquad,
    lpf0: Biquad,
    lpf1: Biquad,
}

impl CabinetSimulator {
    pub fn setup(&mut self, ctx: &AudioContext) {
        self.notch.peakingeq(ctx, 400, -16, 0.7);
        self.highshelf.highshelf(ctx, 400, 6, 0.7);
        self.hpf.highpass(ctx, 60, 0.7);
        self.lpf0.lowpass(ctx, 4000, 0.7);
        self.lpf1.lowpass(ctx, 4000, 0.7);
    }

    pub fn step(&mut self, spl: f32) -> f32 {
        spl = self.notch.step(spl);
        spl = self.shelf.step(spl);
        spl = self.hpf.step(spl);
        spl = self.lpf0.step(spl);
        self.lpf1.step(spl)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::zip;

    #[test]
    fn test_cabsim_step() {
        let ctx = AudioContext::new(48000);
        let mut sg = SignalGenerator::default();
        let mut cabsim = CabinetSimulator::default();
        let mut inbuf = [0.0_f32; 1024];
        let mut outbuf = [0.0_f32; 1024];

        sg.setup(&ctx, 400, 1.570793);
        cabsim.setup(&ctx);
        
        for (inspl, outspl) in zip(&mut inbuf, &mut outbuf) {
            inspl = sg.sin();
            outspl = cabsim.step(inspl);
        }

        assert!(inbuf[0] == 0.0);
        assert!(inbuf[1023] != 0.0);
        assert!(outbuf[0] == 0.0);
    }

