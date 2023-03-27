use crate::{dbug, morpher::Morpher, util::lerpable::Lerpable};

const PROCESS_LEN: usize = 2048;

pub struct Processor {
    stored_a: Vec<f32>,
    stored_b: Vec<f32>,
    prevnext_morphed_ab: Vec<f32>,
    prevnext_morphed_ba: Vec<f32>,
    chunk_len: usize,

    morpher: Morpher,
}
impl Processor {
    pub fn new() -> Self {
        Self {
            stored_a: vec![0.0; PROCESS_LEN],
            stored_b: vec![0.0; PROCESS_LEN],
            prevnext_morphed_ab: vec![],
            prevnext_morphed_ba: vec![],
            chunk_len: 0,

            morpher: Morpher::new(),
        }
    }
    fn set_chunk_len(&mut self, chunk_len: usize) {
        if chunk_len != self.chunk_len {
            self.prevnext_morphed_ab = vec![0.0; chunk_len];
            self.prevnext_morphed_ba = vec![0.0; chunk_len];
            self.chunk_len = chunk_len;
        }
    }

    fn shift_into_stored(&mut self, a: &[f32], b: &[f32]) {
        let len = a.len();
        if len >= PROCESS_LEN {
            dbug::________BREAKPOINT_________("len >= PROCESS_LEN, aborting because I'm lazy");
            panic!("too lazy to do this");
        } else {
            self.stored_a.drain(0..len);
            self.stored_b.drain(0..len);
            self.stored_a.extend(a);
            self.stored_b.extend(b);
        }
    }

    pub fn process_2x(&mut self, a: &mut [f32], b: &[f32], morph_k: &[f32], fade_k: &[f32], z: &[f32]) {
        assert_eq!(a.len(), b.len());
        self.set_chunk_len(a.len());

        self.shift_into_stored(a, b);

        let k = morph_k
            .iter()
            .map(|v| *v)
            .reduce(|a, b| a + b)
            .unwrap_or(0.0)
            / fade_k.len() as f32;

        let (morphed_ab, morphed_ba) =
            self.morpher
                .morph_2x(&self.stored_a[..], &self.stored_b[..], k, z[0]);

        const OFF: usize = 400;
        let current_morphed_ab = morphed_ab
            [(PROCESS_LEN - self.chunk_len * 2 - OFF)..(PROCESS_LEN - self.chunk_len - OFF)]
            .to_vec();
        let current_morphed_ba = morphed_ba
            [(PROCESS_LEN - self.chunk_len * 2 - OFF)..(PROCESS_LEN - self.chunk_len - OFF)]
            .to_vec();

        for i in 0..self.chunk_len {
            let k = ((i as f32 / self.chunk_len as f32) * 8.0).min(1.0);
            let morphed_ab = k.lerp(self.prevnext_morphed_ab[i], current_morphed_ab[i]);
            let morphed_ba = k.lerp(self.prevnext_morphed_ba[i], current_morphed_ba[i]);

            // a is the output channel
            a[i] = fade_k[i].lerp(morphed_ab, morphed_ba);
        }
        self.prevnext_morphed_ab =
            morphed_ab[(PROCESS_LEN - self.chunk_len - OFF)..(PROCESS_LEN - OFF)].to_vec();
        self.prevnext_morphed_ba =
            morphed_ba[(PROCESS_LEN - self.chunk_len - OFF)..(PROCESS_LEN - OFF)].to_vec();
    }

    pub fn process(&mut self, a: &mut [f32], b: &[f32], morph_k: &[f32], z: &[f32]) {
        assert_eq!(a.len(), b.len());
        self.set_chunk_len(a.len());

        self.shift_into_stored(a, b);

        let k = morph_k
            .iter()
            .map(|v| *v)
            .reduce(|a, b| a + b)
            .unwrap_or(0.0)
            / morph_k.len() as f32;

        let morphed = self
            .morpher
            .morph(&self.stored_a[..], &self.stored_b[..], k, z[0]);

        const OFF: usize = 400;
        let current_morphed = morphed
            [(PROCESS_LEN - self.chunk_len * 2 - OFF)..(PROCESS_LEN - self.chunk_len - OFF)]
            .to_vec();

        for i in 0..self.chunk_len {
            let k = i as f32 / self.chunk_len as f32;
            let morphed = k.lerp(self.prevnext_morphed_ab[i], current_morphed[i]);

            // a is the output channel
            a[i] = morphed;
        }
        self.prevnext_morphed_ab =
            morphed[(PROCESS_LEN - self.chunk_len - OFF)..(PROCESS_LEN - OFF)].to_vec();
    }
}

#[cfg(test)]
mod test {
    use super::Processor;

    #[test]
    fn poop() {
        let mut p = Processor::new();
        let bl: usize = 512;

        let mut a = vec![0.0_f32; bl];
        let b = vec![0.0_f32; bl];
        let morph_k = vec![0.0_f32; bl];
        let fade_k = vec![0.0_f32; bl];

        p.process_2x(&mut a[..], &b[..], &morph_k[..], &fade_k[..], &[0.95]);
        p.process(&mut a[..], &b[..], &morph_k[..], &[0.95]);
    }
}
