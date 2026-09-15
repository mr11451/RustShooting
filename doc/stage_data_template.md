# ステージデータひな型

## 目的

ステージの進行、敵の出現、背景スクロール、ボス、成長アイテムを定義するための記入用テンプレート。

MVP では、この Markdown の内容を確認したうえで Rust の定数配列または静的配列へ転記する。

## 共通ルール

- 論理画面サイズは `480x640`
- 表示サイズは `480x640` またはその 2 倍の `960x1280`
- ステージデータの座標は常に論理画面 `480x640` を基準とし、表示倍率では変更しない
- 処理速度は `60fps`
- `StageIntro` 終了後の最初のフレームを `0` とする
- `StageIntro` と `StageClear` の間はスケジュールのフレームカウントを進めない
- 各ステージで使用する画像は、そのステージの `StageIntro` 中に読み込む
- キャラクタ画像の形式はスプライトシート GIF とする
- GIF の各フレームは同一サイズで、`shape_id` と `animation_id` から参照する
- GIF の再生タイミングはゲーム側のフレーム管理で制御する
- 実装では `SpriteSheet::from_gif_path` または `SpriteSheet::from_gif_bytes` で読み込み、GIFフレームを `animation_id`、シート内タイルを `tile_id` で参照する
- `StageIntro` 中は必要画像の読み込みが完了するまで `Playing` へ遷移しない
- 効果音はゲーム開始時に全件読み込む
- BGM は各ステージの `StageIntro` 中に画像と同時に読み込む
- `StageIntro` 中は画像と BGM の読み込みが完了するまで `Playing` へ遷移しない
- リボーン時は同じステージの画像キャッシュを再利用し、ステージ切り替え時に入れ替える
- 座標の原点は画面左上
- `x` は右方向、`y` は下方向を正とする
- 画面内の論理座標は `x: 0..=479`、`y: 0..=639` を基本とする
- 画面外から出現させる場合は、画面外座標を許可する
- ID `0` は未設定または無効値とし、実データは `1` 以上の通し番号を使う
- 同じフレームに複数のスケジュール行がある場合は、ID の小さい順に処理する
- 自弾と敵弾は、画像矩形の全体が画面外へ出た時点で削除する。中心座標や一部が画面外になっただけでは削除しない
- 自弾と敵弾の画像矩形を中心座標 `(x, y)`、半幅 `half_width`、半高 `half_height` で表す場合、`x + half_width < 0`、`x - half_width >= 480`、`y + half_height < 0`、`y - half_height >= 640` のいずれかを満たした時点で削除する
- 画面外へ出た成長アイテムも削除する
- 敵は軌道データの適用中であれば画面外へ出ても削除しない
- 敵の軌道データが終了した後は、敵の位置更新を背景スクロール量に同期し、画像矩形全体が画面外へ出た時点で削除する
- 画面内の敵、弾、アイテムが固定長容量を超える場合は、暫定仕様では新規生成を破棄する
- 方向は 32 方向で管理する
- 速度と位置は符号付き `Q12.4` 固定小数点で管理する。符号付き 16 ビット値の下位 4 ビットを小数部とする
- `Q12.4` の実値は格納値を 16 で割って求める。例: `80.0` は `1280`、`2.0` は `32`
- 値は 2 の補数で表現し、負の速度や左・上方向の座標も扱える
- 符号付き `Q12.4` の表現範囲は `-2048.0` 以上 `2047.9375` 以下とする
- 加速度、背景速度、軌道半径も速度または距離として `Q12.4` で管理する
- 背景はタイルアトラス PNG と幾何学描画を組み合わせる
- タイルアトラス PNG は `StageIntro` 中に読み込み、幾何学描画のパラメータはステージ定義で管理する

## 1. ステージ定義

| 項目 | 値 | 備考 |
|---|---|---|
| ステージ ID | `1` | `1..=6` |
| ステージ名 | `ステージ1` | 仮名称 |
| ステージ長 | `5400` フレーム | 3 分相当。要調整 |
| 開始演出 | `90` フレーム | 約 3 秒。敵なし |
| クリア演出 | `90` フレーム | 約 3 秒。敵なし |
| クリア条件 | `ボス撃破` | ボス ID を指定 |
| ボス特性 ID | `0` | 特性データの ID |
| 初期背景 ID | `0` | 背景データの ID |
| BGM ID | `0` | 音声データの ID |
| スケジュール ID | `0` | 使用するスケジュールの先頭 ID |
| 成長アイテム数 | `4` | 仮値 |

### ステージ定義記入欄

```text
Stage {
    stage_id: <1..6>,
    name: <ステージ名>,
    play_frames: <基本フレーム数>,
    intro_frames: <開始演出フレーム数>,
    clear_frames: <クリア演出フレーム数>,
    boss_character_id: <ボス特性 ID>,
    background_id: <背景 ID>,
    bgm_id: <BGM ID>,
    schedule_start_id: <スケジュール開始 ID>,
    item_count: <成長アイテム数>,
}
```

## 2. 画面構成データ

ゲーム画面の HUD とデモ画面のランキングを定義する。座標は論理画面 `480x640` 基準で、表示倍率には依存しない。

### ゲーム画面 HUD

HUD は画面最下部に 1 行で配置し、HP、得点、残自キャラ数、ストック状況を表示する。`SCORE` などの説明用固定ラベルは表示せず、値とゲージだけで構成する。

| 項目 | 型の候補 | 初期位置・サイズ | 内容 |
|---|---|---|---|
| `hp_gauge` | `HudElement` | `x=8, y=627, width=120, height=10` | 現在 HP と最大 HP の割合を表示 |
| `score` | `HudElement` | `x=140, y=628, width=120, height=8` | 得点値を表示 |
| `lives` | `HudElement` | `x=280, y=628, width=80, height=8` | 残自キャラ数を表示 |
| `stock` | `HudElement` | `x=380, y=628, width=90, height=8` | 成長または HP 回復ストックを表示 |

### デモランキング

デモ画面ではスコアランキングを 10 位まで表示する。名前は 3 文字固定幅とし、3 文字未満の場合は空白で埋める。

ランキング登録時は次の文字盤を表示して名前を選択する。入力は最大3文字とし、`[del]` で現在の文字を削除、`[spc]` で空白を入力する。カーソル移動・決定操作のキー割り当ては入力デバイス定義で管理する。

```text
ABCDFEG
HIJKLMN
OPQRSTU
VWXYZ.-
0123456
789<>[del][spc]
```

| 項目 | 型の候補 | 内容 |
|---|---|---|
| `rank` | `u8` | 1..=10 |
| `name` | `[u8; 3]` | 3 文字。未使用位置は空白 |
| `score` | `u32` | 登録得点 |

```text
ScreenLayout {
    hud: {
        hp_gauge: { x: 8, y: 304, width: 160, height: 8 },
        score: { x: 176, y: 304, width: 160, height: 8 },
        lives: { x: 344, y: 304, width: 96, height: 8 },
        stock: { x: 448, y: 304, width: 184, height: 8 },
    },
    demo_ranking_count: 10,
    ranking_name_length: 3,
}
```

## 3. スケジュールデータ

### 項目定義

| 項目 | 型の候補 | 内容 |
|---|---|---|
| `schedule_id` | `u16` | スケジュール ID。1 以上の通し番号 |
| `stage_id` | `u8` | 所属ステージ。1..=6 |
| `frame` | `u32` | ステージ本編開始からのフレーム。0..=5399 |
| `spawn_x` | `i16` | 出現位置の X 座標。`Q12.4` 固定小数点 |
| `spawn_y` | `i16` | 出現位置の Y 座標。`Q12.4` 固定小数点 |
| `character_id` | `u16` | 敵またはアイテムの特性 ID |
| `object_type` | `enum` | `Enemy`、`Boss`、`GrowthItem`、`Background` など |
| `orbit_id` | `u16` | 使用する軌道 ID。不要なら 0 |
| `difficulty` | `u8` | 難易度レベル。1..=4 |
| `fire_pattern_id` | `u16` | 敵弾発射設定 ID。発射位置、タイミング、弾種を別データから参照。不要なら 0 |
| `background_speed` | `i16` | `Q12.4` 固定小数点。この行から適用する背景速度。不要なら現在値を維持 |
| `enabled` | `bool` | 使用する行かどうか |

### スケジュール記入欄

```text
Schedule {
    schedule_id: <ID>,
    stage_id: <ステージ ID>,
    frame: <フレーム>,
    spawn_x: <X>,
    spawn_y: <Y>,
    object_type: <Enemy|Boss|GrowthItem|Background>,
    character_id: <特性 ID>,
    orbit_id: <軌道 ID>,
    difficulty: <1..4>,
    fire_pattern_id: <敵弾発射設定 ID>,
    background_speed: <Q12.4 速度または変更なし>,
    enabled: true,
}
```

### スケジュール記入例

| schedule_id | stage_id | frame | spawn_x | spawn_y | object_type | character_id | orbit_id | difficulty | fire_pattern_id | background_speed |
|---:|---:|---:|---:|---:|---|---:|---:|---:|---:|---:|
| 1 | 1 | 0 | 1280 | -384 | Enemy | 1 | 1 | 1 | 1 | 16 |
| 2 | 1 | 90 | 5120 | -384 | Enemy | 2 | 2 | 1 | 2 | 16 |
| 3 | 1 | 900 | 2560 | 1920 | GrowthItem | 1 | 0 | 1 | 0 | 32 |
| 4 | 1 | 5399 | 5120 | -512 | Boss | 100 | 3 | 4 | 10 | 0 |

## 4. キャラクタ特性データ

キャラクタ特性データは `character_id` で参照する。自キャラ、敵、ボス、成長アイテムを同じ ID テーブルで管理し、オブジェクト固有の状態は実行時オブジェクトへ持たせる。

### 特性項目

| 項目 | 型の候補 | 内容 |
|---|---|---|
| `character_id` | `u16` | キャラクタ特性 ID。1 以上の通し番号 |
| `character_type` | `enum` | `Player`、`Enemy`、`Boss`、`GrowthItem` |
| `shape_id` | `u16` | スプライトまたは画像形状 ID |
| `animation_id` | `u16` | アニメーション定義 ID。不要なら 0 |
| `hitbox_width` | `i16` | 中心からの矩形幅。符号付き `Q12.4` |
| `hitbox_height` | `i16` | 中心からの矩形高さ。符号付き `Q12.4` |
| `max_hp` | `u16` | 初期 HP または敵の最大 HP。アイテムでは 0 |
| `contact_damage` | `u16` | 自キャラへ与える接触ダメージ。敵・敵弾で使用 |
| `player_damage` | `u16` | 自キャラへ与えるダメージ。敵・敵弾で使用 |
| `score` | `u32` | 撃破時に加算する得点。敵・ボスで使用 |
| `default_orbit_id` | `u16` | 標準軌道 ID。スケジュール指定を優先 |
| `default_fire_pattern_id` | `u16` | 標準敵弾発射パターン ID。不要なら 0 |
| `bullet_character_id` | `u16` | 自キャラまたは敵が使用する弾特性 ID。不要なら 0 |
| `growth_effect_id` | `u16` | 成長アイテムの効果 ID。不要なら 0 |
| `destroy_effect_id` | `u16` | 撃破時エフェクト ID。不要なら 0 |
| `destroy_sound_id` | `u16` | 撃破時に再生する効果音 ID。敵・ボスで使用。不要なら 0 |

### 特性データ記入欄

```text
CharacterTrait {
    character_id: <ID>,
    character_type: <Player|Enemy|Boss|GrowthItem>,
    shape_id: <画像形状 ID>,
    animation_id: <アニメーション ID または0>,
    hitbox_width: <Q12.4幅>,
    hitbox_height: <Q12.4高さ>,
    max_hp: <HPまたは0>,
    contact_damage: <接触ダメージ>,
    player_damage: <自キャラダメージ>,
    score: <撃破得点>,
    default_orbit_id: <軌道 ID または0>,
    default_fire_pattern_id: <発射パターン ID または0>,
    bullet_character_id: <弾特性 ID または0>,
    growth_effect_id: <成長効果 ID または0>,
    destroy_effect_id: <破壊エフェクト ID または0>,
    destroy_sound_id: <撃破効果音 ID または0>,
}
```

### 特性データ例

| character_id | character_type | shape_id | hitbox_width | hitbox_height | max_hp | contact_damage | player_damage | score | default_orbit_id | default_fire_pattern_id | growth_effect_id |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | Enemy | 1 | 256 | 256 | 10 | 10 | 10 | 100 | 1 | 1 | 0 |
| 100 | Boss | 100 | 1024 | 768 | 1000 | 50 | 50 | 10000 | 3 | 10 | 0 |
| 200 | GrowthItem | 200 | 192 | 192 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |

## 5. 軌道データ

### 軌道定義

| 項目 | 型の候補 | 内容 |
|---|---|---|
| `orbit_id` | `u16` | 軌道 ID。1 以上の通し番号 |
| `orbit_type` | `enum` | `Straight`、`Circle`、`Bezier` など |
| `direction` | `enum` | 32方向。1方位11.25度。 |
| `speed` | `i16` | 1 フレームあたりの移動量。`Q12.4` 固定小数点 |
| `radius` | `i16` | 円軌道の半径。`Q12.4` 固定小数点。不要なら 0 |
| `control_point_1` | `(i16, i16)` | ベジェ曲線の第 1 制御点。出現位置からの相対座標。`Q12.4` 固定小数点 |
| `control_point_2` | `(i16, i16)` | ベジェ曲線の第 2 制御点。出現位置からの相対座標。`Q12.4` 固定小数点 |
| `control_point_end` | `(i16, i16)` | ベジェ曲線の終点。出現位置からの相対座標。`Q12.4` 固定小数点 |
| `start_angle` | `u16` | 円軌道の開始角度。1 周を `0..=65535` で表す。円軌道では必須 |
| `end_angle` | `Option<u16>` | 円軌道の終了角度。角度で終了しない場合は未設定 |
| `duration_frames` | `u32` | 軌道を適用するフレーム数。0 は無期限の候補 |
| `acceleration` | `i16` | 加速度。`Q12.4` 固定小数点。不要なら 0 |
| `target_direction` | `enum` | `Fixed`、`PlayerAtSpawn`、`PlayerTracking` など |
| `next_orbit_id` | `u16` | 終了後に切り替える軌道。不要なら 0 |
| `rotation` | `i8` | 円軌道の回転方向。`1` は時計回り、`-1` は反時計回り |

### 軌道記入欄

```text
Orbit {
    orbit_id: <ID>,
    orbit_type: <Straight|Circle|Bezier>,
    direction: <32方向>,
    speed: <Q12.4単位速度>,
    radius: <Q12.4回転半径>,
    control_point_1: <Q12.4相対X, Q12.4相対Y>,
    control_point_2: <Q12.4相対X, Q12.4相対Y>,
    control_point_end: <Q12.4相対X, Q12.4相対Y>,
    start_angle: <円軌道の開始角度>,
    end_angle: <円軌道の終了角度または未設定>,
    duration_frames: <フレーム数>,
    acceleration: <Q12.4加速度>,
    target_direction: <Fixed|PlayerAtSpawn|PlayerTracking>,
    next_orbit_id: <次の軌道 ID>,
    rotation: <1または-1>,
}
```

### 軌道記入例

| orbit_id | orbit_type | direction | speed | radius | control_point_1 | control_point_2 | control_point_end | start_angle | end_angle | duration_frames | acceleration | target_direction | next_orbit_id | rotation |
|---:|---|---:|---:|---:|---|---|---|---:|---:|---:|---:|---|---:|---:|
| 1 | Straight | Down | 32 | 0 | - | - | - | - | - | 540 | 0 | Fixed | 0 | 0 |
| 2 | Circle | Right | 32 | 768 | - | - | - | 0 | - | 180 | 0 | Fixed | 1 | 1 |
| 3 | Bezier | Down | 16 | 0 | 0,0 | 1600,0 | 5120,2560 | - | - | 900 | 0 | Fixed | 0 | 0 |

## 6. MVP 用の最小データ例

### ステージ定義

```text
stage_id: 1
play_frames: 5400
intro_frames: 90
clear_frames: 90
boss_character_id: 100
schedule_start_id: 1
item_count: 4
```

### スケジュール

```text
1: frame=0,    type=Enemy,       character=1,   x=1280, y=-384, orbit=1, difficulty=1
2: frame=90,   type=Enemy,       character=2,   x=5120, y=-384, orbit=2, difficulty=1
3: frame=900,  type=GrowthItem,  character=1,   x=2560, y=1920,  orbit=0, difficulty=1
4: frame=1800, type=Enemy,       character=1,   x=7680, y=-384, orbit=1, difficulty=1
5: frame=3600, type=GrowthItem,  character=1,   x=5120, y=1920,  orbit=0, difficulty=1
6: frame=5399, type=Boss,         character=100, x=5120, y=-512, orbit=3, difficulty=4
```

### 軌道

```text
1: type=Straight,  direction=Down, speed=32, radius=0,  duration=540
2: type=Circle,    direction=Right, speed=32, radius=768, duration=180, rotation=1
3: type=Bezier, direction=Down, speed=16, radius=0, control1=(0,0), control2=(1600,0), end=(5120,2560), duration=900
```

## 7. 敵弾発射データ

敵弾の発射位置は軌道データに含めず、敵弾発射データで管理する。軌道は敵の移動だけを担当し、発射データは敵の現在位置を基準に発射位置を計算する。

### 発射データ項目

| 項目 | 型の候補 | 内容 |
|---|---|---|
| `fire_pattern_id` | `u16` | 発射パターン ID。1 以上の通し番号 |
| `fire_frame` | `u32` | 敵の生成または軌道開始からの経過フレーム |
| `spawn_offset_x` | `i16` | 敵の中心からの X 相対位置。符号付き `Q12.4` |
| `spawn_offset_y` | `i16` | 敵の中心からの Y 相対位置。符号付き `Q12.4` |
| `direction_type` | `enum` | `Fixed16`、`ToPlayerAtFire`、`ToPlayerTracking` など |
| `direction` | `Direction16` | 固定方向。追尾発射では未使用 |
| `bullet_character_id` | `u16` | 敵弾の特性 ID |
| `bullet_count` | `u8` | 同時発射数 |
| `interval_frames` | `u16` | 繰り返し発射の間隔。繰り返さない場合は 0 |
| `enabled` | `bool` | 使用する行かどうか |

### 発射データ記入欄

```text
FirePattern {
    fire_pattern_id: <ID>,
    fire_frame: <敵生成または軌道開始からのフレーム>,
    spawn_offset_x: <Q12.4相対X>,
    spawn_offset_y: <Q12.4相対Y>,
    direction_type: <Fixed16|ToPlayerAtFire|ToPlayerTracking>,
    direction: <32方向または未設定>,
    bullet_character_id: <敵弾特性 ID>,
    bullet_count: <発射数>,
    interval_frames: <間隔または0>,
    enabled: true,
}
```

敵の現在位置を `(enemy_x, enemy_y)`、発射位置の相対座標を `(offset_x, offset_y)` とすると、敵弾の初期位置は次で求める。

```text
bullet_x = enemy_x + offset_x
bullet_y = enemy_y + offset_y
```

## 8. 弾特性データ

自弾と敵弾の種類、形状、移動特性は特性 ID で管理する。発射位置と発射タイミングは `FirePattern` が管理し、弾の性質はこのデータから参照する。

### 弾特性項目

| 項目 | 型の候補 | 内容 |
|---|---|---|
| `bullet_character_id` | `u16` | 弾特性 ID。1 以上の通し番号 |
| `owner_type` | `enum` | `Player` または `Enemy` |
| `shape_id` | `u16` | 弾の画像・形状 ID |
| `motion_type` | `enum` | `Straight`、`FixedDirection`、`Homing`、`Tracking` など |
| `direction` | `Direction16` | 固定方向。固定方向以外では未使用 |
| `speed` | `i16` | 弾速。符号付き `Q12.4` |
| `acceleration` | `i16` | 加速度。符号付き `Q12.4` |
| `damage` | `u16` | 命中時のダメージ |
| `penetrating` | `bool` | 敵に命中しても消滅しないか |
| `homing_target` | `enum` | `None`、`Enemy`、`Player` |
| `target_lock` | `enum` | `AtFire`、`Continuous` |
| `hitbox_width` | `i16` | 当たり判定矩形の幅。`Q12.4` |
| `hitbox_height` | `i16` | 当たり判定矩形の高さ。`Q12.4` |
| `target_frame_id` | `u16` | ホーミング対象表示枠の画像 ID。不要なら 0 |
| `fire_sound_id` | `u16` | 発射時に再生する効果音 ID。不要なら 0 |
| `hit_sound_id` | `u16` | 命中時に再生する効果音 ID。不要なら 0 |

### 弾特性記入欄

```text
BulletCharacter {
    bullet_character_id: <ID>,
    owner_type: <Player|Enemy>,
    shape_id: <形状 ID>,
    motion_type: <Straight|FixedDirection|Homing|Tracking>,
    direction: <32方向または未設定>,
    speed: <Q12.4弾速>,
    acceleration: <Q12.4加速度>,
    damage: <ダメージ>,
    penetrating: <true|false>,
    homing_target: <None|Enemy|Player>,
    target_lock: <AtFire|Continuous>,
    hitbox_width: <Q12.4幅>,
    hitbox_height: <Q12.4高さ>,
    target_frame_id: <対象枠画像 ID または0>,
    fire_sound_id: <発射効果音 ID または0>,
    hit_sound_id: <命中効果音 ID または0>,
}
```

## 9. Rust への転記例

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

struct ScreenLayout {
    hp_gauge: HudElement,
    score: HudElement,
    lives: HudElement,
    stock: HudElement,
    demo_ranking_count: u8,
    ranking_name_length: u8,
}

struct HudElement {
    x: i16,
    y: i16,
    width: u16,
    height: u16,
}

struct RankingEntry {
    rank: u8,
    name: [u8; 3],
    score: u32,
}

struct ScheduleData {
    schedule_id: u16,
    stage_id: u8,
    frame: u32,
    spawn_x: i16,
    spawn_y: i16,
    object_type: ObjectType,
    character_id: u16,
    orbit_id: u16,
    difficulty: u8,
    fire_pattern_id: u16,
    background_speed: i16,
}

struct OrbitData {
    orbit_id: u16,
    orbit_type: OrbitType,
    direction: Direction16,
    speed: i16,
    radius: i16,
    control_point_1: (i16, i16),
    control_point_2: (i16, i16),
    control_point_end: (i16, i16),
    start_angle: u16,
    end_angle: Option<u16>,
    duration_frames: u32,
    acceleration: i16,
    target_direction: TargetDirection,
    next_orbit_id: u16,
    rotation: i8,
}

struct FirePatternData {
    fire_pattern_id: u16,
    fire_frame: u32,
    spawn_offset_x: i16,
    spawn_offset_y: i16,
    direction_type: DirectionType,
    direction: Direction16,
    bullet_character_id: u16,
    bullet_count: u8,
    interval_frames: u16,
}

struct BulletCharacterData {
    bullet_character_id: u16,
    owner_type: OwnerType,
    shape_id: u16,
    motion_type: MotionType,
    direction: Direction16,
    speed: i16,
    acceleration: i16,
    damage: u16,
    penetrating: bool,
    homing_target: HomingTarget,
    target_lock: TargetLock,
    hitbox_width: i16,
    hitbox_height: i16,
    target_frame_id: u16,
}
```

## 10. 背景データ

背景はタイルアトラス PNG をベースにし、幾何学描画レイヤーを重ねる。

```text
BackgroundData {
    background_id: <ID>,
    tile_atlas_path: <PNGパス>,
    tile_width: <論理ピクセル幅>,
    tile_height: <論理ピクセル高さ>,
    atlas_columns: <列数>,
    atlas_rows: <行数>,
    map_width: <タイル列数>,
    map_height: <タイル行数>,
    tile_ids: <タイルID配列>,
    scroll_speed: <符号付きQ12.4速度>,
    geometry_layer_id: <幾何学レイヤーIDまたは0>,
}
```

### 背景レイヤー

- タイルアトラス PNG は地形、建物、雲、模様などのベース背景に使用する
- `tile_ids` はタイルアトラス内の画像番号を参照する
- `tile_id=0` は空タイルとし、実タイルは `1` 以上の `u16` で管理する
- タイルアトラス内の位置は `atlas_x = tile_id % atlas_columns`、`atlas_y = tile_id / atlas_columns` で求める
- 長いステージの `tile_ids` は行単位のチャンクに分割できる
- 背景速度はスケジュールで変更できる
- 幾何学レイヤーは星、グリッド、流線、警告線、発光、ボス演出に使用する
- タイル画像は `StageIntro` 中に読み込み、タイル配置データはステージ開始時に初期化する
- タイルアトラスが見つからない場合は単色または生成図形で代替する

## 11. 実装前に確定する項目

- 円軌道の開始角度は必須とし、直線移動とベジェ曲線では未設定を許可する
- 終了角度が未設定の場合は `duration_frames` で軌道を終了する
- 終了角度を設定した場合は、`rotation` の向きで角度に到達した時点を軌道終了とする
- 角度値 `0..=65535` の基準方向と、時計回り・反時計回りの角度増減を定義する
- 符号付き `Q12.4` のオーバーフロー、飽和、丸め処理
- `frame=0` で生成する対象の初期位置
- 自弾と敵弾は画像矩形全体が画面外になった時点で削除する。成長アイテムの削除境界も定義する
- 敵が軌道適用中に画面外へ出た場合の描画と当たり判定
- `duration_frames=0` の意味
- 軌道終了後に背景スクロールへ同期する具体的な移動方法
- ベジェ曲線は三次ベジェ曲線として、出現位置を `P0`、3 つの相対制御点を `P1/P2/P3` として解釈する
- ベジェ曲線の速度を一定にするための弧長近似またはパラメータ `t` の進め方
- 敵弾発射データの `fire_frame` を敵生成時点と軌道開始時点のどちらから数えるか
- `direction_type` ごとの方向計算と、発射位置オフセットの適用タイミング
- 自弾・敵弾の特性 ID と形状 ID の対応
- ホーミング対象の選択条件、発射時固定か継続追尾か、対象消滅時の挙動
- ホーミング対象を示す四角枠の表示サイズ、表示時間、描画レイヤー
- 貫通弾の命中回数、威力減衰、消滅条件
- ボスはステージ最終フレーム `play_frames - 1` に出現し、撃破をクリア条件とする
- 成長アイテムの取得判定と回復ストックへの変換条件
- 背景速度の単位と、背景速度変更行の適用期間
