use rustfft::{
    num_complex::{Complex32, ComplexFloat},
    num_traits::Zero,
    FftPlanner,
};

use crate::util::lerpable::Lerpable;

pub struct Morpher {
    fft_planner: FftPlanner<f32>,
}

impl Morpher {
    pub fn new() -> Self {
        Self {
            fft_planner: FftPlanner::new(),
        }
    }
    fn p(
        window_size: usize,
        scale: f32,
        cx_a: &Vec<Complex32>,
        cx_b: &Vec<Complex32>,
        k: f32,

        aux_spectral_spread: f32,
        iter_count: i32,
    ) -> Vec<Complex32> {
        let mut cx_out = Vec::<Complex32>::with_capacity(window_size);

        let b_abs = cx_b.iter().map(|v|v.abs()).collect::<Vec<f32>>();
        let z = (50.0 * aux_spectral_spread) as i32 + 1;

        for i in 0..window_size {
            // cx_out.push(Complex32::from(cx_a[i].abs() * cx_b[i].abs() / (scale * scale)));
            // cx_out.push(cx_a[i] * cx_b[i].abs() / (scale * scale));

            let a = cx_a[i] / scale;
            let b = cx_b[i] / scale;

            let theta = if a.is_zero() {
                if b.is_zero() {
                    0.0
                } else {
                    b.arg()
                }
            } else {
                if b.is_zero() {
                    a.arg()
                } else {
                    k.lerp(a.arg(), b.arg())
                }
            };
            let mut a = a.abs();
            let mut b = 0.0;
            // let mut mag = 0.0;
            for j in ((i as i32 - z) as usize) .. ((i as i32 + z) as usize).min(window_size - 1) {
                // let f = 1.0 ;//- ((i as f32 - j as f32).abs() / z as f32);
                b += b_abs[j];// * f;
                // mag += f;
            }
            b /= scale * z as f32 * 2.0;
            // let r = k.lerp(a, b);
            // let r = if k > 0.5 {
            //     (k*2.0-1.0).lerp(a*b, b)
            // } else {
            //     (k*2.0).lerp(a, a*b)
            // };
            // const V:f32 = 0.00001;
            const V:f32 = 1.0e-16;
            for _ in 0 .. iter_count {
                a = k.lerp((a+V).ln(), (b+V).ln()).exp()-V;
            }
            cx_out.push(Complex32::from_polar(a, theta));
        }

        cx_out
    }
    pub fn morph_2x(&mut self, a: &[f32], b: &[f32], k: f32, aux_spectral_spread: f32, iter_count: i32) -> (Vec<f32>, Vec<f32>) {
        assert_eq!(a.len(), b.len());
        let window_size = a.len();
        let scale = (a.len() as f32).sqrt();
        // let scale_sqr = a.len() as f32;

        let fft_fwd = self.fft_planner.plan_fft_forward(window_size);
        let fft_inv = self.fft_planner.plan_fft_inverse(window_size);

        let mut cx_a = a
            .into_iter()
            .map(Complex32::from)
            .collect::<Vec<Complex32>>();
        let mut cx_b = b
            .into_iter()
            .map(Complex32::from)
            .collect::<Vec<Complex32>>();

        fft_fwd.process(&mut cx_a);
        fft_fwd.process(&mut cx_b);

        let mut cx_out_ab = Self::p(window_size, scale, &cx_a, &cx_b, k, aux_spectral_spread, iter_count);
        let mut cx_out_ba = Self::p(window_size, scale, &cx_a, &cx_b, 1.0 - k, aux_spectral_spread, iter_count);

        fft_inv.process(&mut cx_out_ab);
        fft_inv.process(&mut cx_out_ba);

        let mut out_ab = vec![0.0; window_size];
        let mut out_ba = vec![0.0; window_size];

        for i in 0..window_size {
            out_ab[i] = cx_out_ab[i].re / scale;
            out_ba[i] = cx_out_ba[i].re / scale;
        }
        (out_ab, out_ba)
    }

    
    pub fn morph(&mut self, a: &[f32], b: &[f32], k: f32, aux_spectral_spread: f32, iter_count: i32) -> Vec<f32> {
        assert_eq!(a.len(), b.len());
        let window_size = a.len();
        let scale = (a.len() as f32).sqrt();
        // let scale_sqr = a.len() as f32;

        let fft_fwd = self.fft_planner.plan_fft_forward(window_size);
        let fft_inv = self.fft_planner.plan_fft_inverse(window_size);

        let mut cx_a = a
            .into_iter()
            .map(Complex32::from)
            .collect::<Vec<Complex32>>();
        let mut cx_b = b
            .into_iter()
            .map(Complex32::from)
            .collect::<Vec<Complex32>>();

        fft_fwd.process(&mut cx_a);
        fft_fwd.process(&mut cx_b);

        let mut cx_out = Self::p(window_size, scale, &cx_a, &cx_b, k, aux_spectral_spread, iter_count);

        fft_inv.process(&mut cx_out);

        let mut out = vec![0.0; window_size];

        for i in 0..window_size {
            out[i] = cx_out[i].re / scale;
        }
        out
    }
}

#[cfg(test)]
mod test {
    use rustfft::{num_complex::Complex32, FftPlanner};

    #[test]
    fn a() {
        const L: usize = 5;
        let mut p = FftPlanner::<f32>::new();
        let fwd = p.plan_fft_forward(L);
        let inv = p.plan_fft_inverse(L);

        let mut k = vec![
            Complex32::from(0.0),
            Complex32::from(1.0),
            Complex32::from(2.0),
            Complex32::from(3.0),
            Complex32::from(4.0),
        ];
        let mut sc = vec![
            Complex32::from(0.0),
            Complex32::from(1.0),
            Complex32::from(0.0),
            Complex32::from(3.0),
            Complex32::from(0.0),
        ];
        let z = k.as_mut_slice();
        let zs = sc.as_mut_slice();
        l(z);

        fwd.process(z);
        fwd.process(zs);
        let lr = (L as f32).sqrt();

        for i in 0..L {
            zs[i] /= lr;
            z[i] /= lr;
            z[i] *= zs[i];
        }

        l(z);

        inv.process(z);

        for i in 0..L {
            z[i] /= lr;
        }

        l(z);
    }
    fn l(z: &[Complex32]) {
        println!(":::::::: L={}", z.len());
        for v in z {
            println!("{}", v);
        }
    }
}
