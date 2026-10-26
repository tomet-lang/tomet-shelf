# tomet-shelf UI Redesign Specification
## Concept: "Obsidian × Wikipedia"

---

## 1. Vision & Core Philosophy

`tomet-shelf` は Tomet 言語および Tomet Vault の公式標準ドキュメント・Wikiビューアです。  
これまでの「石碑・本型（Stela）」の固有デザインを `stela-shelf` へ完全に移管したことを受け、`tomet-shelf` は **現代的でニュートラル、かつ知的生産と情報探索に最適化された標準ビューア** として再設計します。

### インスピレーションの融合
- **Wikipedia（百科事典・ドキュメント性）**:
  - 圧倒的な可読性、明確な目次（TOC）、要約インフォボックス、脚注・リファレンス、リンク切れ（未解決リンク）の明瞭なアフォーダンス。
- **Obsidian / Quartz（PKM・ナレッジグラフ性）**:
  - ノート間の双方向リンク（バックリンク）、内部リンクのホバープレビュー、パンくずリスト、クイック検索パレット（`Ctrl+K`）、思考を妨げないペイン構成。
- **VitePress / mdBook（開発者・標準ツール性）**:
  - 高速、無駄のないフラット＆洗練されたサーフェス、洗練されたコードブロックとコピー機能。

---

## 2. Design Principles

1. **Content as the Hero（本文が主役）**
   - 具象的なテクスチャ（大理石・青海波などの装飾）を排し、コンテンツそのものが際立つニュートラルなキャンバスを提供します。
2. **Dense yet Breathable（凝縮と可読性の調和）**
   - 情報密度を保ちつつ、目の疲れないタイポグラフィ（適切なフォントサイズ、行長 720〜800px、行間 1.7〜1.8）を徹底します。
3. **Border-less by Default & Depth via Shadows（線ではなく影で描く階層と深さ）**
   - **基本的にボーダー（枠線）はつけない**: ボックスを線で囲い込む「枠線過多」を徹底排除し、余白とサーフェス（背景トーン）の差、そして**一貫した影（Shadow / Elevation）**によって階層と深さを表現します。
   - カード、コールアウト、ボタン、ペインの区切りに硬い線を用いず、柔らかな陰影とトーンで空間的な広がりと立体感を与えます。
4. **Connected Knowledge（つながる知識）**
   - リンク先をチラ見できる「カードプレビュー」、その場で展開できる「センターピーク」、そして「このページを参照しているノート（バックリンク）」を自然に発見できるようにします。
5. **Adaptive 3-Pane Architecture（柔軟な3ペイン構造）**
   - デスクトップでは「ナビ - 本文 - 目次/バックリンク」を一覧でき、画面幅に応じて美しく折りたたまれ、読書・執筆を邪魔しません。

---

## 3. Layout Architecture

```
+---------------------------------------------------------------------------------------+
|  Top App Bar: [Logo / Vault Name]  [Breadcrumbs]      [Search Ctrl+K]  [Theme] [GitHub] |
+-----------------------+-----------------------------------------------+---------------+
| Left Pane: Navigation | Center Pane: Article Canvas                   | Right Pane:   |
|                       |                                               | Context       |
| - Vault Tree / Index  | - Title & Metadata Chips (Kind, Date, Author) | - On This     |
| - Sections & Folders  | - Infobox / Hero Summary (if present)         |   Page (TOC)  |
| - Recent Notes Tab    | - Article Markdown/Tomet Content              | - Backlinks   |
|                       | - Footnotes & References                      |   (Incoming   |
|                       | - Previous / Next Page Links                  |    Links)     |
+-----------------------+-----------------------------------------------+---------------+
```

### (1) Top App Bar
- 高さ 56px のクリーンなバー。下線ボーダーは設けず、スクロール時に自然な微小な影（`--shadow-sm`）でコンテンツから浮上。
- 左側: サイトロゴ、Vault名、現在のパンくずリスト。
- 中央/右側:
  - 検索入力（`Ctrl+K` / `/` でモーダル起動、枠線なしのソフトサーフェス）。
  - ビューモード切り替え（Wiki / Classic）。
  - テーマ切り替え（Light / Dark / Auto）。
  - 外部ソースリンク（GitHubリポジトリ等）。

### (2) Left Pane: Navigation (幅: 260px〜300px)
- **フォルダ・ツリー表示**: Vault 内の階層を素直にインデント表示。アクティブなページを視覚的に強調（枠線なし、サーフェス色とアクセント）。
- **タブ切り替え / サブセクション**:
  - 「ドキュメントツリー」
  - 「最近開いたノート (Recent Notes)」
  - 「タグ / 索引一覧」
- 左下に折りたたみトグルボタン（ショートカット対応）。

### (3) Center Pane: Article Canvas (最大幅: 768px〜840px、中央揃え)
- **ヘッダー領域**:
  - 文書タイトル（`H1`、堂々としたセリフまたは洗練されたサンセリフ）。
  - メタデータバー: 種別バッジ（`::kind`）、更新日時、タグ、親階層。
  - Wikipedia風インフォボックス（必要な場合、右フロートまたは上部要約カード。枠線なし、`--shadow-sm` の浮遊カード）。
- **本文領域**:
  - 余計な枠線をなくし、美しいタイポグラフィと見出しアンカーリンク。
  - リンクの表現:
    - 解決済みリンク: 落ち着いたブルー系、下線はホバー時または繊細な点。
    - 未解決リンク: 赤色の破線（執筆者には気付きやすく、読書にはノイズにならない設計）。
  - コードブロック: 言語バッジ、コピーボタン、枠線なしでわずかな背景トーン（`--bg-subtle`）。
  - コールアウト / カード: 枠線ではなく、ソフトな背景トーンと影（`--shadow-sm`）で浮遊感と階層を表現。
- **フッター領域**:
  - 最終更新日時、前後のドキュメントナビゲーションカード（`--shadow-sm`）。

### (4) Right Pane: Context & TOC (幅: 240px〜280px)
- **目次 (On this page)**:
  - スクロール位置と連動したリアルタイム・アクティブハイライト。
  - ネストされた見出しのスムーズスクロール。
- **バックリンク (Backlinks / Linked Mentions)**:
  - このノートを参照している他のノート一覧と、リンク前後のコンテキスト抜粋（Obsidian 的な最重要機能）。
- **クイックアクション**:
  - ページの先頭に戻る、ソースを表示する（`source-viewer`）。

---

## 4. Semantic Color & Shadow (Elevation) System

ボーダーで囲むのではなく、**サーフェスの明度差** と **一貫した影（Shadows）** で階層を定義します。

### Elevation & Shadow Tokens
- `--shadow-none`: `none` (最下層キャンバス)
- `--shadow-sm`: `0 1px 3px 0 rgba(0, 0, 0, 0.06), 0 1px 2px -1px rgba(0, 0, 0, 0.04)` (カード、コールアウト、ボタン、ホバーチップ)
- `--shadow-md`: `0 4px 6px -1px rgba(0, 0, 0, 0.08), 0 2px 4px -2px rgba(0, 0, 0, 0.05)` (ドロップダウン、フローティングコントロール、ヘッダー浮遊)
- `--shadow-lg`: `0 10px 15px -3px rgba(0, 0, 0, 0.10), 0 4px 6px -4px rgba(0, 0, 0, 0.05)` (ホバープレビューカード、`Ctrl+K` 検索パレット)
- `--shadow-xl`: `0 20px 25px -5px rgba(0, 0, 0, 0.12), 0 8px 10px -6px rgba(0, 0, 0, 0.06)` (Center Peek モーダル、Lightbox)

*※ダークモード時は、アンビエントシャドウ（より濃い黒の拡散）＋サーフェス明度差で自然な奥行きを再現。*

### Light Theme
- `--bg-canvas`: `#fcfcfc` (最下層メイン背景)
- `--bg-surface`: `#ffffff` (浮遊する記事キャンバス・カード・ペイン)
- `--bg-subtle`: `#f3f4f6` (ホバー、入力欄、タグチップ背景)
- `--text-primary`: `#111827` (本文・見出し)
- `--text-secondary`: `#4b5563` (メタデータ、説明文)
- `--text-muted`: `#9ca3af` (プレースホルダー、アイコン)
- `--accent-primary`: `#2563eb` (標準ブルー。Wikipedia/Obsidian的安心感)
- `--link`: `#1d4ed8`
- `--link-unresolved`: `#dc2626` (破線アンダーライン)

### Dark Theme
- `--bg-canvas`: `#0d0f14` (最下層背景)
- `--bg-surface`: `#161922` (浮遊サーフェス・カード)
- `--bg-subtle`: `#1f2430` (ホバー、入力欄、タグチップ背景)
- `--text-primary`: `#f9fafb`
- `--text-secondary`: `#9ca3af`
- `--text-muted`: `#6b7280`
- `--accent-primary`: `#3b82f6`
- `--link`: `#60a5fa`
- `--link-unresolved`: `#ef4444`

---

## 5. Client Interactions & Components

既存の堅牢なスクリプト資産を新デザインに統合：
1. **ホバーカードプレビュー**:
   - リンクホバーから 300ms 後に、リンク先のタイトル＋要約文を浮動カードで表示（Wikipedia の Page Previews 相当）。
2. **Center Peek / Lightbox**:
   - `Shift + Click` または特定操作で、別ペインに移動せず中央モーダルでノート全容を即座に参照。
3. **コマンド / 検索パレット (`Ctrl+K`)**:
   - Pagefind の高速インデックスを活用した即時インクリメンタル検索。
4. **Web Components (`<tmt-*>`)**:
   - `<tmt-btn>`, `<tmt-badge>`, `<tmt-icon>`, `<tmt-swatch>`, `<tmt-switch>` のスタイルをフラット・モダン化。
