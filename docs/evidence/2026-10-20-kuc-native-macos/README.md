# macOS KUC Native IME Observation

## 実行

公開KUC v0.4.2のmacOS ARM64 producerを使用した。KUC/KatanAの編集はない。
実行時刻は2026-10-20 12:45-12:46 JST。KLEの合格artifactではない。

- 公開版: https://github.com/HiroyukiFuruno/katana-ui-core/releases/tag/v0.4.2
- revision: `2a891ac60cd421216ee648d60148356490d599b0`
- ZIP SHA-256: `ea8b023397ec2d631825968023b9284ac95965d7e844473abca39a461c4dd3cf`
- producer SHA-256: `7afb03231cf9558de2bb371bb861fb5bcddeae3496ce6e99d7521315f2fc0890`
- verifier SHA-256: `0ad0dde2735d5998cef17a2d41180673f67434109bdf52a1dd2b98599556f35b`

ZIPの公開metadata、実bytes、manifestのbinary digestを照合した。起動済みprocessを
Computer Useで操作するため、同一binaryをローカルapp bundleに配置した。
描画・IME・検査をKLEに複製していない。

```text
--revision=2a891ac60cd421216ee648d60148356490d599b0
--challenge=kle-20261020-019f2593-native-macos-02
--producer-id=kuc-public-0.4.2-macos-arm64
--runner-id=kle-local-macos-arm64
--timeout-ms=180000
```

OSの日本語IMEでキー`n i h o n g o`、Space、Returnを送信した。
未確定の`に`、`にほんご`、変換後の`日本語`を画面とAccessKitで確認した。
paste、Textイベントの直接注入、文書stateの書換えは使用していない。
producerはpreedit sequence 8、commit sequence 9と入力method
`com.google.inputmethod.Japanese.base`を記録し、exit 0で終了した。

process stdoutの別経路の受領値は次のとおり。

```json
{"artifact_sha256":"13f62efb3d7909e6cb6ff2b73c2db92c34393efd0da5fc6afae1d26becb93cca","run_id":"native-34433-1792467942051075000"}
```

元の起動引数とこの受領値からローカルexpectationsを作り、公開verifierがexit 0と
`native IME evidence verified against supplied trusted expectations`を返した。
artifactからdigestを読み戻して期待値にする自己検査は行っていない。ただし、この
ローカルprocess受領は公開されたrunner外部attestationではないため、他者がこの
READMEだけをtrust anchorとして公開受入に使うことはできない。

## 観測と制約

同梱`native-ime.json`はproducerの元bytesであり、後から数値を補完していない。
`⭐️`のscalar列は`[11088, 65039]`、色cropは19 x 26、
`has_colored_pixels=true`。`☆`と異なるcrop hash、同一frameのcaret、hit test、
`MultilineTextInput`の値・boundsを保持する。

このrunで実際に入力・確定したのは`日本語`である。`⭐️`と`☆`はproducerの
初期文書に含まれる比較対象であり、今回`⭐️`を入力した証明ではない。
Windows/Linux実入力、KLE root、文書編集・検索・置換・undo、KatanA host effect、
source leafとの同一run結合、公開attestationの完了とは扱わない。
