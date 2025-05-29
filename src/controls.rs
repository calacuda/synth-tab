use iced_core::Element;
use iced_wgpu::Renderer;
use iced_widget::{
    button, column, container, horizontal_space, pick_list, row, slider, text, text_editor,
    text_input, vertical_space, PickList, Row, Slider, Space,
};
use iced_winit::core::{Alignment, Color, Length, Theme};
use iced_winit::runtime::{Program, Task};
use iced_winit::winit::event_loop::EventLoopProxy;
use log::*;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use stepper_synth_backend::pygame_coms::WTSynthParam;
use stepper_synth_backend::synth_engines::wave_table::WaveTableEngine;
use stepper_synth_backend::synth_engines::SynthModule;

use crate::synth::TabSynth;
use crate::UserEvent;

// const EXAMPLES: [Example; 3] = [Example::Integration, Example::Counter, Example::TextEditor];

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum Example {
//     Integration,
//     Counter,
//     TextEditor,
// }

// impl std::fmt::Display for Example {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SynthScreen {
    Osc,
    Env,
    LFO,
    LowPass,
    ModMatrix,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Screen {
    Settings,
    MidiSelection,
    SynthScreen(SynthScreen),
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenSettingsMenu,
    OpenMidiMenu,
    SwitchSynthScreen(SynthScreen),
    SetSynthParam { param: WTSynthParam },
}

#[derive(Debug)]
pub struct Controls {
    background_color: Color,
    input: String,
    value: i32,
    screen: Screen,
    editor: text_editor::Content<Renderer>,
    proxy: EventLoopProxy<UserEvent>,
    synth: Arc<RwLock<TabSynth>>,
}

// #[derive(Debug, Clone)]
// pub enum Message {
//     RedChanged(f32),
//     GreenChanged(f32),
//     BlueChanged(f32),
//     InputChanged(String),
//     EditorAction(text_editor::Action),
//     ExampleSelected(Example),
//     Inc,
//     Dec,
// }

impl Controls {
    pub fn new(proxy: EventLoopProxy<UserEvent>, synth: Arc<RwLock<TabSynth>>) -> Controls {
        Controls {
            background_color: Color::BLACK,
            input: String::default(),
            value: 0,
            screen: Screen::SynthScreen(SynthScreen::Osc),
            editor: text_editor::Content::new(),
            proxy,
            synth,
        }
    }

    pub fn background_color(&self) -> Color {
        self.background_color
    }
}

impl Program for Controls {
    type Theme = Theme;
    type Message = Message;
    type Renderer = Renderer;

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Message::Inc => self.value += 1,
            // Message::Dec => self.value -= 1,
            // Message::ExampleSelected(example) => self.selected_example = example,
            // Message::InputChanged(value) => self.input = value,
            // Message::RedChanged(r) => self.background_color.r = r,
            // Message::GreenChanged(g) => self.background_color.g = g,
            // Message::BlueChanged(b) => self.background_color.b = b,
            // Message::EditorAction(action) => match action {
            //     text_editor::Action::Focus => {
            //         log::info!("Editor focused");
            //         // it's possible to call java::call_instance_method("showKeyboard")
            //         // right here, but needed something to show the usage of user events
            //         let _ = self.proxy.send_event(UserEvent::ShowKeyboard);
            //     }
            //     text_editor::Action::Blur => {
            //         log::info!("Editor lost focus");
            //         let _ = self.proxy.send_event(UserEvent::HideKeyboard);
            //     }
            //     other => self.editor.perform(other),
            // },
            Message::OpenSettingsMenu => warn!("Settings menu not written yet"),
            Message::OpenMidiMenu => warn!("MIDI menu not written yet"),
            Message::SetSynthParam { param } => match param {
                _ => warn!("Settings wavetable synth params not written yet"),
            },
            Message::SwitchSynthScreen(screen) => self.screen = Screen::SynthScreen(screen),
        }

        Task::none()
    }

    fn view(&self) -> Element<Message, Theme, Renderer> {
        let top_bar = row![
            // settings button
            container(
                button("Settings").on_press(Message::OpenSettingsMenu) // .alig(Alignment::Left)
                                                                       // .into()
            )
            .align_x(Alignment::Start),
            container(row![
                // Oscilator button
                button("Oscilator").on_press(Message::SwitchSynthScreen(SynthScreen::Osc)),
                // .alig(Alignment::Left)knob_widge
                // .into(),
                // .center
                // adsr button
                button("ADSR").on_press(Message::SwitchSynthScreen(SynthScreen::Env)),
                // LFO button
                button("LFO").on_press(Message::SwitchSynthScreen(SynthScreen::LFO)),
                // Lowpass Filter button
                button("Low-Pass").on_press(Message::SwitchSynthScreen(SynthScreen::LowPass)),
                // Mod Matrix button
                button("Mod-Matrix").on_press(Message::SwitchSynthScreen(SynthScreen::ModMatrix)),
            ])
            .width(Length::Fill)
            .align_x(Alignment::Center),
            // Midi Settings menu
            container(
                button("MIDI").on_press(Message::OpenSettingsMenu) // .alig(Alignment::Left)
                                                                   // .into()
            )
            .align_x(Alignment::End),
        ]
        // .spacing(Length::Fill)
        .width(Length::Fill);

        // match self.selected_example {
        //     Example::Integration => self.integration(),
        //     Example::Counter => self.counter(),
        //     Example::TextEditor => self.text_editor(),
        // }

        let synth_screen = if let SynthModule::WaveTable(ref engine) =
            self.synth.read().unwrap().synth.read().unwrap().engine
        {
            match self.screen {
                Screen::Settings => row![text("Settings")
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center()]
                .into(),
                Screen::MidiSelection => row![text("MIDI-Selector")
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center()]
                .into(),
                Screen::SynthScreen(SynthScreen::Osc) => self.osc(engine.deref()),
                Screen::SynthScreen(SynthScreen::Env) => row![text("Env")
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center()]
                .into(),
                Screen::SynthScreen(SynthScreen::LFO) => row![text("LFO")
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center()]
                .into(),
                Screen::SynthScreen(SynthScreen::LowPass) => row![text("LowPass")
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center()]
                .into(),
                Screen::SynthScreen(SynthScreen::ModMatrix) => row![text("Mod")
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center()]
                .into(),
            }
        } else {
            row![text("Error")
                .width(Length::Fill)
                .height(Length::Fill)
                .center()]
            .into()
        };

        column![top_bar, synth_screen]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()

        // top_bar.into()
        // self.integration()
    }
}

fn color_slider<'a>(value: f32, f: impl Fn(f32) -> Message + 'a) -> Slider<'a, f32, Message> {
    slider(0.0..=1.0, value, f).step(0.01)
}

impl Controls {
    fn osc(&self, engine: &WaveTableEngine) -> Element<Message, Theme, Renderer> {
        // fn osc(&self, engine: &WaveTableEngine) -> Element<Message> {
        let osc_display = move |i: usize| -> Row<'_, Message> {
            // text(format!("Osc {i}"))
            //     .width(Length::Fill)
            //     .height(Length::Fill)
            //     .center()
            row![
                // text(format!("{}", i + 1))
                // .width(Length::Fill)
                // .height(Length::Fill)
                // .center(),

            ]
        };

        column![
            // text("Osc")
            // .width(Length::Fill)
            // .height(Length::Fill)
            // .center()
            osc_display(0),
            osc_display(1),
            osc_display(2),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

impl Controls {
    // fn examples(&self) -> PickList<Example, &[Example], Example, Message> {
    //     pick_list(
    //         &EXAMPLES[..],
    //         Some(self.selected_example),
    //         Message::ExampleSelected,
    //     )
    // }

    // fn integration(&self) -> Element<Message, Theme, Renderer> {
    //     // let sliders = row![
    //     //     color_slider(self.background_color.r, Message::RedChanged),
    //     //     color_slider(self.background_color.g, Message::GreenChanged),
    //     //     color_slider(self.background_color.b, Message::BlueChanged),
    //     // ]
    //     // .width(Length::Fill)
    //     // .spacing(20);
    //
    //     container(
    //         column![
    //             Space::with_height(20),
    //             // self.examples(),
    //             vertical_space(),
    //             row![
    //                 text!("{:?}", self.backgrouiced_gitnd_color).size(14),
    //                 horizontal_space(),
    //             ],
    //             // text_input("Placeholder", &self.input).on_input(Message::InputChanged),
    //             // sliders,
    //             Space::with_height(20),
    //         ]
    //         .align_x(Alignment::Center)
    //         .spacing(10),
    //     )
    //     .padding(10)
    //     .into()
    // }

    // fn counter(&self) -> Element<Message, Theme, Renderer> {
    //     container(
    //         column![
    //             Space::with_height(30),
    //             self.examples(),
    //             vertical_space(),
    //             button("Increment").on_press(Message::Inc),
    //             text!("{}", self.value).size(50),
    //             button("Decrement").on_press(Message::Dec),
    //             vertical_space(),
    //             Space::with_height(100),
    //         ]
    //         .align_x(Alignment::Center)
    //         .spacing(10),
    //     )
    //     .center(Length::Fill)
    //     .style(add_background)
    //     .into()
    // }
    //
    // fn text_editor(&self) -> Element<Message, Theme, Renderer> {
    //     container(
    //         column![
    //             Space::with_height(30),
    //             self.examples(),
    //             vertical_space(),
    //             text_editor::<Message, Theme, Renderer>(&self.editor)
    //                 .height(400)
    //                 .on_action(Message::EditorAction),
    //             vertical_space(),
    //         ]
    //         .align_x(Alignment::Center),
    //     )
    //     .padding(10)
    //     .center(Length::Fill)
    //     .style(add_background)
    //     .into()
    // }
}

fn add_background(theme: &Theme) -> container::Style {
    theme.palette().background.into()
}
