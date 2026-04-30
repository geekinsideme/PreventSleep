# PreventSleep

Windows PC のスリープ・画面消灯を防止し、設定ファイルに従ってウィンドウを自動再配置するユーティリティです。

## バージョン

2.1.4 (Rust 実装)

## 機能

- **スリープ・画面消灯防止** — Windows API でディスプレイの電源オフを抑止し、30秒ごとにマウス微小移動を送信
- **ウィンドウ自動再配置** — `PreventSleep.txt` の設定に従い、起動時・モニター構成変化時にウィンドウを配置
- **マルチモニター対応** — モニター数変化を検知して自動的に再配置を実行
- **モニター電源ON検知** — モニターが復帰した際に 2 秒後に再配置を実行
- **常時最前面** — アプリウィンドウは常に最前面に表示
- **CLI モード** — コマンドライン引数による非 GUI 操作

## ビルド

### 必要な環境

```
rustup (https://rustup.rs)
Visual Studio Build Tools 2019 以降 (MSVC ツールチェーン)
```

### セットアップ

```powershell
# Rust インストール (未インストールの場合)
winget install Rustlang.Rustup

# MSVC ターゲットを追加 (通常は自動)
rustup target add x86_64-pc-windows-msvc
```

### ビルド

```powershell
cd c:\path\to\PreventSleep
cargo build --release
```

生成物: `target\release\PreventSleep.exe`

## 使い方

### GUI モード

```
PreventSleep.exe
PreventSleep.exe noprevent   # スリープ防止を無効にして起動
```

| ボタン | 動作 |
|---|---|
| Set Location | PreventSleep.txt に従ってウィンドウを再配置 |
| List Windows | 現在の可視ウィンドウ一覧をログに表示 |
| 1-Display | 1画面構成として再配置 |
| X | スリープ防止を解除してモニターをオフにする |

### CLI モード

```
PreventSleep.exe set         # ウィンドウを再配置して終了
PreventSleep.exe monitoroff  # モニターをオフにして終了
```

## 設定ファイル (`PreventSleep.txt`)

アプリケーションと同じディレクトリに置く CSV ファイル。

### 書式

```
<タイトル正規表現>,<クラス名正規表現>,<X>,<Y>,<幅>,<高さ>[,<画面数]
```

| フィールド | 説明 |
|---|---|
| タイトル正規表現 | ウィンドウタイトルのマッチ正規表現。空欄は任意 |
| クラス名正規表現 | ウィンドウクラス名のマッチ正規表現。空欄は任意 |
| X / Y | 配置先の左上座標 (ピクセル) |
| 幅 / 高さ | ウィンドウサイズ (ピクセル) |
| 画面数 | 有効にするモニター接続数 (例: `12` → 1台・2台接続時のみ有効)。省略すると `12345` |

### 記述例

```
# コメント行 (# で始まる行は無視)
####,####,0,0,0,0          # 区切り行 (#### で始まる行も無視)

Notepad,,100,100,800,600   # タイトルに "Notepad" を含むウィンドウを (100,100) に配置
,CabinetWClass,0,0,1280,720,12  # クラス名が CabinetWClass のウィンドウ (1〜2画面時)
```

## GitHub Releases

タグ `v*` を push すると GitHub Actions が自動的にビルドし、Release に `PreventSleep.exe` を添付します。

```powershell
git tag v2.1.4
git push origin v2.1.4
```

## ライセンス

MIT
