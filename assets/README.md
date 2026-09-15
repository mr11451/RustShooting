# 仮アセット

実装検証用の仮データです。本番素材へ差し替える場合も、定義ファイルのパスと ID 対応を維持します。

- `characters/player_level_00.gif` ～ `player_level_04.gif`: 成長レベル別の64x32、32x32タイル2フレーム自機スプライト
- `characters/stage01_enemy_02.gif` ～ `stage06_enemy_06.gif`: ステージ・敵IDごとの2フレーム仮アニメーション
- `characters/boss.gif`: 64x32、32x32 タイル 2 列のボス用スプライトシート
- `characters/growth_item.gif`: 64x32、32x32 タイル 2 列の成長アイテム用スプライトシート
- `bullets/bullets.gif`: 96x32、32x32 タイル 3 列の自弾・敵弾用スプライトシート
- `bullets/enemy_bullet_06.gif` ～ `enemy_bullet_16.gif`: 敵弾IDごとの16x8、8x8タイル2フレームアニメーション
- `backgrounds/stage01_atlas.png`: 64x64 の背景タイルアトラス
- `backgrounds/stage02_atlas.png` ～ `stage06_atlas.png`: ステージ2～6用の64x64背景タイルアトラス
- `audio/stage02_bgm.wav` ～ `stage06_bgm.wav`: ステージ2～6用の仮BGM
- `audio/stage01_bgm.wav`: ステージ1用の仮BGM
- `audio/fire.wav`: 発射効果音
- `audio/hit.wav`: 命中効果音
- `audio/destroy.wav`: 撃破効果音
- `audio/damage.wav`: 被弾効果音
- `data/characters.toml`: キャラクタ特性 ID と画像の対応
- `data/backgrounds.toml`: 背景 ID、タイルアトラス、スクロール設定
- `data/bullets.toml`: 自弾・敵弾の特性 ID と画像フレームの対応
- `data/audio.toml`: BGM・効果音の ID とファイルパスの対応
- `data/stage01_tilemap.txt`: ステージ 1 の仮タイル配置
