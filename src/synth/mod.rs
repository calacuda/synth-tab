use core::panic;
use std::sync::{Arc, Mutex};
use stepper_synth_backend::{
    // pygame_coms::SynthEngineType,
    synth_engines::{
        // organ::organ::Organ,
        Synth,
        SynthChannel,
        SynthEngine,
    },
    SampleGen,
    CHANNEL_SIZE,
    SAMPLE_RATE,
};
use tinyaudio::{run_output_device, OutputDevice, OutputDeviceParameters};

pub struct TabSynth {
    pub synth: Arc<Mutex<Synth>>,
    // exit: Arc<AtomicBool>,
    // _audio_handle: JoinHandle<()>,
    // _device: OutputDevice,
}

impl TabSynth {
    pub fn new() -> (Self, OutputDevice) {
        // let synth = Arc::new(Mutex::new(SynthChannel::from(SynthEngineType::SubSynth)));
        // let synth = Arc::new(Mutex::new(SynthChannel::from(SynthEngineType::MidiOut)));
        let synth = Arc::new(Mutex::new(Synth::new()));

        // let _audio_handle = spawn({
        // let seq = seq.clone();
        let device = {
            let synth = synth.clone();

            // move || {
            let params = OutputDeviceParameters {
                channels_count: 1,
                sample_rate: SAMPLE_RATE as usize,
                // channel_sample_count: 2048,
                channel_sample_count: CHANNEL_SIZE,
            };
            // NOTE: must stay in this thread so that it stays in scope
            run_output_device(params, {
                // let seq = seq.clone();

                move |data| {
                    for samples in data.chunks_mut(params.channels_count) {
                        // let value =
                        //     seq.lock().expect("couldn't lock synth").synth.get_sample();
                        let value = synth
                            .lock()
                            .map(|mut synth| synth.get_sample())
                            .unwrap_or(0.0);

                        for sample in samples {
                            *sample = value;
                        }
                    }
                }
            })
        };

        // if let Err(e) = device {
        //     println!("starting audio playback caused error: {e}");
        // }
        // }
        // });
        match device {
            Ok(device) => (Self { synth }, device),
            Err(e) => {
                println!("starting audio playback caused error: {e}");
                panic!("{e}");
            }
        }
    }

    #[unsafe(no_mangle)]
    pub fn play(&mut self, note: u8, velocity: u8) {
        println!("playing note {note}");
        self.synth.lock().unwrap().get_engine().play(note, velocity);
        // self.synth.lock().unwrap().engine.play(note, velocity);
    }

    #[unsafe(no_mangle)]
    pub fn stop(&mut self, note: u8) {
        println!("stopping note {note}");
        self.synth.lock().unwrap().get_engine().stop(note);
        // self.synth.lock().unwrap().engine.stop(note);
    }
}

// #[unsafe(no_mangle)]
pub fn make_synth() -> (TabSynth, OutputDevice) {
    // let synth = Synth::new();
    // let sequencer = Arc::new(Mutex::new(SequencerIntake::new(synth)));

    let (mut synth, dev) = TabSynth::new();

    // synth.play(42, 127);

    (synth, dev)
}
