use candle_core::{Device, IndexOp, Tensor};
use candle_nn::{VarBuilder, ops::softmax};
use candle_transformers::models::whisper::{self as m, Config, audio};
use rand::SeedableRng;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::{CODEC_TYPE_NULL, DecoderOptions};
use symphonia::core::conv::FromSample;
use tokenizers::Tokenizer;

fn conv<T>(samples: &mut Vec<f32>, data: std::borrow::Cow<symphonia::core::audio::AudioBuffer<T>>)
where
    T: symphonia::core::sample::Sample,
    f32: symphonia::core::conv::FromSample<T>,
{
    samples.extend(data.chan(0).iter().map(|v| f32::from_sample(*v)))
}

fn pcm_decode<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<(Vec<f32>, u32)> {
    // Open the media source.
    let src = std::fs::File::open(path)?;

    // Create the media source stream.
    let mss = symphonia::core::io::MediaSourceStream::new(Box::new(src), Default::default());

    // Create a probe hint using the file's extension. [Optional]
    let hint = symphonia::core::probe::Hint::new();

    // Use the default options for metadata and format readers.
    let meta_opts: symphonia::core::meta::MetadataOptions = Default::default();
    let fmt_opts: symphonia::core::formats::FormatOptions = Default::default();

    // Probe the media source.
    let probed = symphonia::default::get_probe().format(&hint, mss, &fmt_opts, &meta_opts)?;
    // Get the instantiated format reader.
    let mut format = probed.format;

    // Find the first audio track with a known (decodeable) codec.
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .expect("no supported audio tracks");

    // Use the default options for the decoder.
    let dec_opts: DecoderOptions = Default::default();

    // Create a decoder for the track.
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &dec_opts)
        .expect("unsupported codec");
    let track_id = track.id;
    let sample_rate = track.codec_params.sample_rate.unwrap_or(0);
    let mut pcm_data = Vec::new();
    // The decode loop.
    while let Ok(packet) = format.next_packet() {
        // Consume any new metadata that has been read since the last packet.
        while !format.metadata().is_latest() {
            format.metadata().pop();
        }

        // If the packet does not belong to the selected track, skip over it.
        if packet.track_id() != track_id {
            continue;
        }
        match decoder.decode(&packet)? {
            AudioBufferRef::F32(buf) => pcm_data.extend(buf.chan(0)),
            AudioBufferRef::U8(data) => conv(&mut pcm_data, data),
            AudioBufferRef::U16(data) => conv(&mut pcm_data, data),
            AudioBufferRef::U24(data) => conv(&mut pcm_data, data),
            AudioBufferRef::U32(data) => conv(&mut pcm_data, data),
            AudioBufferRef::S8(data) => conv(&mut pcm_data, data),
            AudioBufferRef::S16(data) => conv(&mut pcm_data, data),
            AudioBufferRef::S24(data) => conv(&mut pcm_data, data),
            AudioBufferRef::S32(data) => conv(&mut pcm_data, data),
            AudioBufferRef::F64(data) => conv(&mut pcm_data, data),
        }
    }
    Ok((pcm_data, sample_rate))
}

pub fn token_id(tokenizer: &Tokenizer, token: &str) -> u32 {
    tokenizer.token_to_id(token).unwrap()
}

pub struct DecodingResult {
    pub tokens: Vec<u32>,
    pub text: String,
    pub avg_logprob: f64,
    pub no_speech_prob: f64,
    pub temperature: f64,
    pub compression_ratio: f64,
}

pub struct Segment {
    pub start: f64,
    pub duration: f64,
    pub dr: DecodingResult,
}

pub struct Decoder {
    pub model: m::model::Whisper,
    pub rng: rand::rngs::StdRng,
    pub timestamps: bool,
    pub tokenizer: Tokenizer,
    pub suppress_tokens: Tensor,
    pub sot_token: u32,
    pub transcribe_token: u32,
    pub translate_token: u32,
    pub eot_token: u32,
    pub no_speech_token: u32,
    pub no_timestamps_token: u32,
    pub language_token: Option<u32>,
}

impl Decoder {
    pub fn new(
        model: m::model::Whisper,
        tokenizer: Tokenizer,
        seed: u64,
        device: &Device,
        language_token: Option<u32>,
        timestamps: bool,
    ) -> Self {
        let no_timestamps_token = token_id(&tokenizer, m::NO_TIMESTAMPS_TOKEN);
        // Suppress the notimestamps token when in timestamps mode.
        // https://github.com/openai/whisper/blob/e8622f9afc4eba139bf796c210f5c01081000472/whisper/decoding.py#L452
        let suppress_tokens: Vec<f32> = (0..model.config.vocab_size as u32)
            .map(|i| {
                if model.config.suppress_tokens.contains(&i)
                    || timestamps && i == no_timestamps_token
                {
                    f32::NEG_INFINITY
                } else {
                    0f32
                }
            })
            .collect();
        let suppress_tokens = Tensor::new(suppress_tokens.as_slice(), device).unwrap();
        let sot_token = token_id(&tokenizer, m::SOT_TOKEN);
        let transcribe_token = token_id(&tokenizer, m::TRANSCRIBE_TOKEN);
        let translate_token = token_id(&tokenizer, m::TRANSLATE_TOKEN);
        let eot_token = token_id(&tokenizer, m::EOT_TOKEN);
        let no_speech_token = m::NO_SPEECH_TOKENS
            .iter()
            .find_map(|token| tokenizer.token_to_id(token))
            .unwrap();

        Self {
            model,
            rng: rand::rngs::StdRng::seed_from_u64(seed),
            tokenizer,
            timestamps,
            suppress_tokens,
            sot_token,
            transcribe_token,
            translate_token,
            eot_token,
            no_speech_token,
            language_token,
            no_timestamps_token,
        }
    }

    pub fn decode(&mut self, mel: &Tensor, t: f64) -> DecodingResult {
        let model = &mut self.model;
        let audio_features = model.encoder.forward(mel, true).unwrap();
        let sample_len = model.config.max_target_positions / 2;
        let mut sum_logprob = 0f64;
        let mut no_speech_prob = f64::NAN;
        let mut tokens = vec![self.sot_token];
        if let Some(language_token) = self.language_token {
            tokens.push(language_token);
        }

        tokens.push(self.transcribe_token);

        if !self.timestamps {
            tokens.push(self.no_timestamps_token);
        }
        for i in 0..sample_len {
            let tokens_t = Tensor::new(tokens.as_slice(), mel.device()).unwrap();

            // The model expects a batch dim but this inference loop does not handle
            // it so we add it at this point.
            let tokens_t = tokens_t.unsqueeze(0).unwrap();
            let ys = model
                .decoder
                .forward(&tokens_t, &audio_features, i == 0)
                .unwrap();

            // Extract the no speech probability on the first iteration by looking at the first
            // token logits and the probability for the according token.
            if i == 0 {
                let logits = model
                    .decoder
                    .final_linear(&ys.i(..1).unwrap())
                    .unwrap()
                    .i(0)
                    .unwrap()
                    .i(0)
                    .unwrap();
                no_speech_prob = softmax(&logits, 0)
                    .unwrap()
                    .i(self.no_speech_token as usize)
                    .unwrap()
                    .to_scalar::<f32>()
                    .unwrap() as f64;
            }

            let (_, seq_len, _) = ys.dims3().unwrap();
            let logits = model
                .decoder
                .final_linear(&ys.i((..1, seq_len - 1..)).unwrap())
                .unwrap()
                .i(0)
                .unwrap()
                .i(0)
                .unwrap();
            // TODO: Besides suppress tokens, we should apply the heuristics from
            // ApplyTimestampRules, i.e.:
            // - Timestamps come in pairs, except before EOT.
            // - Timestamps should be non-decreasing.
            // - If the sum of the probabilities of timestamps is higher than any other tokens,
            //   only consider timestamps when sampling.
            // https://github.com/openai/whisper/blob/e8622f9afc4eba139bf796c210f5c01081000472/whisper/decoding.py#L439
            let logits = logits.broadcast_add(&self.suppress_tokens).unwrap();
            let next_token = if t > 0f64 {
                let prs = softmax(&(&logits / t).unwrap(), 0).unwrap();
                let logits_v: Vec<f32> = prs.to_vec1().unwrap();
                let distr = WeightedIndex::new(&logits_v).unwrap();
                distr.sample(&mut self.rng) as u32
            } else {
                let logits_v: Vec<f32> = logits.to_vec1().unwrap();
                logits_v
                    .iter()
                    .enumerate()
                    .max_by(|(_, u), (_, v)| u.total_cmp(v))
                    .map(|(i, _)| i as u32)
                    .unwrap()
            };
            tokens.push(next_token);
            let prob = softmax(&logits, candle_core::D::Minus1)
                .unwrap()
                .i(next_token as usize)
                .unwrap()
                .to_scalar::<f32>()
                .unwrap() as f64;
            if next_token == self.eot_token || tokens.len() > model.config.max_target_positions {
                break;
            }
            sum_logprob += prob.ln();
        }
        let text = self.tokenizer.decode(&tokens, true).unwrap();
        let avg_logprob = sum_logprob / tokens.len() as f64;

        DecodingResult {
            tokens,
            text,
            avg_logprob,
            no_speech_prob,
            temperature: t,
            compression_ratio: f64::NAN,
        }
    }

    pub fn decode_with_fallback(&mut self, segment: &Tensor) -> DecodingResult {
        for (i, &t) in m::TEMPERATURES.iter().enumerate() {
            let dr = self.decode(segment, t);
            if i == m::TEMPERATURES.len() - 1 {
                return dr;
            }

            let needs_fallback = dr.compression_ratio > m::COMPRESSION_RATIO_THRESHOLD
                || dr.avg_logprob < m::LOGPROB_THRESHOLD;
            if !needs_fallback || dr.no_speech_prob > m::NO_SPEECH_THRESHOLD {
                return dr;
            }
        }
        unreachable!()
    }

    pub fn run(&mut self, mel: &Tensor) -> Vec<Segment> {
        let (_, _, content_frames) = mel.dims3().unwrap();
        let mut seek = 0;
        let mut segments = vec![];
        while seek < content_frames {
            let time_offset = (seek * m::HOP_LENGTH) as f64 / m::SAMPLE_RATE as f64;
            let segment_size = usize::min(content_frames - seek, m::N_FRAMES);
            let mel_segment = mel.narrow(2, seek, segment_size).unwrap();
            let segment_duration = (segment_size * m::HOP_LENGTH) as f64 / m::SAMPLE_RATE as f64;
            let dr = self.decode_with_fallback(&mel_segment);
            seek += segment_size;
            if dr.no_speech_prob > m::NO_SPEECH_THRESHOLD && dr.avg_logprob < m::LOGPROB_THRESHOLD {
                println!("no speech detected, skipping {seek}");
                continue;
            }
            let segment = Segment {
                start: time_offset,
                duration: segment_duration,
                dr,
            };

            segments.push(segment)
        }
        segments
    }
}

pub fn load_model(
    config_file: &str,
    tokenizer_file: &str,
    model_file: &str,
    device: &Device,
) -> (Decoder, Config, Vec<f32>) {
    let config: Config =
        serde_json::from_str(&std::fs::read_to_string(config_file).unwrap()).unwrap();
    let tokenizer = Tokenizer::from_file(tokenizer_file).unwrap();
    let vb =
        unsafe { VarBuilder::from_mmaped_safetensors(&[model_file], m::DTYPE, device).unwrap() };
    let model = m::model::Whisper::load(&vb, config.clone()).unwrap();
    let decoder = Decoder::new(model, tokenizer, 299792458, device, None, false);

    let mel_bytes = match config.num_mel_bins {
        80 => include_bytes!("../../melfilters/melfilters.bytes").as_slice(),
        128 => include_bytes!("../../melfilters/melfilters128.bytes").as_slice(),
        _ => panic!("unexpected num_mel_bins"),
    };
    let mut mel_filters = vec![0f32; mel_bytes.len() / 4];
    <byteorder::LittleEndian as byteorder::ByteOrder>::read_f32_into(mel_bytes, &mut mel_filters);

    (decoder, config, mel_filters)
}

pub fn preprocess_input(
    recording_path: &str,
    config: &Config,
    device: &Device,
    mel_filters: &Vec<f32>,
) -> Tensor {
    let input = std::path::PathBuf::from(recording_path);

    let (pcm_data, sample_rate) = pcm_decode(input).unwrap();
    assert_eq!(sample_rate, m::SAMPLE_RATE as u32);
    let mel = audio::pcm_to_mel(config, &pcm_data, mel_filters);
    let mel_len = mel.len();

    Tensor::from_vec(
        mel,
        (1, config.num_mel_bins, mel_len / config.num_mel_bins),
        device,
    )
    .unwrap()
}
