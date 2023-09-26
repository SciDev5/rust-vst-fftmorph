use rustfft::{
    num_complex::{Complex32, ComplexFloat},
    FftPlanner,
};

use crate::util::{lerpable::Lerpable, ring_buffer::RingBuffer};

/// Morphs two single-channel audio signals together
pub struct Morpher {
    fft_fwd: std::sync::Arc<dyn rustfft::Fft<f32>>,
    fft_inv: std::sync::Arc<dyn rustfft::Fft<f32>>,

    window_func: Vec<f32>,
    window_size: usize,
    hop_length: usize,

    input_buf_a: RingBuffer<f32>,
    input_buf_b: RingBuffer<f32>,
    output_buf: RingBuffer<(f32, f32)>,

    proc_buf: (Vec<Complex32>, Vec<Complex32>),
    phase_accum: Vec<(f32, f32)>,
    phase_prev: Vec<(f32, f32)>,
    // mag_faded: Vec<(f32, f32)>,
    mag_prev: Vec<(f32, f32)>,
}

fn cosine_window_fn(window_size: usize) -> Vec<f32> {
    let mut output = vec![0.0; window_size];
    for i in 0..window_size {
        output[i] =
            1.0 - ((i + 1) as f32 / (window_size + 1) as f32 * std::f32::consts::PI * 2.0).cos();
    }
    output
}

impl Morpher {
    pub fn new() -> Self {
        const DEFAULT_WINDOW_SIZE: usize = 1024;
        const DEFAULT_HOP_LENGTH: usize = 256;
        let window_size = DEFAULT_WINDOW_SIZE;
        let hop_length = DEFAULT_HOP_LENGTH;
        let mut fft_planner = FftPlanner::new();
        let fft_fwd = fft_planner.plan_fft_forward(window_size);
        let fft_inv = fft_planner.plan_fft_inverse(window_size);
        Self {
            fft_fwd,
            fft_inv,

            window_size,
            hop_length,

            window_func: cosine_window_fn(window_size),

            input_buf_a: RingBuffer::new(window_size, 0.0),
            input_buf_b: RingBuffer::new(window_size, 0.0),
            output_buf: RingBuffer::new(window_size, (0.0, 0.0)),

            proc_buf: (
                vec![Complex32::default(); window_size],
                vec![Complex32::default(); window_size],
            ),

            phase_accum: vec![(0.0, 0.0); window_size],
            phase_prev: vec![(0.0, 0.0); window_size],
            // mag_faded: vec![(0.0, 0.0); window_size],
            mag_prev: vec![(0.0, 0.0); window_size],
        }
    }

    pub fn hop_length(&self) -> usize {
        self.hop_length
    }

    fn put_inputs(&mut self, a: &[f32], b: &[f32]) {
        debug_assert_eq!(a.len(), self.hop_length);
        debug_assert_eq!(b.len(), self.hop_length);

        self.input_buf_a.push_clone_from_slice(a);
        self.input_buf_b.push_clone_from_slice(b);
    }
    fn take_outputs(&mut self) -> Vec<f32> {
        const MIN_WINDOW_FACTOR_SUM: f32 = 0.5;
        let mut out = vec![0.0; self.hop_length];
        let (lower, upper) = self.output_buf.slice_raw_mut(0, self.hop_length as isize);
        for (i, (wave, window_factor_sum)) in lower.iter_mut().chain(upper.iter_mut()).enumerate() {
            out[i] = *wave / window_factor_sum.max(MIN_WINDOW_FACTOR_SUM);
            // reset for next time.
            *wave = 0.0;
            *window_factor_sum = 0.0;
        }
        self.output_buf.shift(self.hop_length as isize);
        out
    }

    fn take_windowed_input(
        window_func: &Vec<f32>,
        input_buf: &RingBuffer<f32>,
        windowed_inputs: &mut [Complex32],
    ) {
        let window_size = window_func.len();
        debug_assert_eq!(input_buf.len(), window_size);
        debug_assert_eq!(windowed_inputs.len(), window_size);

        let (lower, upper) = input_buf.slice_raw(0, window_size as isize);

        for (i, value) in lower.iter().chain(upper.iter()).enumerate() {
            windowed_inputs[i] = (window_func[i] * value).into();
        }
    }

    /// Morph one `hop_length` of samples.
    pub fn morph(
        &mut self,
        a: &[f32],
        b: &[f32],
        k_morph: f32,
        k_fade: f32,
        _aux_spectral_spread: f32,
        _iter_count: i32,
    ) -> Vec<f32> {
        self.put_inputs(a, b);

        // <load> input
        // input *= window_fn
        Self::take_windowed_input(
            &self.window_func,
            &mut self.input_buf_a,
            &mut self.proc_buf.0,
        );
        Self::take_windowed_input(
            &self.window_func,
            &mut self.input_buf_b,
            &mut self.proc_buf.1,
        );

        // (mag, phase) = fft(input)   [unnormalized]
        self.fft_fwd.process(&mut self.proc_buf.0);
        self.fft_fwd.process(&mut self.proc_buf.1);

        // # morphing interpolation
        for i in 0..self.window_size {
            // const BIN_MAGNITUDE_FADE_COEFFICIENTS: (f32, f32) = (0.0, 0.6);

            // (mag, phase)
            let phase = (self.proc_buf.0[i].arg(), self.proc_buf.1[i].arg());
            let mag = (self.proc_buf.0[i].abs(), self.proc_buf.1[i].abs());

            // phase_delta = phase - phase_prev
            let phase_delta = (
                phase.0 - self.phase_prev[i].0,
                phase.1 - self.phase_prev[i].1,
            );
            // self.mag_faded[i] = BIN_MAGNITUDE_FADE_COEFFICIENTS.lerp(mag, self.mag_faded[i]);

            // phase_accum += lerp<k>(phase_delta[..])
            self.phase_accum[i].0 += k_morph.lerp(phase_delta.0, phase_delta.1); // A -> B
            self.phase_accum[i].1 += k_morph.lerp(phase_delta.1, phase_delta.0); // B -> A

            // if (mag/mag_prev > 10) phase_accum = phase
            //// for A -> B morph
            if mag.0 / self.mag_prev[i].0 * (1.0 - k_morph) > 10.0 {
                self.phase_accum[i].0 = phase.0
            }
            if mag.1 / self.mag_prev[i].1 * (k_morph) > 10.0 {
                self.phase_accum[i].0 = phase.1
            }
            //// for B -> A morph
            if mag.1 / self.mag_prev[i].1 * (1.0 - k_morph) > 10.0 {
                self.phase_accum[i].1 = phase.1
            }
            if mag.0 / self.mag_prev[i].0 * (k_morph) > 10.0 {
                self.phase_accum[i].1 = phase.0
            }

            // ...prev = ...current
            self.phase_prev[i] = phase;
            self.mag_prev[i] = mag;

            // reconstructed = complex(r= mag_fade, theta= phase_accum)
            //// for A -> B morph
            self.proc_buf.0[i] = Complex32::from_polar(
                // k.lerp(self.mag_faded[i].0, self.mag_faded[i].1), !!!!!!!!!!!!!!!!!!!!!!!
                mag.0.powf((1.0 - k_morph).sqrt()) * mag.1.powf(k_morph.sqrt()),
                self.phase_accum[i].0,
            );
            //// for B -> A morph
            self.proc_buf.1[i] = Complex32::from_polar(
                // k.lerp(self.mag_faded[i].1, self.mag_faded[i].0), !!!!!!!!!!!!!!!!!!!!!!!
                mag.1.powf((1.0 - k_morph).sqrt()) * mag.0.powf(k_morph.sqrt()),
                self.phase_accum[i].1,
            );
        }

        // output_windowed = real(ifft(reconstructed)) * window_fn / window_size
        // window_factor_sum += window_fn^2
        self.fft_inv.process(&mut self.proc_buf.0);
        self.fft_inv.process(&mut self.proc_buf.1);
        for i in 0..self.window_size {
            let (wave, window_factor_sum) = &mut self.output_buf[i as isize];
            *wave += k_fade.lerp(self.proc_buf.0[i].re, self.proc_buf.1[i].re)
                * self.window_func[i]
                / self.window_size as f32;
            *window_factor_sum += self.window_func[i] * self.window_func[i];
        }

        self.take_outputs()
    }
}
