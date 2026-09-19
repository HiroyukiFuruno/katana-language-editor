# Scenario session同期後に検索Closeが維持されない

## 再現経路

公開 `katana-ui-core 0.3.6` をKLEの公開 `EguiTextCommandSurfaceEditor` 経由で
利用する。Storybookは `FullTextCommandSurfaceScenarioSession` をproviderに保持し、
構築時にretain、各frameでsynchronize leaseを発行する。KLEによるtext/search状態の
書換えや独自入力列はなく、検索traceはKUC公開continuationを使う。

```sh
cargo test -p kle-storybook --locked \
  storybook_motion_artifact_uses_the_exact_kuc_plan_without_idle_frames \
  -- --test-threads=1
```

検索close後の次frame検査で `Continuation(Search(CloseNotApplied))` になる。
初期leaseだけでproviderを破棄する旧KLE経路はこの不整合を露出しなかったが、
更新projectionも受け取らないため、そこへ戻す解決はしない。

## 公開ソースの対応箇所

- `src/egui/text_command_surface/scenario_session.rs` の `synchronize_lease` は
  session stateからpresentationを作り、次revisionでleaseを発行する。
- 同ファイルの `ScenarioSessionUpdate::from_context` はsearch eventの
  `CommandChromeSearchEvent::Strip`以外をcontinueし、`CloseRequested`を保持しない。
  query/valueは保存するが、close可視状態はstate/update型にも存在しない。
- `src/egui/text_command_surface/synchronization.rs` の `synchronize_presentation` は
  `search_closed_by_interaction`をfalseへ戻し、sessionの `Some(search)`を同期する。
- `src/egui/text_command_surface/interaction_locator/continuations.rs` は
  Close -> VerifyClosedの次frameでsearch_visibleがtrueだと `CloseNotApplied` を返す。

実測失敗とこのコード経路は対応している。ただし、全search操作や三OSでの影響を
検証済みとはしない。generic rootの明示的再openとsessionの暗黙復元を区別して調査する。

## KUC側の対応条件

- [ ] KUC-owned scenario sessionでclose結果を次の同期projectionへ反映する。
  KLEにvisible flag、query解析、子UI setter、callbackの追加状態管理を要求しない。
- [ ] retain一回・毎frame synchronizeの実KUC経路で、query入力、next/previous、close、
  次frameの非表示、明示的再openを連続検証する。receipt/AX/実frameを同じ実行で照合する。
- [ ] replace modeやoption変更等、sessionが現在保存しないgeneric状態にも影響調査を行う。
- [ ] macOS/Windows/Linuxの実入力検証と、既存全ゲート・coverageを維持する。
- [ ] 直接/推移的依存・lockfileの更新調査を同梱し、修正版をregistryへ公開する。

KUC #40のconsumer-defined artifact planとKUC #43のHiDPI不整合は別要件として残る。
KLEからKUC/registryキャッシュを編集せず、検索終了の回帰をskip又は成功扱いにしない。
