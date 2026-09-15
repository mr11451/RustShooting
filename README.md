# RustShooting

縦画面型の2Dシューティングゲームです。論理画面は `480x640`、ゲーム更新は固定 `60fps` です。

## 必要環境

- Windows MSVC環境
- Rust stable（検証済み: `1.98.0`）
- Cargo

GPU、音声出力、ゲームパッドを使用します。ゲームパッド入力は `gilrs` で取得します。

## ビルドと実行

PowerShellでリポジトリのルートを開いて実行します。

```powershell
cargo build
cargo run
```

リリースビルド:

```powershell
cargo build --release
cargo run --release
```

品質確認:

```powershell
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

実行には作業ディレクトリをリポジトリルートにしてください。`assets/` と `data/` は相対パスで読み込まれます。

## 操作

| 操作 | キーボード | コントローラ |
|---|---|---|
| 移動 | 矢印キー / `WASD` | 十字キー / 左スティック |
| 発射・開始 | `Space` / `Z` / `Enter` / `X` | Southボタン |
| ポーズ | `P` | Start |
| タイトルへ戻る | `Esc` | - |

押しっぱなしの発射は0.5秒間隔です。新規押下トリガーは間隔を待たずに発射できます。

## データの正本

現在のランタイムは、Rustの静的テーブルを正本として使用します。

- `src/data/mod.rs`: ステージ、敵、ボス、弾、軌道、発射パターン、成長特性
- `src/game/world.rs`: 実行時の移動、生成、衝突、発射処理
- `src/runtime/mod.rs`: 固定長オブジェクトプールと実行時状態

`data/*.toml` は調整・設計用のデータテンプレートです。TOMLを変更しただけでは現在の実行バイナリへ自動反映されないため、ランタイムへ反映する場合は対応する `src/data/mod.rs` の静的テーブルも更新してください。

## データ調整

### 敵・ボス

キャラクター特性は `src/data/mod.rs` の `CHARACTER_TRAITS` を変更します。

```rust
CharacterTrait {
    character_id: 105,
    character_type: CharacterType::Boss,
    max_hp: 3_000,
    contact_damage: 60,
    score: 50_000,
    default_orbit_id: 10,
    default_fire_pattern_id: 15,
    ..
}
```

主な項目:

- `character_id`: スケジュールや描画で参照するID
- `hitbox_width` / `hitbox_height`: Q12.4の当たり判定サイズ
- `max_hp`: 敵・ボスのHP
- `contact_damage` / `player_damage`: 接触・弾による自機ダメージ
- `score`: 撃破得点
- `default_orbit_id`: 初期軌道ID
- `default_fire_pattern_id`: 発射パターンID

実際の出現位置とタイミングは `STAGE_SCHEDULES` で管理します。ボスは各ステージの最後のスケジュール行を追加・変更します。

### 軌道

軌道は `ORBIT_DATA` の `OrbitData` を追加します。

- `Straight`: `velocity_x` / `velocity_y` による直線移動
- `Bezier`: `control_point_1`、`control_point_2`、`control_point_end`による曲線移動
- `Circle`: `radius` と `speed`から円運動。`next_orbit_id`を自分自身にすると無限ループ
- `MoveToPosition`: `target_position`へ移動
- `next_orbit_id`: 軌道終了後に移る軌道。`0`は終了、別IDは任意の軌道へ遷移

ボスの仮シーケンスは、軌道 `8`で定位置へ移動し、軌道 `9`で円運動を無限ループします。ステージ6ボスは専用軌道 `10`を使用します。

軌道を追加したら、対応する`ScheduleData.orbit_id`も変更してください。円軌道へ遷移する場合、直前軌道の終了位置と円周開始位置が一致するよう中心・半径を設定します。

### 敵弾

敵弾の発射パターンは `FIRE_PATTERNS` の `FirePatternData` で管理します。

- `fire_frame`: 敵の軌道開始からの発射フレーム
- `spawn_offset_x` / `spawn_offset_y`: 敵からの発射位置
- `bullet_character_id`: 使用する弾特性ID
- `direction`: 32方向の発射方向
- `speed`: Q12.4の速度
- `next` のような暗黙の補正はなく、データに設定した速度がそのまま使用されます

敵弾速度の1.25倍は発射コードで掛けず、`FIRE_PATTERNS` の `speed` と速度成分をあらかじめ1.25倍の値に設定します。

### 弾特性

弾の当たり判定・威力・貫通は `BULLET_CHARACTER_DATA` の `BulletCharacterData` で調整します。

- `bullet_character_id`: 弾ID
- `hitbox_width` / `hitbox_height`: 当たり判定サイズ
- `damage`: 敵へ与えるダメージ
- `penetrating`: 貫通フラグ
- `player_damage`: 自機へ与えるダメージ

画像の対応は `assets/bullets/`、弾の用途とフレームは `data/bullets.toml` を参照します。

### 自弾の成長特性

成長レベルごとの自弾設定は `PLAYER_GROWTH_DATA` の `PlayerGrowthData` で管理します。

- `max_bullet_groups`: 同時保持できるグループ数
- `bullets_per_group`: 1グループの横並び弾数
- `bullet_character_id`: 自弾特性ID
- `speed`: Q12.4の自弾速度
- `directions`: グループ内各弾の32方向

現在の仮データは、グループ数 `4/6/8/10/12`、弾数 `1/2/3/4/5` です。発射方向を変更する場合は、`PLAYER_DIRECTIONS_LEVEL_0` から`LEVEL_4`の配列を変更します。

### ステージ背景

- `data/backgrounds.toml`: 背景アトラスとスクロール速度のテンプレート
- `data/stage01_tilemap.txt`: タイル配置
- `assets/backgrounds/*.png`: 64x64の4x4タイルアトラス

背景速度はQ12.4で管理し、ステージスケジュールの`background_speed`で変更できます。

## アセット形式

詳細は [assets/README.md](assets/README.md) と [doc/data_formats.md](doc/data_formats.md) を参照してください。

- キャラクター・ボス・アイテム: 32x32タイルのGIF
- ボス画像: 64x64タイルを使用する実装箇所あり
- 敵弾: `enemy_bullets.gif` の8x8タイル2パターン
- 背景: 64x64 RGBA PNG
- 音声: WAV

## ドキュメント

- [doc/program_design.md](doc/program_design.md): プログラム設計
- [doc/data_formats.md](doc/data_formats.md): データ形式
- [doc/plan_review.md](doc/plan_review.md): 仕様レビュー
- [doc/implementation_plan.md](doc/implementation_plan.md): 実装計画
- [doc/stage_data_template.md](doc/stage_data_template.md): ステージデータテンプレート
