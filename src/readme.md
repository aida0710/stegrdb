# ソース構成

パケットの解析は`packet/`、送受信の進行と停止は`engine/`、Linuxのraw socketは`network/`が担当する。

中継の共通APIは`../crates/relay/`、具体的な実装は`../plugins/`にある。新しい中継方式を登録する場所は`plugins.rs`。

設定と実行方法は[README](../readme.md)、プラグインの契約は[中継プラグインの設計](../docs/relay-plugins.md)を参照する。
