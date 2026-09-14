use std::{collections::HashMap, fs, path::Path};

use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};
use std::io::Cursor;

#[derive(Clone, Copy, Debug)]
pub enum AudioEvent {
    Fire,
    Hit,
    Destroy,
    Damage,
    Sound(u16),
}

impl AudioEvent {
    pub const fn path(self) -> Option<&'static str> {
        match self {
            Self::Fire => Some("assets/audio/fire.wav"),
            Self::Hit => Some("assets/audio/hit.wav"),
            Self::Destroy => Some("assets/audio/destroy.wav"),
            Self::Damage => Some("assets/audio/damage.wav"),
            Self::Sound(1) => Some("assets/audio/fire.wav"),
            Self::Sound(2) => Some("assets/audio/hit.wav"),
            Self::Sound(3) => Some("assets/audio/destroy.wav"),
            Self::Sound(4) => Some("assets/audio/damage.wav"),
            Self::Sound(_) => None,
        }
    }
}

pub trait PlatformAudio {
    fn play(&mut self, event: AudioEvent);
}

#[derive(Default)]
pub struct SoundCatalog {
    sounds: HashMap<String, Vec<u8>>,
}

#[derive(Default)]
pub struct BgmCache {
    pub stage_id: Option<u8>,
    pub data: Option<Vec<u8>>,
}

impl BgmCache {
    pub fn load_stage(&mut self, stage_id: u8, path: impl AsRef<Path>) -> bool {
        if self.stage_id == Some(stage_id) && self.data.is_some() {
            return true;
        }
        let Ok(data) = fs::read(path) else {
            self.unload();
            return false;
        };
        self.stage_id = Some(stage_id);
        self.data = Some(data);
        true
    }

    pub fn unload(&mut self) {
        self.stage_id = None;
        self.data = None;
    }
}

impl SoundCatalog {
    pub fn load_at_start(paths: &[&str]) -> Self {
        let sounds = paths
            .iter()
            .filter_map(|path| fs::read(path).ok().map(|bytes| ((*path).to_owned(), bytes)))
            .collect();
        Self { sounds }
    }

    pub fn get(&self, path: impl AsRef<Path>) -> Option<&[u8]> {
        self.sounds
            .get(path.as_ref().to_string_lossy().as_ref())
            .map(Vec::as_slice)
    }

    pub fn len(&self) -> usize {
        self.sounds.len()
    }

    pub fn bytes(&self, path: impl AsRef<Path>) -> Option<&[u8]> {
        self.get(path)
    }
}

pub struct RodioAudio {
    _stream: OutputStream,
    sounds: SoundCatalog,
}

impl RodioAudio {
    pub fn new(sounds: SoundCatalog) -> Result<Self, rodio::StreamError> {
        let stream = OutputStreamBuilder::open_default_stream()?;
        Ok(Self {
            _stream: stream,
            sounds,
        })
    }

    pub fn play_sound_path(&self, path: impl AsRef<Path>) -> bool {
        let Some(bytes) = self.sounds.bytes(path) else {
            return false;
        };
        let Ok(source) = Decoder::try_from(Cursor::new(bytes.to_vec())) else {
            return false;
        };
        let sink = Sink::connect_new(self._stream.mixer());
        sink.append(source);
        sink.detach();
        true
    }

    pub fn play_event(&self, event: AudioEvent) -> bool {
        event.path().is_some_and(|path| self.play_sound_path(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_placeholder_sound_effects_at_start() {
        let catalog = SoundCatalog::load_at_start(&[
            "assets/audio/fire.wav",
            "assets/audio/hit.wav",
            "assets/audio/destroy.wav",
            "assets/audio/damage.wav",
        ]);
        assert_eq!(catalog.len(), 4);
        assert!(
            catalog
                .get("assets/audio/fire.wav")
                .unwrap()
                .starts_with(b"RIFF")
        );
    }

    #[test]
    fn reuses_and_unloads_stage_bgm() {
        let mut cache = BgmCache::default();
        assert!(cache.load_stage(1, "assets/audio/stage01_bgm.wav"));
        let size = cache.data.as_ref().unwrap().len();
        assert!(cache.load_stage(1, "assets/audio/stage01_bgm.wav"));
        assert_eq!(cache.data.as_ref().unwrap().len(), size);
        cache.unload();
        assert!(cache.data.is_none());
    }
}
