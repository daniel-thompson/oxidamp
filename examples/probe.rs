// Probe: the compensated loop should stay bounded across the range, including
// the top notes where the loop loss being cancelled is largest.
use oxidamp::prelude::*;

fn main() {
    let fs = 48000.0f32;
    let ctx = AudioContext::new(fs as i32);
    for &f in &[27.5f32, 440.0, 4186.0] {
        let mut ks = KarplusStrong::default();
        ks.setup(&ctx);
        ks.tune(&ctx, f);
        ks.trigger(1.0);
        ks.set_sustain(10.0);

        let mut peak = 0.0f32;
        let mut out = Vec::new();
        for i in 0..48000 {
            let s = ks.step();
            peak = peak.max(s.abs());
            if i % 4800 == 0 {
                out.push((peak * 1000.0).round() / 1000.0);
                peak = 0.0;
            }
        }
        println!("f={:8.1} peak/0.1s = {:?}", f, out);
    }
}
