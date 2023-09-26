use crate::morpher::Morpher;

pub struct Processor {
    morpher: Morpher,
}
impl Processor {
    pub fn new() -> Self {
        Self {
            morpher: Morpher::new(),
        }
    }

    pub fn process(
        &mut self,
        ch0: &mut [f32],
        ch1: &[f32],
        morph_k: &[f32],
        aux_spectral_spread: f32,
        iter_count: i32,
    ) {
        debug_assert_eq!(ch0.len(), ch1.len());
        debug_assert_eq!(ch0.len(), morph_k.len());
        let hop_length = self.morpher.hop_length();
        let n_chunks = ch0.len() / hop_length;
        let overflow = ch0.len() % hop_length;
        assert!(overflow == 0);

        for n in 0..n_chunks {
            let range = n * hop_length..(n + 1) * hop_length;
            let out = self.morpher.morph(
                &ch0[range.clone()],
                &ch1[range.clone()],
                morph_k[n * hop_length],
                aux_spectral_spread,
                iter_count,
            );
            ch0[range].copy_from_slice(&out);
        }
    }
}
