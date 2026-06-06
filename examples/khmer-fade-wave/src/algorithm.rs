use crate::KhmerFadeWave;

#[derive(Clone, Debug, PartialEq)]
pub enum KhmerFadeWaveEvent {
    RevealUnits,
    WaveUnits,
}

#[derive(Clone, Debug, PartialEq)]
pub struct KhmerFadeWaveTrace {
    pub events: Vec<KhmerFadeWaveEvent>,
}

pub fn khmer_fade_wave_algorithm(_state: KhmerFadeWave) -> KhmerFadeWaveTrace {
    KhmerFadeWaveTrace {
        events: vec![
            KhmerFadeWaveEvent::RevealUnits,
            KhmerFadeWaveEvent::WaveUnits,
        ],
    }
}
