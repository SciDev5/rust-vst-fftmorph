use std::{num::NonZeroU32, sync::Arc};

use nih_plug::{nih_export_vst3, prelude::*};
use processor::Processor;

mod dbug;
mod morpher;
mod processor;
mod util;

struct MorphPlugin {
    params: Arc<MorphParams>,
    sample_rate: f32,

    processors: [Processor; 2],
}
impl MorphPlugin {}
impl Default for MorphPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(MorphParams::default()),
            sample_rate: 1.0,
            processors: [Processor::new(), Processor::new()],
        }
    }
}

#[derive(Params)]
struct MorphParams {
    #[id = "morph"]
    pub k_morph: FloatParam,
    #[id = "fade"]
    pub k_fade: FloatParam,
    #[id = "z"]
    pub z: FloatParam,
    #[id = "iter_count"]
    pub iter_count: IntParam,
    // #[id = "2x-mode"]
    // pub double_mode: BoolParam,
    #[id = "gain"]
    pub gain: FloatParam,
}
impl Default for MorphParams {
    fn default() -> Self {
        Self {
            k_morph: FloatParam::new(
                "Morph",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 1.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(3.0))
            .with_step_size(0.01),
            k_fade: FloatParam::new(
                "X-Fade",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 1.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(3.0))
            .with_step_size(0.01),
            z: FloatParam::new(
                "FrequencySpread",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 1.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(3.0))
            .with_step_size(0.01),
            iter_count: IntParam::new(
                "Iter. Count",
                5,
                IntRange::Linear {
                    min: 0,
                    max: 15,
                },
            )
            .with_smoother(SmoothingStyle::None),
            gain: FloatParam::new(
                "Gain",
                -10.0,
                FloatRange::Linear {
                    min: -50.0,
                    max: 0.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(3.0))
            .with_step_size(0.01)
            .with_unit(" dB"),
        }
    }
}

const N_CHANNELS: usize = 2;

impl Plugin for MorphPlugin {
    const NAME: &'static str = "SR-FFTMorph";

    const VENDOR: &'static str = "SciDev5";

    const URL: &'static str = "no";

    const EMAIL: &'static str = "no";

    const VERSION: &'static str = "0.0.0";

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(N_CHANNELS as u32),
        main_output_channels: NonZeroU32::new(N_CHANNELS as u32),
        aux_input_ports: &[new_nonzero_u32(N_CHANNELS as u32)],
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;

        true
    }

    fn reset(&mut self) {}

    fn params(&self) -> std::sync::Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let block_len = buffer.samples();
        let samples_main = buffer.as_slice();
        let samples_aux = aux.inputs[0].as_slice();
        
        let mut morph_k = vec![0.0; block_len];
        let mut fade_k = vec![0.0; block_len];
        let mut aux_spectral_spread = vec![0.0; block_len];
        let mut iter_count = vec![0; block_len];
        self.params.k_morph.smoothed.next_block(&mut morph_k[..], block_len);
        self.params.k_fade.smoothed.next_block(&mut fade_k[..], block_len);
        self.params.z.smoothed.next_block(&mut aux_spectral_spread[..], block_len);
        self.params.iter_count.smoothed.next_block(&mut iter_count[..], block_len);

        for channel_id in 0..N_CHANNELS {
            self.processors[channel_id].process(
                samples_main[channel_id],
                samples_aux[channel_id],
                &morph_k,
                &fade_k,
                aux_spectral_spread[0],
                iter_count[0],
            );
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for MorphPlugin {
    const CLAP_ID: &'static str = "me.scidev5";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("funky lmao");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}
impl Vst3Plugin for MorphPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"SR-FFTMorph_____";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Modulation];
}

nih_export_clap!(MorphPlugin);
nih_export_vst3!(MorphPlugin);
