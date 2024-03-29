use std::sync::Arc;

use miette::Result;

use crate::{
    dsp::{
        basic::{UiInputWidget, UiOutputWidget},
        generators::BL_SQUARE_MAX_COEFF,
    },
    graph::Node,
};

use super::{ParsedCreationArg, ParsedIdent};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinNode {
    Sine,
    Cosine,
    Tanh,
    Exp,
    Abs,
    FmSineOsc,
    SineOsc,
    BlSawOsc,
    BlSquareOsc,
    MidiToFreq,
    Clock,
    Delay,
    NoteIn,
    RisingEdge,
    FallingEdge,
    Var,
    Max,
    Min,
    Clip,
    Debug,
    If,
    Not,
    Sample,
    Noise,
    Ladder,

    // UI nodes
    Slider,
    Button,
    Toggle,
    Led,

    // not actually creatable nodes
    Constant,
    Dac0,
    Add,
    Sub,
    Mul,
    Div,
    VariableInput,
}

impl BuiltinNode {
    pub fn try_from_ident(id: &ParsedIdent) -> Option<BuiltinNode> {
        match id.0.as_str() {
            "Sin" => Some(BuiltinNode::Sine),
            "Cos" => Some(BuiltinNode::Cosine),
            "Exp" => Some(BuiltinNode::Exp),
            "Tanh" => Some(BuiltinNode::Tanh),
            "Abs" => Some(BuiltinNode::Abs),
            "SineFm" => Some(BuiltinNode::FmSineOsc),
            "SineOsc" => Some(BuiltinNode::SineOsc),
            "SawOsc" => Some(BuiltinNode::BlSawOsc),
            "SquareOsc" => Some(BuiltinNode::BlSquareOsc),
            "M2F" => Some(BuiltinNode::MidiToFreq),
            "Clock" => Some(BuiltinNode::Clock),
            "Delay" => Some(BuiltinNode::Delay),
            "NoteIn" => Some(BuiltinNode::NoteIn),
            "Redge" => Some(BuiltinNode::RisingEdge),
            "Fedge" => Some(BuiltinNode::FallingEdge),
            "Var" => Some(BuiltinNode::Var),
            "Max" => Some(BuiltinNode::Max),
            "Min" => Some(BuiltinNode::Min),
            "Clip" => Some(BuiltinNode::Clip),
            "Debug" => Some(BuiltinNode::Debug),
            "If" => Some(BuiltinNode::If),
            "Not" => Some(BuiltinNode::Not),
            "Sample" => Some(BuiltinNode::Sample),
            "Noise" => Some(BuiltinNode::Noise),
            "Ladder" => Some(BuiltinNode::Ladder),

            // UI nodes
            "Slider" => Some(BuiltinNode::Slider),
            "Button" => Some(BuiltinNode::Button),
            "Toggle" => Some(BuiltinNode::Toggle),
            "Led" => Some(BuiltinNode::Led),
            _ => None,
        }
    }

    pub fn default_creation_args(&self) -> &[ParsedCreationArg] {
        match self {
            Self::Sample => unimplemented!(),
            Self::Slider => &[
                ParsedCreationArg::Scalar(0.0),
                ParsedCreationArg::Scalar(1.0),
            ],
            Self::Button => &[ParsedCreationArg::Scalar(0.0)],
            Self::Toggle => &[ParsedCreationArg::Scalar(0.0)],
            Self::Constant => unimplemented!(),
            _ => &[],
        }
    }

    pub fn create_node(
        &self,
        name: &str,
        creation_args: &[ParsedCreationArg],
    ) -> Result<Arc<Node>> {
        let node = match self {
            Self::Debug => crate::dsp::basic::DebugNode::create_node(name),
            Self::Sine => crate::dsp::basic::Sine::create_node(name),
            Self::Cosine => crate::dsp::basic::Cosine::create_node(name),
            Self::Exp => crate::dsp::basic::Exp::create_node(name),
            Self::Tanh => crate::dsp::basic::Tanh::create_node(name),
            Self::Abs => crate::dsp::basic::Abs::create_node(name),
            Self::Not => crate::dsp::basic::Not::create_node(name),
            Self::FmSineOsc => crate::dsp::generators::FmSineOsc::create_node(name),
            Self::SineOsc => crate::dsp::generators::SineOsc::create_node(name),
            Self::BlSawOsc => crate::dsp::generators::BlSawOsc::create_node(name, 0.0, 1.0, 0.0),
            Self::BlSquareOsc => {
                crate::dsp::generators::BlSquareOsc::create_node(name, [0.0; BL_SQUARE_MAX_COEFF])
            }
            Self::MidiToFreq => crate::dsp::midi::MidiToFreq::create_node(name),
            Self::Clock => crate::dsp::time::Clock::create_node(name),
            Self::Delay => crate::dsp::time::Delay::create_node(
                name,
                vec![0.0; 480000], // TODO: don't hardcode
                0.0,
                0.0,
                0.0,
            ),
            Self::NoteIn => crate::io::midi::NoteIn::create_node(name),
            Self::RisingEdge => crate::dsp::basic::RisingEdge::create_node(name, 0.0),
            Self::FallingEdge => crate::dsp::basic::FallingEdge::create_node(name, 0.0),
            Self::Var => crate::dsp::graph_util::Var::create_node(name, 0.0),
            Self::Max => crate::dsp::basic::Max::create_node(name),
            Self::Min => crate::dsp::basic::Min::create_node(name),
            Self::Clip => crate::dsp::basic::Clip::create_node(name),
            Self::If => crate::dsp::basic::If::create_node(name),
            Self::Sample => crate::dsp::samplers::Sample::create_node(
                name,
                creation_args[0].unwrap_string().into(),
            )?,
            Self::Noise => crate::dsp::generators::WhiteNoiseOsc::create_node(name),
            Self::Ladder => crate::dsp::filters::MoogLadder::create_node(
                name, [0.0; 4], [0.0; 3], [0.0; 6], 0.0, 0.0, 0.0,
            ),
            // Self::Ladder => {
            //     crate::dsp::filters::DummyFilter::create_node(name, vec![0.0; 128].into())
            // }

            // UI nodes
            Self::Slider => crate::dsp::basic::UiInput::create_node(
                name,
                *creation_args[2].unwrap_scalar(),
                UiInputWidget::Slider {
                    minimum: *creation_args[0].unwrap_scalar(),
                    maximum: *creation_args[1].unwrap_scalar(),
                },
            ),
            Self::Button => crate::dsp::basic::UiInput::create_node(
                name,
                *creation_args[0].unwrap_scalar(),
                UiInputWidget::Button,
            ),
            Self::Toggle => crate::dsp::basic::UiInput::create_node(
                name,
                *creation_args[0].unwrap_scalar(),
                UiInputWidget::Toggle,
            ),
            Self::Led => {
                crate::dsp::basic::UiOutput::create_node(name, UiOutputWidget::Led { value: 0.0 })
            }

            // not actually creatable nodes
            Self::Constant => {
                crate::dsp::basic::Constant::create_node(name, *creation_args[0].unwrap_scalar())
            }
            Self::Dac0 => unimplemented!(),
            Self::Add => crate::dsp::basic::Add::create_node(name),
            Self::Sub => crate::dsp::basic::Sub::create_node(name),
            Self::Mul => crate::dsp::basic::Mul::create_node(name),
            Self::Div => crate::dsp::basic::Div::create_node(name),
            Self::VariableInput => unimplemented!(),
        };
        Ok(node)
    }
}
