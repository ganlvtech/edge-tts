# Rust Edge TTS

A simple Azure Speech Service module that uses the Microsoft Edge Read Aloud API.

Inspired by https://github.com/rany2/edge-tts and https://github.com/Migushthe2nd/MsEdgeTTS

https://learn.microsoft.com/en-us/azure/ai-services/speech-service/speech-synthesis-markup

## Example

```rust
use std::fs::OpenOptions;
use std::io::Write;
use edge_tts::{build_ssml, request_audio};

fn main() {
    OpenOptions::new().create(true).truncate(true).write(true).open("test.mp3").unwrap()
        .write(&request_audio(&build_ssml("晚上好，欢迎进入直播间。", "zh-CN-XiaoxiaoNeural", "medium", "medium", "medium"), "audio-24khz-48kbitrate-mono-mp3").unwrap()).unwrap();
}
```

## LICENSE

MIT License
