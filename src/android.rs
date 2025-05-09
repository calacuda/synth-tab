extern crate jni;

use super::*;
use jni::objects::{JByteArray, JClass, JList, JString, ReleaseMode};
use jni::JNIEnv;
use midi_control::MidiMessage;
use stepper_synth_backend::MidiControlled;

#[no_mangle]
pub unsafe extern "C" fn Java_co_realfit_example_AppMidiManager_newMidiDev(
    mut env: JNIEnv,
    _: JClass,
    newMidiDev: JString,
) {
    // // Our Java companion code might pass-in "world" as a string, hence the name.
    // let world = rust_greeting(env.get_string(java_pattern).expect("invalid pattern string").as_ptr());
    // // Retake pointer so that we can use it below and allow memory to be freed when it goes out of scope.
    // let world_ptr = CString::from_raw(world);
    // let output = env.new_string(world_ptr.to_str().unwrap()).expect("Couldn't create java string!");

    // output.into_inner()

    // let world = env.g(java_pattern).expect("invalid pattern string").as_ptr();
    // let Ok(mut iterator) = newMidiDev.iter(&mut env) else {
    //     log::error!("new midi dev failed");
    //     return;
    // };
    //
    // let mut devices = Vec::new();
    //
    // while let Ok(Some(j_obj)) = iterator.next(&mut env) {
    //     devices.push(
    //         env.get_string((&j_obj).into())
    //             .expect("invalid pattern string")
    //             .to_string_lossy()
    //             .to_string(),
    //     );
    // }
    //
    // println!("midi ports = {devices:?}")

    let dev = env.get_string(&newMidiDev).expect("invalid pattern string");

    if let Ok(mut devs) = MIDI_DEVS.write() {
        devs.push(dev.into());
    }

    if let Ok(mut devs) = MIDI_DEVS.read() {
        info!("devices: {devs:?}")
    }
}

#[no_mangle]
pub unsafe extern "C" fn Java_co_realfit_example_AppMidiManager_sendMidiMessage(
    mut env: JNIEnv,
    _: JClass,
    message: JByteArray,
) {
    // // Our Java companion code might pass-in "world" as a string, hence the name.
    // let world = rust_greeting(env.get_string(java_pattern).expect("invalid pattern string").as_ptr());
    // // Retake pointer so that we can use it below and allow memory to be freed when it goes out of scope.
    // let world_ptr = CString::from_raw(world);
    // let output = env.new_string(world_ptr.to_str().unwrap()).expect("Couldn't create java string!");

    // output.into_inner()

    // let world = env.g(java_pattern).expect("invalid pattern string").as_ptr();
    println!("midi message = {message:?}");
    log::info!("midi message = {message:?}");

    let bytes: Vec<u8> = match env.get_array_elements(&message, ReleaseMode::NoCopyBack) {
        Ok(elms) => elms
            .iter()
            .map(|b| {
                // if *b == -128 {
                //
                // } else if *b < 0 {
                //     error!("{b} is less then 0!");
                //     b.abs() as u8
                // } else {
                //     *b as u8
                // }
                b.to_ne_bytes()[0]
            })
            .collect(),
        Err(e) => {
            log::error!("{e}");
            return;
        }
    };

    log::info!("bytes = {bytes:?}");

    let message = MidiMessage::from(bytes.as_ref());

    log::info!("midi message (as enum) = {message:?}");

    // TAB_SYNTH.lock().map(|synth| {
    //     synth
    //         .synth
    //         .lock()
    //         .map(|mut synth| synth.midi_input(&message))
    // });

    // if let Ok(ref mut synth) = TAB_SYNTH.lock() {
    //     log::info!("SYNTH!");
    //
    //     if let Some(ref mut synth) = **synth {
    //         log::info!("synth was set!");
    //
    //         if let Ok(ref mut synth) = synth.synth.lock() {
    //             log::info!("sending message");
    //
    //             synth.midi_input(&message);
    //         }
    //     }
    // }
    // CBEAM_CHANNELS.0.send(message);
    MIDI_SEND.send(message);
}

#[no_mangle]
pub unsafe extern "C" fn Java_co_realfit_example_AppMidiManager_clearKnownDevs(
    mut env: JNIEnv,
    _: JClass,
) {
    if let Ok(mut devs) = MIDI_DEVS.write() {
        devs.clear();
    }
}
