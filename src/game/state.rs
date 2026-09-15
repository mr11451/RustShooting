#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum GameState {
    #[default]
    Title,
    Demo,
    StageIntro,
    Playing,
    StageClear,
    Respawn,
    GameOver,
    NameEntry,
    Ending,
}
