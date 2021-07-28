use crate::state::{ItemType, MenuItem, UICategory};
use hexodsp::{NodeInfo, NodeId};

#[derive(Debug, Clone)]
pub enum MenuState {
    None,
    SelectCategory,
    SelectNodeIdFromCat { category: UICategory },
}

impl MenuState {
    pub fn to_items(&self) -> Vec<MenuItem> {
        match self {
            MenuState::None => vec![],
            MenuState::SelectCategory => {
                vec![
                    MenuItem {
                        typ:    ItemType::Back,
                        label:  "<Exit".to_string(),
                        help:   "\nExit Menu".to_string(),
                    },
                    MenuItem {
                        typ:    ItemType::Category(UICategory::Osc),
                        label:  "Osc".to_string(),
                        help:   "Osc\nAudio oscillators.".to_string(),
                    },
                    MenuItem {
                        typ: ItemType::Category(UICategory::Signal),
                        label: "Signal".to_string(),
                        help:  "Signal\nSignal shapers:\n- Filters\n- Waveshapers\n- Delays".to_string(),
                    },
                    MenuItem {
                        typ: ItemType::Category(UICategory::CV),
                        label: "CV".to_string(),
                        help: "CV\nControl voltage shapers:\n- CV converters\n- Quantizers\n- Sample & Hold\n- Slew".to_string(),
                    },
                    MenuItem {
                        typ: ItemType::Category(UICategory::Mod),
                        label: "Mod".to_string(),
                        help: "Mod\nModulation sources:\n- Envelopes\n- LFOs\n- Sequencers\n- Utils".to_string(),
                    },
                    MenuItem {
                        typ: ItemType::Category(UICategory::NtoM),
                        label: "N->M".to_string(),
                        help: "N->M\nSignal merge and splitters:\n- Mixers\n- Logic\n- Math\n- Multiplexers".to_string(),
                    },
                    MenuItem {
                        typ: ItemType::Category(UICategory::IOUtil),
                        label: "I/O".to_string(),
                        help: "I/O\nExternal connections:\n- Audio I/O\n- MIDI".to_string(),
                    },
                ]
            },
            MenuState::SelectNodeIdFromCat { category } => {
                let mut items = vec![];
                items.push(MenuItem {
                    typ:    ItemType::Back,
                    label:  "<Exit".to_string(),
                    help:   "\nExit Menu".to_string(),
                });

                category.get_node_ids(0, |node_id| {
                    items.push(
                        MenuItem {
                            typ:    ItemType::NodeId(node_id),
                            label:  node_id.label().to_string(),
                            help:   NodeInfo::from_node_id(node_id).desc().to_string(),
                        },
                    );
                });
                items
            },
        }
    }
}