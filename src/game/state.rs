#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum GameState {
    #[default]
    Title,
    Demo,
    StageIntro,
    Playing,
    StageClear,
    GameOver,
    NameEntry,
    Ending,
}
