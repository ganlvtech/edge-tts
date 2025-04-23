use anyhow::{anyhow, Result};
use rand::RngCore;
use tungstenite::{Message, WebSocket};
use xml::escape::{escape_str_attribute, escape_str_pcdata};

const SYNTH_URL: &str = "wss://speech.platform.bing.com/consumer/speech/synthesize/readaloud/edge/v1?TrustedClientToken=6A5AA1D4EAFF4E9FB37E23D68491D6F4";

fn random_request_id() -> String {
    let mut buf = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut buf);
    hex::encode(&buf[..])
}

fn parse_headers(s: impl AsRef<str>) -> Vec<(String, String)> {
    s.as_ref().split("\r\n").filter_map(|s| {
        if s.len() > 0 {
            let mut iter = s.splitn(2, ":");
            let k = iter.next().unwrap_or("").to_owned();
            let v = iter.next().unwrap_or("").to_owned();
            Some((k, v))
        } else {
            None
        }
    }).collect()
}

/// `voice_short_name`: eg: "zh-CN-XiaoxiaoNeural"
///
/// `pitch`
/// * x-low
/// * low
/// * medium
/// * high
/// * x-high
/// * default
///
/// `rate`
/// * x-slow
/// * slow
/// * medium
/// * fast
/// * x-fast
/// * default
///
/// `volume`
/// * silent
/// * x-soft
/// * soft
/// * medium
/// * loud
/// * x-loud
/// * default
pub fn build_ssml(text: &str, voice_short_name: &str, pitch: &str, rate: &str, volume: &str) -> String {
    format!("<speak version=\"1.0\" xmlns=\"http://www.w3.org/2001/10/synthesis\" xmlns:mstts=\"https://www.w3.org/2001/mstts\" xml:lang=\"en-US\"><voice name=\"{}\"><prosody pitch=\"{}\" rate=\"{}\" volume=\"{}\">{}</prosody></voice></speak>", escape_str_attribute(voice_short_name), escape_str_attribute(pitch), escape_str_attribute(rate), escape_str_attribute(volume), escape_str_pcdata(text))
}

/// `output_format`: eg: "audio-24khz-48kbitrate-mono-mp3". See https://learn.microsoft.com/en-us/azure/ai-services/speech-service/rest-text-to-speech?tabs=streaming#audio-outputs
pub fn request_audio(ssml: &str, output_format: &str) -> anyhow::Result<Vec<u8>> {
    let (mut socket, _) = tungstenite::connect(SYNTH_URL)?;
    process_socket_data(&ssml, &output_format, &mut socket)
}

/// `output_format`: eg: "audio-24khz-48kbitrate-mono-mp3". See https://learn.microsoft.com/en-us/azure/ai-services/speech-service/rest-text-to-speech?tabs=streaming#audio-outputs
/// `proxy_addr`: socks5 proxy addr，like "127.0.0.1:1080"
pub fn request_audio_via_socks5_proxy(ssml: &str, output_format: &str, proxy_addr: &str) -> anyhow::Result<Vec<u8>> {
    let url = url::Url::parse(SYNTH_URL)?;
    let host = url.host_str().unwrap();
    let port = url.port_or_known_default().unwrap();

    let proxy_stream = socks::Socks5Stream::connect(proxy_addr, (host, port))?;
    let tls_connector = native_tls::TlsConnector::new()?;
    let tls_stream = tls_connector.connect(host, proxy_stream)?;
    let (mut socket, _) = tungstenite::client::client(&url, tls_stream)?;
    process_socket_data(&ssml, &output_format, &mut socket)
}

fn process_socket_data<S: std::io::Read + std::io::Write>(
    ssml: &str,
    output_format: &str,
    socket: &mut WebSocket<S>,
) -> Result<Vec<u8>> {
    socket.send(Message::Text(format!("Content-Type:application/json; charset=utf-8\r\nPath:speech.config\r\n\r\n{{\"context\":{{\"synthesis\":{{\"audio\":{{\"metadataoptions\":{{\"sentenceBoundaryEnabled\":false,\"wordBoundaryEnabled\":true}},\"outputFormat\":\"{}\"}}}}}}}}", output_format)))?;
    let request_id = random_request_id();
    socket.send(Message::Text(format!("X-RequestId:{}\r\nContent-Type:application/ssml+xml\r\nPath:ssml\r\n\r\n{}", request_id, ssml)))?;
    let mut buf = Vec::new();
    loop {
        match socket.read() {
            Ok(msg) => {
                match msg {
                    Message::Text(s) => {
                        if let Some(header_str) = s.splitn(2, "\r\n\r\n").next() {
                            let headers = parse_headers(header_str);
                            if headers.iter().any(|(k, v)| k == "Path" && v == "turn.end") {
                                if headers.iter().any(|(k, v)| k == "X-RequestId" && v.as_str() == request_id) {
                                    return Ok(buf);
                                } else {
                                    return Err(anyhow!("Path:turn.end no X-RequestId header"));
                                }
                            }
                        } else {
                            return Err(anyhow!("bad text response. message not complete"));
                        }
                    }
                    Message::Binary(s) => {
                        let header_len = s[0] as usize * 256 + s[1] as usize;
                        if s.len() >= header_len + 2 {
                            let headers = parse_headers(String::from_utf8_lossy(&s[2..header_len]));
                            let body = &s[(header_len + 2)..];
                            if headers.iter().any(|(k, v)| k == "Path" && v == "audio") {
                                if headers.iter().any(|(k, v)| k == "X-RequestId" && v.as_str() == request_id) {
                                    buf.extend(body);
                                } else {
                                    return Err(anyhow!("Path:audio no X-RequestId header"));
                                }
                            }
                        } else {
                            return Err(anyhow!("bad binary response. response len: {} header len: {}", s.len(), header_len));
                        }
                    }
                    _ => {}
                };
            }
            Err(e) => {
                return Err(anyhow!("socket read error: {:?}", e));
            }
        };
    }
}
