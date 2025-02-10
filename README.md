# XInputDJ

このソフトウェアを使用すると、XboxコントローラーなどXInput対応のコントローラーを使用してDJソフトウェアであるMixxxを操作することができるようになる予定です。  
Steam DeckやROG AllyなどのゲーミングポータブルPCでの使用を想定しています。  
ただし、現時点ではWindowsのみ対応とする予定です。  

## Develop

```
npm install
npm run tauri dev
```

loopMIDIなどの仮想MIDIデバイスを使用してください。

## Mapping

まだ実装していませんが、下記のようになる予定です。  
ノブの回転に特化し、PLAYやCUE、SYNCなどのボタンはタッチパネルで押下してもらう想定です。  

```
左スティック: デッキ1のノブを回す
十字キー下: 左スティックの機能をイコライザーLoに設定
十字キー左: 左スティックの機能をフィルターに設定
十字キー上: 左スティックの機能をイコライザーHiに設定
十字キー右: 左スティックの機能をトリムに設定
L: 左スティックの機能をチャンネルフェーダーに設定
LT: 左スティックの機能をテンポに設定
L3: 左スティックの機能をジョグに設定

右スティック: デッキ2のノブを回す
A: 右スティックの機能をイコライザーLoに設定
B: 右スティックの機能をフィルターに設定
Y: 右スティックの機能をイコライザーHiに設定
X: 右スティックの機能をトリムに設定
R: 右スティックの機能をチャンネルフェーダーに設定
RT: 右スティックの機能をテンポに設定
R3: 右スティックの機能をジョグに設定
```
