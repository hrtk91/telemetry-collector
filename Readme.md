# Telemetry-colelctor

## 概要

Raspberry Pi 3 Model B と Zero W 　向けに作成したﾃﾚﾒﾄﾘｺﾚｸﾀｰ  
現状は CPU 使用率と使用済みメモリ量を rocket で/metrics から取得可能です
出力は opentelemetry-prometheus で出力されます

## 使い方

1. `build.ps1`を実行することでビルド用 docker イメージを作成し、ビルド結果が armv6,armv7 ディレクトリに出力されます

- armv6: Raspberry Pi Zero W 向け
- armv7: Raspberry Pi 3 Model B 向け

2. `.env`ファイルを作成し、以下の環境変数を設定します

```env
ROCKET_ADDRESS=0.0.0.0
ROCKET_PORT=8000
METER_NAME={任意}
CORE_NAME=./collector_core
```

3. `./collector-webapi`の実行で`/metrics`にアクセスすることで、CPU 使用率と使用済みメモリ量を取得できます

## 覚え書き

raspberrypi にコピーするとき
`scp -r ./armv6 {username}@{ipaddress}:/home/{username}/`
