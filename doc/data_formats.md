# データフォーマット仕様書

本書では、本プロジェクト（RustShooting）で使用されるすべてのデータフォーマット、数値表現、ファイル仕様、およびメモリ上のデータ構造を定義します。

---

## 1. 座標系と数値表現

### 1.1 論理画面と座標系
- **論理画面解像度**: 横 `480` × 縦 `640`（縦画面仕様）
- **表示倍率**: 1倍（`480x640`）または 2倍（`960x1280`）
- **原点**: 画面左上端を `(x: 0, y: 0)` とする。
- **軸方向**: X軸は右方向が正、Y軸は下方向が正。
- **画面内有効範囲**: `x: 0..=479`, `y: 0..=639`

### 1.2 Q12.4 固定小数点（`fixed::Q12_4`）
ゲームロジック内の位置座標、移動速度、加速度、当たり判定寸法、軌道制御点はすべて符号付き `Q12.4` 固定小数点で保持します。

- **ビット構成**: 符号付き 16 ビット整数（`i16`）。下位 4 ビットが小数部（$1/16 = 0.0625$ 単位）。
- **スケール**: 実数値 $V$ に対し、格納値は $V \times 16$。
- **表現範囲**: $-2048.0$（`-32768`）〜 $+2047.9375$（`32767`）。
- **主要な定数値**:
  - 画面幅 (`SCREEN_WIDTH_Q12`): `480 * 16 = 7,680`
  - 画面高 (`SCREEN_HEIGHT_Q12`): `640 * 16 = 10,240`
  - 自機初期位置: `(PLAYER_INITIAL_X_Q12, PLAYER_INITIAL_Y_Q12) = (3840, 8960)` （実座標 `(240.0, 560.0)`）
  - 自弾速度: 成長特性テーブルでQ12.4値を管理。現行値は `64 / 67 / 70 / 74 / 77`
  - 敵弾速度: 発射パターンデータ自体に1.25倍済みのQ12.4速度を設定。

### 1.3 32方向（`Direction32`）
自弾・敵弾・敵移動の向きは32等分（1方位あたり11.25°）で管理されます。既存の`Direction16`参照名は互換エイリアスとして残しています。

| インデックス | 方向名 | 角度 (時計回り, 0=上) |
|---|---|---|---|---|
| 0 | North | 0.0° |
| 1 | NorthByEast | 11.25° |
| 2 | NorthNorthEast | 22.5° |
| 3 | NorthEastByNorth | 33.75° |
| 4 | NorthEast | 45.0° |
| 5 | NorthEastByEast | 56.25° |
| 6 | EastNorthEast | 67.5° |
| 7 | EastByNorth | 78.75° |
| 8 | East | 90.0° |
| 9 | EastBySouth | 101.25° |
| 10 | EastSouthEast | 112.5° |
| 11 | SouthEastByEast | 123.75° |
| 12 | SouthEast | 135.0° |
| 13 | SouthEastBySouth | 146.25° |
| 14 | SouthSouthEast | 157.5° |
| 15 | SouthByEast | 168.75° |
| 16 | South | 180.0° |
| 17 | SouthByWest | 191.25° |
| 18 | SouthSouthWest | 202.5° |
| 19 | SouthWestBySouth | 213.75° |
| 20 | SouthWest | 225.0° |
| 21 | SouthWestByWest | 236.25° |
| 22 | WestSouthWest | 247.5° |
| 23 | WestBySouth | 258.75° |
| 24 | West | 270.0° |
| 25 | WestByNorth | 281.25° |
| 26 | WestNorthWest | 292.5° |
| 27 | NorthWestByWest | 303.75° |
| 28 | NorthWest | 315.0° |
| 29 | NorthWestByNorth | 326.25° |
| 30 | NorthNorthWest | 337.5° |
| 31 | NorthByWest | 348.75° |

### 1.4 ボス円軌道

ボス（キャラクターID `100..=105`）は、軌道データの中心・半径・弧長速度から円軌道を計算します。

- 円の中心: `(240, 220)` pixel
- 円の半径: `96` pixel
- 軌道: 半径と弧長速度から角速度を計算し、`next_orbit_id`で無限ループ
- 座標: 円の中心を保持して毎フレーム計算するため、加算誤差でドリフトしない
- 境界: ボス画像と当たり判定が画面内に収まる余白を確保

---

## 2. ゲーム定義データ構造

### 2.1 ステージ定義（`StageData`）
各ステージの全体構成を定義します。

```rust
pub struct StageData {
    pub stage_id: u8,           // ステージ番号 (1..=6)
    pub play_frames: u32,       // 本編ステージ長フレーム数 (10,800 = 3分 at 60fps)
    pub intro_frames: u16,      // 開始演出フレーム数 (90 = 3秒)
    pub clear_frames: u16,      // クリア演出フレーム数 (90 = 3秒)
    pub boss_character_id: u16, // ステージボスのキャラクターID
    pub schedule_start_id: u16, // スケジュールの先頭ID
    pub item_count: u8,         // ステージ内アイテム総数
}
```

### 2.2 スケジュールデータ（`ScheduleData`）
ステージ進行フレームに応じた敵・ボス・アイテムの出現を定義します。

```rust
pub struct ScheduleData {
    pub schedule_id: u16,       // スケジュール一意ID (1..)
    pub stage_id: u8,           // 所属ステージ (1..=6)
    pub frame: u32,             // 出現タイミング (0..play_frames-1)
    pub spawn_x: Q12_4,         // 出現X座標 (Q12.4)
    pub spawn_y: Q12_4,         // 出現Y座標 (Q12.4)
    pub object_type: ObjectType,// オブジェクト種別 (Enemy, Boss, GrowthItem, Background)
    pub character_id: u16,      // キャラクター特性ID
    pub orbit_id: u16,          // 適用軌道ID (0=静止/初期速度維持)
    pub difficulty: u8,         // 難易度レベル (1..=4)
    pub fire_pattern_id: u16,   // 敵弾発射パターンID (0=発射なし)
    pub background_speed: Q12_4,// このフレーム以降適用する背景スクロール速度
}
```

### 2.3 ステージ戦闘特性（`StageCombatData`）

ステージごとの通常敵、ボス、発射パターン、敵弾種をまとめて定義します。

```rust
pub struct StageCombatData {
  pub stage_id: u8,
  pub enemy_character_id: u16,
  pub boss_character_id: u16,
  pub enemy_fire_pattern_id: u16,
  pub boss_fire_pattern_id: u16,
  pub enemy_bullet_character_id: u16,
}
```

`STAGE_COMBAT_DATA`を変更すると、スケジュールの出現タイミングを変えずにステージの敵構成や敵弾種を変更できます。

### 2.4 キャラクター特性テーブル（`CharacterTrait`）
自機、各種敵、ボス、アイテムの基本ステータスを定義します。

```rust
pub struct CharacterTrait {
    pub character_id: u16,           // キャラクターID (1: 自機, 2..: 敵, 100..: ボス, 200: アイテム)
    pub character_type: CharacterType,// Player, Enemy, Boss, GrowthItem
    pub shape_id: u16,               // スプライト形状ID
    pub animation_id: u16,           // アニメーションID
    pub hitbox_width: i16,           // 当たり判定全幅 (Q12.4)
    pub hitbox_height: i16,          // 当たり判定全高 (Q12.4)
    pub max_hp: u16,                 // 最大HP (敵・ボスの耐久力、自機=100)
    pub contact_damage: u16,         // 自機との接触時に自機が受けるダメージ
    pub player_damage: u16,          // 敵弾等が自機に与えるダメージ
    pub score: u32,                  // 撃破時に加算される得点
    pub default_orbit_id: u16,       // デフォルト軌道ID
    pub default_fire_pattern_id: u16,// デフォルト発射パターンID
    pub bullet_character_id: u16,    // 発射する弾の特性ID
    pub growth_effect_id: u16,       // アイテム取得時等のエフェクトID
    pub destroy_effect_id: u16,      // 撃破時のエフェクトID (1: 通常爆発, 2: ボス大爆発)
    pub destroy_sound_id: u16,       // 撃破時の音声ID
}
```

敵・ボスの表示サイズは`EnemyVisualData`でIDごとに管理します。`width`/`height`は表示ピクセルであり、`CharacterTrait`の当たり判定サイズとは独立しています。

```rust
pub struct EnemyVisualData {
  pub character_id: u16,
  pub width: u16,
  pub height: u16,
}
```

ステージ別の敵画像ファイル名は`StageEnemyImageData`で管理します。固定長配列ではなくテーブルの行を検索するため、ステージごとに敵種類の行数を変えられます。

```rust
pub struct StageEnemyImageData {
  pub stage_id: u8,
  pub character_id: u16,
  pub image_file_name: &'static str,
}
```

全ステージ共通の画像ディレクトリは`STAGE_ENEMY_IMAGE_DIRECTORY`（`assets/characters`）で管理し、テーブルにはファイル名だけを記載します。

敵弾は`EnemyBulletProfileData`で発射元キャラクターIDごとに弾特性IDを割り当てます。通常敵ID `2..=6`とボスID `100..=105`は、それぞれ弾ID `6..=16`へ対応し、形状・当たり判定・ダメージ・画像フレームを個別に調整できます。

### 2.5 弾特性データ（`BulletCharacterData`）
自機弾および敵弾の衝突特性・威力を定義します。

```rust
pub struct BulletCharacterData {
    pub bullet_character_id: u16, // 弾ID (1: 自機通常, 2: 敵通常, 3: 敵重弾, 4: 自機貫通, 5: 敵高速)
    pub fire_sound_id: u16,       // 発射音ID
    pub hit_sound_id: u16,        // 命中音ID
    pub hitbox_width: i16,        // 当たり判定全幅 (Q12.4)
    pub hitbox_height: i16,       // 当たり判定全高 (Q12.4)
    pub damage: u16,              // 敵に与えるダメージ (自弾時)
    pub penetrating: bool,        // 貫通弾フラグ (trueの場合命中後も消滅しない)
    pub player_damage: u16,       // 自機に与えるダメージ (敵弾時)
}
```

### 2.6 自弾成長特性データ（`PlayerGrowthData`）

成長レベルごとの自弾発射設定をテーブルで管理します。ゲームコードはレベル別の数値を直接分岐せず、このテーブルを参照します。

```rust
pub struct PlayerGrowthData {
  pub growth_level: u8,        // 成長レベル 0..=4
  pub max_bullet_groups: u8,   // 同時保持できる発射グループ数
  pub bullets_per_group: u8,   // 1グループ内の横並び弾数
  pub bullet_character_id: u16,// 使用する自弾特性ID
  pub speed: Q12_4,            // 自弾の速さ。方向変換前の正値
  pub visual_width: u16,       // 自機表示幅
  pub visual_height: u16,      // 自機表示高
  pub hitbox_half_width: i16,  // 自機当たり判定の半幅
  pub hitbox_half_height: i16, // 自機当たり判定の半高
  pub move_speed: Q12_4,       // 自機移動速度
  pub directions: &'static [Direction16], // グループ内各弾の射出方向
}
```

現行の仮データは、グループ数 `4/6/8/10/12`、グループ内弾数 `1/2/3/4/5`、弾種ID `1`です。方向配列はレベルごとに定義し、北方向と左右の斜め方向を組み合わせて、発射時にQ12.4速度ベクトルへ変換します。

### 2.7 発射パターンデータ（`FirePatternData`）
敵が弾を発射するタイミング・オフセット・弾種・方向を定義します。

```rust
pub struct FirePatternData {
    pub fire_pattern_id: u16,    // 発射パターンID
    pub fire_frame: u32,         // 敵出現（軌道開始）からの発射フレーム
    pub spawn_offset_x: Q12_4,   // 敵中心からの発射Xオフセット (Q12.4)
    pub spawn_offset_y: Q12_4,   // 敵中心からの発射Yオフセット (Q12.4)
    pub bullet_character_id: u16,// 発射する弾特性ID
    pub direction: Direction16,  // 発射方向 (32方位。互換名)
    pub speed: Q12_4,            // 弾速 (Q12.4)
    pub bullet_velocity_x: Q12_4,// X方向初速
    pub bullet_velocity_y: Q12_4,// Y方向初速
}
```

### 2.8 軌道データ（`OrbitData`）
敵およびボスの移動軌道を定義します。

```rust
pub struct OrbitData {
    pub orbit_id: u16,                   // 軌道ID
    pub orbit_type: OrbitType,           // Straight, Circle, Bezier, MoveToPosition
    pub direction: Direction16,          // 初期進行方向 (32方位。互換名)
    pub speed: Q12_4,                    // 進行速度
    pub radius: Q12_4,                   // 円軌道の回転半径
    pub control_point_1: (Q12_4, Q12_4), // 三次ベジェ曲線の相対制御点 P1
    pub control_point_2: (Q12_4, Q12_4), // 三次ベジェ曲線の相対制御点 P2
    pub control_point_end: (Q12_4, Q12_4),// 三次ベジェ曲線の相対終点 P3
    pub target_position: (Q12_4, Q12_4),  // MoveToPositionの目標座標
    pub start_angle: Option<u16>,        // 円軌道開始角度
    pub end_angle: Option<u16>,          // 円軌道終了角度
    pub duration_frames: u32,            // 軌道持続フレーム数
    pub acceleration: Q12_4,             // 加速度
    pub next_orbit_id: u16,              // 完了後に移る任意の軌道ID (0=終了、自己指定で無限ループ)
    pub rotation: i8,                    // 回転方向 (+1: 時計回り, -1: 反時計回り)
}
```

ボスの仮シーケンスは、軌道 `8`（画面内の定位置へ90フレームで移動）から軌道 `9`（半径96ピクセルの円、弧長1ピクセル/frame）へ遷移します。1周は約603フレームです。軌道 `9`の`next_orbit_id`は`9`自身を指し、円の中心を保持したまま無限ループします。別の軌道IDを指定すれば、シーケンスの任意位置へ遷移できます。

ステージ6のボス（キャラクターID `105`）は、一般軌道ID `3`を共有せず、専用の軌道ID `10`を使用します。現時点の仮データでは軌道3と同じベジェ形状ですが、ステージ6専用に独立しているため、後から軌道だけを変更できます。

---

## 3. アセットファイルフォーマット

### 3.1 スプライトシート GIF（`assets/characters/*.gif`, `assets/bullets/*.gif`）
- **画像形式**: 標準 GIF アニメーション形式（透過色対応）。
- **タイルグリッド**: 同一サイズの正方形タイル（敵弾: `8x8`, 自機・通常敵: `32x32`, ボス: `64x64`）を横またはグリッド状に配置。
- **デコード仕様**:
  - `image::codecs::gif::GifDecoder` により全フレームを RGBA8 ピクセルバッファへ展開。
  - GIFフレーム番号を `animation_id`、画像内の各タイル位置を `tile_id` としてインデックス管理。
  - GPU 転送時に `Rgba8UnormSrgb` 2D テクスチャとしてバインド。

### 3.2 背景タイルアトラス PNG（`assets/backgrounds/*_atlas.png`）
- **画像形式**: 32bit RGBA PNG 形式。
- **構成**: `64x64` ピクセル画像内に `16x16` の背景タイルを 4列 × 4行（計16タイル）配置。
- **UVマッピング**:
  - タイルID $T$ ($0..15$) に対し、アトラス列 $C = T \pmod 4$、アトラス行 $R = \lfloor T / 4 \rfloor$。
  - テクスチャ座標: $U \in [0.25 C, 0.25 (C + 1)]$, $V \in [0.25 R, 0.25 (R + 1)]$。

### 3.3 タイルマップ配置テキスト（`data/*_tilemap.txt`）
ステージ背景のタイル配置をプレーンテキストで記述します。

- **フォーマット**:
  - スペースまたはタブで区切られたタイルID（整数）を行列状に記述。
  - 空白行は無視。
  - ID `0` は「描画なし（透明/スキップ）」。
  - ID `1..=15` は背景アトラス内のタイル番号。
- **例 (`data/stage01_tilemap.txt`)**:
  ```text
  0 1 1 0 0 1 1 0
  1 2 2 1 1 2 2 1
  1 1 3 1 1 3 1 1
  0 1 1 0 0 1 1 0
  ```

### 3.4 音声ファイル WAV（`assets/audio/*.wav`）
- **フォーマット**: 16bit PCM WAV 形式。
- **ロード方式**:
  - **効果音 (SE)**: ゲーム開始時に `SoundCatalog` へ全件ロードされ、メモリ上で再利用。
  - **BGM**: 各ステージの `StageIntro` で `BgmCache` にロードされ、無限ループ再生。ステージ終了時に解放。
- **音声IDマッピング**:
  | 音声ID | 用途 | 参照ファイル |
  |---|---|---|
  | `1` | 自弾発射音 | `assets/audio/fire.wav` |
  | `2` | 弾命中音 | `assets/audio/hit.wav` |
  | `3` | 敵撃破爆発音 | `assets/audio/destroy.wav` |
  | `4` | 被弾ダメージ音 | `assets/audio/damage.wav` |
  | BGM | ステージBGM | `assets/audio/stage01_bgm.wav` 等 |

### 3.5 入力仕様

入力は `InputState` に統一してゲームロジックへ渡します。

| 入力 | キーボード | コントローラ |
|---|---|---|
| 上下左右移動 | 矢印キー / `WASD` | 十字キー / 左スティック |
| 発射・開始 | `Space` / `Z` / `Enter` / `X` | Southボタン |
| ポーズ切替 | `P` | `Start` |
| タイトルへ戻る | `Escape` | - |

- 押下中の入力は `fire` として扱い、押しっぱなしの自弾は30フレーム間隔で連射する。
- 新規押下は `fire_trigger` として扱い、連射間隔を待たずに1グループを発射できる。
- キーリピートは新規トリガーとして扱わない。
- 左右・上下は各キーの押下状態を個別に保持し、片方を離しても残りの方向を維持する。
- コントローラ入力は `gilrs` で取得し、キーボード入力と合成する。

### 3.6 被弾時の回復ストック処理

- 被弾時はHPを減算し、成長レベルをいったん0へ戻す。
- 回復ストックはまずHP回復に使用し、1個につき25HP回復する。
- HPが満タンになった後、残りのストックを1個につき1レベルの成長回復へ使用する。
- 成長レベルの上限は4、回復ストックの上限は9。

---

## 4. セーブデータ・バイナリフォーマット

### 4.1 ランキングデータ（`ranking.bin`）
ハイスコアトップ10を保存・復元するための固定長バイナリフォーマットです。

- **総サイズ**: `75 bytes` 固定
- **エンディアン**: リトルエンディアン (LE)
- **ファイルレイアウト**:

| オフセット (byte) | サイズ (bytes) | 型 | 項目名 | 内容 |
|---|---|---|---|---|
| `0x00` | 4 | `[u8; 4]` | Magic Header | ASCII固定文字列 `"RSHK"` (`0x52, 0x53, 0x48, 0x4B`) |
| `0x04` | 1 | `u8` | Format Version | フォーマットバージョン番号 (`1`) |
| `0x05` | 3 | `[u8; 3]` | Rank 1 Name | 1位の名前（ASCII 3文字、空白パディング） |
| `0x08` | 4 | `u32` (LE) | Rank 1 Score | 1位の得点 |
| `0x0C` | 3 | `[u8; 3]` | Rank 2 Name | 2位の名前 |
| `0x0F` | 4 | `u32` (LE) | Rank 2 Score | 2位の得点 |
| ... | ... | ... | ... | ... |
| `0x44` | 3 | `[u8; 3]` | Rank 10 Name | 10位の名前 |
| `0x47` | 4 | `u32` (LE) | Rank 10 Score | 10位の得点 |

### 4.2 プラットフォーム別保存先パス
セーブデータは OS 標準のデータディレクトリに分離して保存されます。

- **Windows**: `%APPDATA%\RustShooting\ranking.bin`
  （例: `C:\Users\<ユーザー名>\AppData\Roaming\RustShooting\ranking.bin`）
- **Linux / その他**: `$XDG_DATA_HOME/rust_shooting/ranking.bin`
  （未設定時は `~/.local/share/rust_shooting/ranking.bin`）
- **フォールバック**: 上記が取得できない場合は `./data/ranking.bin`

---

## 5. UIおよび文字盤データ仕様

### 5.1 ネームエントリー文字盤（`CHAR_MATRIX`）
名前入力画面（`NameEntry`）で使用される 7列 × 6行 の選択用文字盤です。

```text
[ 行 0 ]  A    B    C    D    F    E    G
[ 行 1 ]  H    I    J    K    L    M    N
[ 行 2 ]  O    P    Q    R    S    T    U
[ 行 3 ]  V    W    X    Y    Z    .    -
[ 行 4 ]  0    1    2    3    4    5    6
[ 行 5 ]  7    8    9    <    >   DEL  SPC
```

- **キー種別（`GridKey`）**:
  - `Char(u8)`: 指定文字を入力
  - `Left` (`<`): 入力カーソルを左へ戻す
  - `Right` (`>`): 入力カーソルを右へ進める
  - `Delete` (`DEL`): 現在文字を削除して1文字戻る
  - `Space` (`SPC`): 空白文字を入力
- **入力制限**: 最大 3 文字。3文字入力確定時にランキングへ自動挿入・保存。

### 5.2 HUD 表示レイアウト
画面最下部の 1 行（高さ 16px、`y: 624..=639`）にゲームステータスを配置します。

| 領域 | X座標範囲 (px) | 幅 × 高 (px) | 表示内容 |
|---|---|---|---|
| **HPゲージ** | `x: 8..=128` | 120 × 10 | 残HP割合バー（緑 > 50%、黄 > 25%、赤 ≤ 25%） |
| **スコア** | `x: 140..=260` | 120 × 8 | 8桁ゼロ埋め得点表示（例: `00012340`） |
| **残機数** | `x: 280..=360` | 80 × 8 | 残自機数表示（例: `L:02`） |
| **ストック** | `x: 380..=470` | 90 × 8 | 成長Lv (`LV:0`〜`LV:4`) または 回復ストック (`ST:+0`〜`ST:+9`) |
