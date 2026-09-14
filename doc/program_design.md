# プログラム設計

## 1. 方針

MVP は、1 ステージ、固定長オブジェクト管理、30fps、640x320 論理画面で実装する。表示倍率は 1 倍または 2 倍とし、ゲームロジックは常に論理座標で処理する。

データ定義と実行時状態を分離する。

- 定義データ: ステージ、スケジュール、キャラクタ特性、軌道、敵弾発射、弾特性
- 実行時状態: 位置、速度、HP 現在値、残機、成長段階、無敵時間、使用中状態
- 固定長プール: 自キャラ、敵、自弾、敵弾、アイテム、エフェクト

## 2. モジュール構成

### 現在の実装構成

```text
src/
  main.rs          エントリポイント。World を生成してフレーム更新を呼び出す
  fixed.rs         符号付き Q12.4 の基本型
  sprite.rs        スプライトシートGIFのデコードとタイル分割
  input.rs         InputState と入力抽象 trait
  render.rs        描画抽象 trait
  audio.rs         音声イベントと音声抽象 trait
  game/
    mod.rs         ゲーム更新と状態遷移の入口
    state.rs       GameState
    world.rs       World とステージリセット
    update.rs      フレーム更新の公開入口
  runtime/
    mod.rs         ObjectState と固定長 ObjectPool
```

現在はゲームロジックとスプライトシートGIFのデコードを検証する内部フレームワーク段階であり、winit、wgpu、rodio の具体的な初期化処理は未接続である。描画・入力・音声は抽象 trait の境界を先に定義し、後続作業で各ライブラリへ接続する。

### 実装予定の構成

```text
src/
  main.rs                 winit / wgpu 初期化とアプリ起動
  app.rs                  アプリ全体とフレーム制御
  game/
    mod.rs
    state.rs              ゲーム状態と状態遷移
    world.rs              ステージと実行時ワールド
    update.rs             1 フレームの更新処理
    collision.rs          矩形当たり判定と衝突イベント
    spawn.rs              スケジュールとオブジェクト生成
    movement.rs           16方向、円、ベジェ、背景同期移動
    firing.rs             敵弾発射パターン処理
    scoring.rs            得点とランキング
  data/
    mod.rs
    stage.rs              StageData と定数配列
    character.rs          CharacterTrait
    orbit.rs              OrbitData
    firing.rs             FirePatternData
    bullet.rs              BulletCharacterData
  runtime/
    mod.rs
    object.rs             共通オブジェクト状態
    pool.rs               固定長プール
    player.rs             自キャラ固有状態
  input.rs                キーボード、パッド入力の抽象化
  render.rs               背景、キャラクタ、HUD、ランキング描画
  audio.rs                BGM、効果音、ミックス
  fixed.rs                符号付き Q12.4 演算
```

プラットフォーム依存と描画依存は `main.rs`、`input.rs`、`render.rs`、`audio.rs` の境界に閉じ込め、ゲームロジックから直接呼び出さない。

使用ライブラリ:

| ライブラリ | バージョン | 担当する処理 | 担当しない処理 |
|---|---:|---|---|
| `winit` | `0.30` | ウィンドウ生成、イベントループ、キーボード、基本的な入力イベント | ゲーム状態、当たり判定、スプライト描画、音声 |
| `wgpu` | `27` | GPU 初期化、テクスチャ、スプライト、背景、エフェクト、1 倍・2 倍描画 | ステージ進行、物理計算、入力割り当て、音声 |
| `egui` | `0.33` | デバッグ画面、開発用設定画面、データ確認画面 | ゲーム本編の描画、ゲーム状態、当たり判定 |
| `rodio` | `0.21` | WAV、BGM、効果音の再生、音量、同時再生、ミックス | ゲームイベントの判定、描画、入力 |

### 内製モジュールとの責務分担

| 内製モジュール | 担当する処理 |
|---|---|
| `game/state.rs` | タイトル、デモ、ステージ、ゲームオーバー、エンディングの状態遷移 |
| `game/update.rs` | 30fps の 1 フレーム更新順序 |
| `game/movement.rs` | 16 方向、円軌道、ベジェ曲線、背景同期移動 |
| `game/collision.rs` | 矩形当たり判定と衝突イベント生成 |
| `game/spawn.rs` | スケジュール、敵、弾、アイテムの生成と削除 |
| `game/firing.rs` | 敵弾発射パターンと発射位置の計算 |
| `game/scoring.rs` | 得点、ランキング、撃破時の加算 |
| `data/*` | ステージ、特性、軌道、発射、弾の定義データ |
| `runtime/pool.rs` | 固定長オブジェクトプール |
| `fixed.rs` | 符号付き `Q12.4` の加算、乗算、丸め、範囲処理 |
| `render.rs` | `wgpu` を使ったゲーム画面、HUD、ランキング、ターゲット枠の描画 |
| `input.rs` | `winit` の入力をゲーム共通入力へ変換 |
| `audio.rs` | ゲームイベントを `rodio` の再生要求へ変換 |

ゲームロジックはライブラリの型を直接保持せず、`InputState`、`RenderCommand`、`AudioEvent` などの内部型を介して連携する。これにより、ライブラリ変更や Windows/Linux の差分をゲーム本体へ波及させない。弾特性は `fire_sound_id` を発射時、`hit_sound_id` を命中時、敵特性は `destroy_sound_id` を撃破時の `AudioEvent` に対応付ける。

### キャラクタ画像の取り込み状況

現時点では、キャラクタ画像の読み込みと `wgpu` テクスチャへの変換は未実装である。`CharacterTrait.shape_id` は画像を参照するための ID として定義済みだが、画像ファイル、画像ローダー、テクスチャキャッシュ、スプライト描画は未接続である。画像は全ステージ分を起動時に読み込まず、各ステージの `StageIntro` で必要なデータだけを取り込む。

実装時は次の構成にする。

```text
assets/
  characters/
  bullets/
  effects/
  backgrounds/

src/
  asset.rs       画像ファイルの読み込みと ID 管理
  texture.rs     wgpu::Texture、TextureView、Sampler の管理
  render.rs      shape_id からテクスチャを選択して描画
```

画像取り込みの流れ:

1. `StageData` とスケジュールから、対象ステージで使用する `shape_id`、弾画像、エフェクト、背景を収集する
2. `shape_id` に対応する画像パスをアセット定義から解決する
3. スプライトシート GIF をデコードし、各フレームを RGBA8 のピクセルデータへ展開する
4. `wgpu::Queue::write_texture` で GPU テクスチャへ転送する
5. `TextureView` と `Sampler` をステージ単位のキャッシュへ登録する
6. 必要な画像の読み込み完了後に `Playing` へ遷移する
7. 描画時に `shape_id` からステージキャッシュのテクスチャを取得する
8. `CharacterTrait` の矩形サイズを論理座標へ変換してスプライトと当たり判定へ使用する

リボーン時は同じステージのキャッシュを再利用し、ステージ切り替え時に前ステージのキャッシュを解放して次ステージの `StageIntro` で入れ替える。読み込み中は敵を生成せず、スケジュールのフレームカウントも進めない。読み込み失敗時はエラーを記録し、代替矩形または代替テクスチャを登録してゲームを継続する。

### 音声データの取り込み

- 効果音はゲーム開始時に全件読み込み、ゲーム中はメモリ上の音声データを再利用する
- BGM は各ステージの `StageIntro` で、そのステージの画像アセットと同じタイミングに読み込む
- `StageIntro` は画像と BGM の読み込みが完了するまで `Playing` へ遷移しない
- リボーン時は同じステージの BGM キャッシュを再利用する
- ステージ切り替え時は前ステージの BGM を停止・解放し、次ステージの `StageIntro` で読み込む
- 効果音の読み込み失敗は代替無音データで継続し、BGM の読み込み失敗は無音 BGM とエラー記録で継続する

### キャラクタ画像形式

- キャラクタ画像はスプライトシート GIF とする
- 1 ファイルに同一サイズのアニメーションフレームを横またはタイル状に配置する
- GIF の再生タイミングは使用せず、ゲーム側の `animation_id`、フレーム番号、フレームレートで制御する
- GIF の各フレームを RGBA8 へ展開してから `wgpu` テクスチャへ転送する
- GIF の透過は読み込み時に RGBA のアルファ値へ変換する
- `SpriteSheet::from_gif_path` または `SpriteSheet::from_gif_bytes` で読み込む
- GIF の各アニメーションフレームを `animation_id`、シート内の各タイルを `tile_id` として保持する
- `SpriteSheet::frame(animation_id, tile_id)` で描画対象の RGBA8 フレームを参照する
- 画像が見つからない、またはデコードできない場合は生成色の矩形を代替表示し、ゲームループを停止させない

### 背景描画方式

背景は、**タイルアトラス PNG と幾何学描画の組み合わせ**を正式採用する。

- タイルアトラス PNG: 地形、建物、雲、模様などのベース背景
- タイルマップ: `tile_id` の配列で背景配置を定義
- 幾何学描画: 星、グリッド、流線、警告線、発光、ボス演出などの前景エフェクト
- 背景速度: スケジュールから符号付き `Q12.4` で更新
- 描画順: 背景タイル、幾何学背景、キャラクタ、弾、エフェクト、HUD

背景タイルはキャラクタ GIF と分け、PNG を RGBA8 テクスチャとして `StageIntro` 中に読み込む。タイル配置データとタイル画像を分離することで、同じタイルセットを複数ステージで再利用できる。

タイル画像のデータ表現:

```rust
struct TileSet {
  texture_id: u16,
  tile_width: u16,
  tile_height: u16,
  atlas_columns: u16,
  atlas_rows: u16,
}

struct TileMap {
  width: u16,
  height: u32,
  tile_ids: Vec<u16>,
}
```

- タイルアトラス PNG は同一サイズの画像を格子状に配置する
- `tile_id` はアトラス内の画像番号で、`0` は空タイルとする
- タイル ID から `atlas_x = tile_id % atlas_columns`、`atlas_y = tile_id / atlas_columns` を求める
- 論理画面上の描画位置は `screen_x = column * tile_width`、`screen_y = row * tile_height - scroll_y` とする
- 長いステージでは `TileMap` を行単位のチャンクに分割し、表示範囲のチャンクだけを参照する
- タイル配置データは画像本体と分離し、同じタイルアトラスを複数ステージで共有できるようにする

## 3. Windows / Linux 対応

ゲームロジックは Windows と Linux で共通化し、OS 依存処理をプラットフォーム層へ分離する。

共通化する処理:

- ゲーム状態、ステージ、スケジュール
- `Q12.4` 演算、16 方向、軌道計算
- 固定長プール、当たり判定、HP、残機、得点
- キャラクタ特性、弾特性、敵弾発射データ

プラットフォームごとに吸収する処理:

- ウィンドウ生成、表示倍率、フルスクリーン
- キーボードとコントロールパッド入力
- WAV、BGM、効果音の再生
- ファイルパス、ランキング保存先
- タイマーと終了処理

```rust
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;
```

ゲーム側は `PlatformInput`、`PlatformAudio`、`PlatformWindow` などの共通 trait に依存し、Windows/Linux の実装を直接参照しない。`winit`、`wgpu`、`rodio` の呼び出しもこの境界内に限定する。

Windows と Linux で、入力デバイス、音声デバイス、フルスクリーン、保存先の挙動が異なる可能性があるため、両環境で起動、入力、描画、音声、ランキング保存を確認する。

## 4. ゲーム状態

```rust
enum GameState {
    Title,
    Demo,
    NameEntry,
    StageIntro,
    Playing,
    StageClear,
    GameOver,
    Ending,
}
```

状態遷移:

```text
Title --入力--> StageIntro
Title --30秒無入力--> Demo
Playing --ハイスコアトップ10--> NameEntry --30秒経過--> Demo
Demo --入力--> StageIntro
StageIntro --90フレーム--> Playing
Playing --ボス撃破--> StageClear
StageClear --90フレーム、ステージ6以外--> StageIntro
StageClear --90フレーム、ステージ6--> Ending
Playing --被弾後もHP0、残機0--> GameOver
GameOver --30秒--> Title
```

`StageIntro` と `StageClear` では敵の生成とステージスケジュールのフレーム進行を停止する。

## 5. 定義データ

### StageData

```rust
struct StageData {
    stage_id: u8,
    play_frames: u32,
    intro_frames: u16,
    clear_frames: u16,
    boss_character_id: u16,
    schedule_start_id: u16,
    item_count: u8,
}
```

### CharacterTrait

`character_id` から参照する読み取り専用データ。敵の現在 HP や位置は含めない。

```rust
struct CharacterTrait {
    character_id: u16,
    character_type: CharacterType,
    shape_id: u16,
    animation_id: u16,
    hitbox_width: i16,
    hitbox_height: i16,
    max_hp: u16,
    contact_damage: u16,
    player_damage: u16,
    score: u32,
    default_orbit_id: u16,
    default_fire_pattern_id: u16,
    bullet_character_id: u16,
    growth_effect_id: u16,
    destroy_effect_id: u16,
}
```

### 参照関係

```text
StageData
  -> ScheduleData
       -> character_id -> CharacterTrait
       -> orbit_id -> OrbitData
       -> fire_pattern_id -> FirePatternData
FirePatternData
  -> bullet_character_id -> BulletCharacterData
BulletCharacterData
  -> shape_id / target_frame_id
```

ID `0` は無効値とする。参照先が存在しない場合は、その行の生成を中止してエラーを記録する。

## 6. 実行時状態

```rust
struct ObjectState {
    active: bool,
    object_type: ObjectType,
    character_id: u16,
    x: i16,
    y: i16,
    velocity_x: i16,
    velocity_y: i16,
    hp: u16,
    orbit_id: u16,
    orbit_frame: u32,
    fire_pattern_id: u16,
    fire_frame: u32,
    animation_frame: u16,
}

struct PlayerState {
    object: ObjectState,
    max_hp: u16,
    lives: u8,
    invincible_frames: u16,
    growth_level: u8,
    recovery_stock: u8,
    shot_cooldown: u8,
}
```

`x`、`y`、速度、軌道制御点、発射位置オフセットは符号付き `Q12.4` とする。`Q12.4` は格納値を 16 で割って実値として扱う。

## 7. 固定長プール

```rust
const PLAYER_CAPACITY: usize = 1;
const ENEMY_CAPACITY: usize = 16;
const PLAYER_BULLET_CAPACITY: usize = 16;
const ENEMY_BULLET_CAPACITY: usize = 128;
```

プールは `active` が false のスロットを先頭から検索する。空きがない場合は新規生成を破棄し、ゲームループは継続する。

リボーン時には敵、敵弾、自弾、アイテムのプールをクリアする。プレイヤーの成長段階と回復ストックは保持する。

## 8. フレーム処理

毎フレームの基本順序:

1. winit から入力を取得
2. ゲーム状態と入力タイマーを更新
3. 前フレームの衝突イベントを反映
4. プレイヤーを移動し、発射要求を処理
5. 背景を移動
6. ステージスケジュールを現在フレームまで進める
7. 敵、弾、アイテムを移動
8. 敵弾発射パターンを処理
9. 画面外オブジェクトを削除
10. 描画用スナップショットを作成
11. キャラクタ、背景、HUD、ターゲット枠を描画
12. ダブルバッファを表示
13. 描画スナップショットに対して当たり判定を実行
14. 衝突イベント、音声イベント、得点イベントを次フレームへ登録

`StageIntro` と `StageClear` では、ステージスケジュール、敵生成、敵弾発射を停止する。

## 9. 移動処理

### 直線・固定方向

16 方向を速度ベクトルへ変換し、Q12.4 のまま位置へ加算する。

### 円軌道

- 軌道開始時の中心、半径、開始角度を保存する
- `rotation` に応じて角度を更新する
- `end_angle` があれば到達時、なければ `duration_frames` 到達時に終了する

### 三次ベジェ曲線

出現位置を `P0`、相対制御点を `P1/P2`、相対終点を `P3` とする。実際の曲線点は次で計算する。

```text
B(t) = (1-t)^3 P0 + 3(1-t)^2 t P1 + 3(1-t)t^2 P2 + t^3 P3
```

MVP では `duration_frames` に応じて `t` を進める。一定速度が必要になった場合は弧長テーブルを追加する。

### 軌道終了後

敵は削除せず、背景スクロール量に同期して位置を更新する。敵の画像全体が画面外へ出ても、軌道終了後の敵は撃破またはステージ終了まで存続する。

## 10. 生成と削除

- スケジュールの `frame` が現在フレームに到達したらオブジェクトを生成する
- ボスは `play_frames - 1` のスケジュールで生成する
- 自弾、敵弾、成長アイテムは画像矩形全体が画面外になった時点で削除する
- 敵は軌道適用中なら画面外でも削除しない
- 敵の軌道終了後も背景同期状態として保持する
- 固定長プールが満杯の場合は生成を破棄する

## 11. 衝突処理

矩形は中心座標と半幅・半高で管理し、境界が接触した場合を命中とする。

- 自弾対敵: 敵 HP を弾のダメージ分減らす
- 通常弾: 最初の対象への命中後に削除
- 貫通弾: 同じフレームに対象ごとに 1 回命中
- 敵弾対自キャラ: 敵弾を削除し、特性の `player_damage` を適用
- 敵対自キャラ: 特性の `contact_damage` を適用
- 無敵中の自キャラは敵と敵弾から命中しない
- HP が 0 になったら成長を初期化し、残機を減らしてステージ先頭から再開
- 残機が 0 の場合はゲームオーバーへ遷移

## 12. 入力・HUD・ランキング

- 矢印キーまたはパッドの方向入力で移動
- スペースキーまたはパッドの A ボタンで発射
- 発射ボタンを押している間は 6 フレームごとに発射
- HUD は画面最下部に HP、得点、残自キャラ数、ストックを表示
- デモではランキングを 10 位まで表示し、名前は 3 文字で管理

ランキング登録時の名前入力画面は、次の固定文字盤を表示する。入力文字数は最大3文字で、`[del]` は現在の文字を削除し、`[spc]` は空白を入力する。カーソル移動と決定ボタンの割り当ては入力デバイス仕様で確定する。

```text
ABCDFEG
HIJKLMN
OPQRSTU
VWXYZ.-
0123456
789<>[del][spc]
```

## 13. 最初に実装する範囲

1. `Q12.4` 演算と 16 方向ベクトル
2. 固定長プール
3. `Title -> StageIntro -> Playing` の状態遷移
4. 自キャラの移動と自弾発射
5. 敵 1 種の生成と直線移動
6. 敵弾 1 種の発射
7. 矩形当たり判定
8. HP、残機、リボーン、ゲームオーバー
9. ボス撃破とステージクリア
10. HUD 表示

デモ、ランキング、円軌道、ベジェ曲線、ホーミング、複数ステージ、音声合成は、MVP の基本ループ確認後に追加する。

## 14. 使用バージョン

2026-09-15 時点の採用方針:

- Rust: `1.98.0` (`stable-x86_64-pc-windows-msvc`)
- winit: `0.30`
- wgpu: `27`
- egui: `0.33`
- rodio: `0.21`
- `Cargo.toml` の依存バージョンと `Cargo.lock` に上記バージョンを固定する
- nightly 版や開発中の依存版は使用しない

## 15. 保留事項

- Rust、winit、wgpu、egui、rodio の具体的な API 利用範囲
- Q12.4 の乗算、丸め、オーバーフロー処理
- 16 方向の基準角度と座標変換
- ベジェ曲線の一定速度化
- ホーミング対象の選択規則と対象枠表示
- ランキングの保存先と同点順位
