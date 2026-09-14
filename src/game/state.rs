#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameState {
    Title,
    Demo,
    StageIntro,
    Playing,
    StageClear,
    GameOver,
    NameEntry,
    Ending,
}

impl Default for GameState {
    fn default() -> Self {
        Self::Title
    }
}
