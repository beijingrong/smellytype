//! Doubao streaming ASR 2.0. Cumulative hypotheses are previewed, then the
//! final full transcript is committed once after end-of-audio.
//! API: https://docs.volcengine.com/docs/6561/2630027
use super::{StreamHandle, StreamingEvent, StreamingTranscriber, Transcriber};
use crate::{config::DoubaoConfig, error::TranscribeError};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, http::HeaderValue, Message};

const ENDPOINT: &str = "wss://openspeech.bytedance.com/api/v3/sauc/bigmodel_async";
const CHUNK_SAMPLES: usize = 3200; // 200 ms at 16 kHz, mono
const MAX_PAYLOAD: usize = 1024 * 1024;
const IO_TIMEOUT: Duration = Duration::from_secs(10);

type Result<T> = std::result::Result<T, TranscribeError>;
fn failure(message: impl std::fmt::Display) -> TranscribeError {
    TranscribeError::InferenceFailed(format!("Doubao: {message}"))
}

#[derive(Debug, Clone)]
pub struct DoubaoTranscriber {
    config: DoubaoConfig,
}

impl DoubaoTranscriber {
    pub fn new(mut config: DoubaoConfig) -> Result<Self> {
        if config.api_key.is_none() {
            config.api_key = std::env::var("DOUBAO_API_KEY").ok();
        }
        let key = config.api_key.as_deref().unwrap_or("");
        if key.trim().is_empty() {
            return Err(TranscribeError::InitFailed(
                "Doubao requires DOUBAO_API_KEY or [doubao] api_key".into(),
            ));
        }
        if HeaderValue::from_str(key).is_err() {
            return Err(TranscribeError::InitFailed(
                "Invalid Doubao API key header".into(),
            ));
        }
        if !matches!(
            config.resource_id.as_str(),
            "volc.seedasr.sauc.duration" | "volc.seedasr.sauc.concurrent"
        ) {
            return Err(TranscribeError::InitFailed(
                "Doubao resource_id must be volc.seedasr.sauc.duration or volc.seedasr.sauc.concurrent (ASR 2.0)".into(),
            ));
        }
        if !(1..=300).contains(&config.final_timeout_secs) {
            return Err(TranscribeError::InitFailed(
                "Doubao final_timeout_secs must be between 1 and 300".into(),
            ));
        }
        Ok(Self { config })
    }

    fn start_at(&self, endpoint: String, samples_rx: mpsc::Receiver<Vec<f32>>) -> StreamHandle {
        let (events_tx, events) = mpsc::channel(64);
        let (cancel, cancel_rx) = oneshot::channel();
        let config = self.config.clone();
        let task = tokio::spawn(async move {
            tokio::select! {
                biased;
                _ = cancel_rx => {},
                result = run_session(&endpoint, &config, samples_rx, &events_tx) => {
                    if let Err(err) = result {
                        let _ = events_tx.send(StreamingEvent::Error(err)).await;
                    }
                }
            }
            let _ = events_tx.send(StreamingEvent::Ended).await;
            Ok(())
        });
        StreamHandle {
            events,
            cancel,
            task,
        }
    }
}

impl Transcriber for DoubaoTranscriber {
    fn transcribe(&self, samples: &[f32]) -> Result<String> {
        // CLI/file transcription uses the same protocol, paced at real time.
        let run = async {
            let (tx, rx) = mpsc::channel(8);
            let mut handle = self.start_at(ENDPOINT.into(), rx);
            let send = async {
                for chunk in samples.chunks(CHUNK_SAMPLES) {
                    if tx.send(chunk.to_vec()).await.is_err() {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
                drop(tx);
            };
            let receive = async {
                let mut result = None;
                while let Some(event) = handle.events.recv().await {
                    match event {
                        StreamingEvent::Final { text, .. } => result = Some(Ok(text)),
                        StreamingEvent::Error(err) => result = Some(Err(err)),
                        _ => {}
                    }
                }
                result.unwrap_or_else(|| Err(failure("stream ended without a final result")))
            };
            let (_, result) = tokio::join!(send, receive);
            let _ = handle.task.await;
            result
        };
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => tokio::task::block_in_place(|| handle.block_on(run)),
            Err(_) => tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(failure)?
                .block_on(run),
        }
    }

    fn as_streaming(&self) -> Option<&dyn StreamingTranscriber> {
        Some(self)
    }
}

impl StreamingTranscriber for DoubaoTranscriber {
    fn start_stream(&self, samples_rx: mpsc::Receiver<Vec<f32>>) -> Result<StreamHandle> {
        Ok(self.start_at(ENDPOINT.into(), samples_rx))
    }
}

fn init_payload() -> Value {
    json!({
        "user": {"uid": "smellytype"},
        "audio": {"format": "pcm", "codec": "raw", "rate": 16000, "bits": 16, "channel": 1},
        "request": {"model_name": "bigmodel", "enable_nonstream": true,
            "enable_itn": true, "enable_punc": true, "show_utterances": true,
            "result_type": "full"}
    })
}

fn encode(kind: u8, sequence: i32, data: &[u8]) -> Result<Vec<u8>> {
    let mut gzip = GzEncoder::new(Vec::new(), Compression::fast());
    gzip.write_all(data).map_err(failure)?;
    let payload = gzip.finish().map_err(failure)?;
    let flags = if sequence < 0 { 3 } else { 1 };
    let serialization = if kind == 1 { 1 } else { 0 };
    let mut out = vec![0x11, (kind << 4) | flags, (serialization << 4) | 1, 0];
    out.extend(sequence.to_be_bytes());
    out.extend((payload.len() as u32).to_be_bytes());
    out.extend(payload);
    Ok(out)
}

fn pcm(samples: &[f32]) -> Vec<u8> {
    samples
        .iter()
        .flat_map(|&s| {
            let s = if s.is_finite() {
                s.clamp(-1.0, 1.0)
            } else {
                0.0
            };
            ((s * 32768.0).round().clamp(-32768.0, 32767.0) as i16).to_le_bytes()
        })
        .collect()
}

#[derive(Debug, PartialEq)]
struct Response {
    text: Option<String>,
    last: bool,
}

fn read_u32(bytes: &[u8], offset: &mut usize) -> Result<u32> {
    let value = bytes
        .get(*offset..*offset + 4)
        .ok_or_else(|| failure("truncated frame"))?;
    *offset += 4;
    Ok(u32::from_be_bytes(value.try_into().unwrap()))
}

fn decode(bytes: &[u8]) -> Result<Response> {
    if bytes.len() < 4 || bytes[0] >> 4 != 1 || bytes[0] & 15 == 0 {
        return Err(failure("invalid protocol header"));
    }
    let kind = bytes[1] >> 4;
    let flags = bytes[1] & 15;
    let mut offset = usize::from(bytes[0] & 15) * 4;
    let mut last = flags & 2 != 0;
    if kind == 9 {
        if flags & 1 != 0 {
            last |= (read_u32(bytes, &mut offset)? as i32) < 0;
        }
        if flags & 4 != 0 {
            read_u32(bytes, &mut offset)?;
        }
    } else if kind == 15 {
        let code = read_u32(bytes, &mut offset)?;
        // Do not echo untrusted response bodies (which can include credentials).
        return Err(failure(format!(
            "server error {code}; check ASR 2.0 activation, quota and API key"
        )));
    } else {
        return Err(failure("unexpected server message type"));
    }
    let len = read_u32(bytes, &mut offset)? as usize;
    if len > MAX_PAYLOAD || bytes.len().checked_sub(offset) != Some(len) {
        return Err(failure("invalid payload length"));
    }
    let payload = &bytes[offset..];
    let mut decompressed = Vec::new();
    let payload = match bytes[2] & 15 {
        0 => payload,
        1 => {
            GzDecoder::new(payload)
                .take((MAX_PAYLOAD + 1) as u64)
                .read_to_end(&mut decompressed)
                .map_err(failure)?;
            if decompressed.len() > MAX_PAYLOAD {
                return Err(failure("response too large"));
            }
            &decompressed
        }
        _ => return Err(failure("unsupported compression")),
    };
    if bytes[2] >> 4 != 1 {
        return Err(failure("expected JSON response"));
    }
    let value: Value =
        serde_json::from_slice(payload).map_err(|_| failure("invalid response JSON"))?;
    if let Some(code) = value.get("code").and_then(Value::as_i64) {
        if code != 0 && code != 20000000 {
            return Err(failure(format!("server error {code}")));
        }
    }
    Ok(Response {
        text: value
            .pointer("/result/text")
            .and_then(Value::as_str)
            .map(str::to_owned),
        last,
    })
}

async fn run_session(
    endpoint: &str,
    config: &DoubaoConfig,
    mut samples_rx: mpsc::Receiver<Vec<f32>>,
    events: &mpsc::Sender<StreamingEvent>,
) -> Result<()> {
    let mut request = endpoint.into_client_request().map_err(failure)?;
    let headers = request.headers_mut();
    let mut key = HeaderValue::from_str(config.api_key.as_deref().unwrap_or(""))
        .map_err(|_| failure("invalid API key header"))?;
    key.set_sensitive(true);
    headers.insert("X-Api-Key", key);
    headers.insert(
        "X-Api-Resource-Id",
        HeaderValue::from_str(&config.resource_id)
            .map_err(|_| failure("invalid resource header"))?,
    );
    headers.insert(
        "X-Api-Request-Id",
        HeaderValue::from_str(&uuid::Uuid::new_v4().to_string()).unwrap(),
    );
    let (mut ws, _) = tokio::time::timeout(IO_TIMEOUT, tokio_tungstenite::connect_async(request))
        .await
        .map_err(|_| failure("connection timeout"))?
        .map_err(|err| match err {
            tokio_tungstenite::tungstenite::Error::Http(response) => failure(format!(
                "HTTP {}; check API key, ASR 2.0 activation and quota",
                response.status()
            )),
            _ => failure("WebSocket connection failed; check network access"),
        })?;
    tokio::time::timeout(
        IO_TIMEOUT,
        ws.send(Message::Binary(encode(
            1,
            1,
            &serde_json::to_vec(&init_payload()).map_err(failure)?,
        )?)),
    )
    .await
    .map_err(|_| failure("initial request timeout"))?
    .map_err(|_| failure("initial request failed"))?;
    let mut sequence = 1i32;
    let mut pending = Vec::new();
    let mut eof = false;
    let mut deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    let mut transcript = String::new();
    let mut ended_at = None;
    loop {
        tokio::select! {
            _ = tokio::time::sleep_until(deadline), if eof => return Err(failure(format!("timed out waiting for final result after {}s; increase doubao.final_timeout_secs if second-pass recognition is slow", config.final_timeout_secs))),
            chunk = samples_rx.recv(), if !eof => {
                match chunk {
                    Some(samples) => {
                        pending.extend(samples);
                        // Keep the final chunk for the negative-sequence EOS packet;
                        // exact multiples must not end with an empty audio payload.
                        while pending.len() > CHUNK_SAMPLES {
                            sequence += 1;
                            let frame = encode(2, sequence, &pcm(&pending[..CHUNK_SAMPLES]))?;
                            pending.drain(..CHUNK_SAMPLES);
                            tokio::time::timeout(IO_TIMEOUT, ws.send(Message::Binary(frame))).await
                                .map_err(|_| failure("audio upload timeout"))?.map_err(|_| failure("audio upload failed"))?;
                        }
                    }
                    None => {
                        eof = true;
                        sequence += 1;
                        let frame = encode(2, -sequence, &pcm(&pending))?;
                        tokio::time::timeout(IO_TIMEOUT, ws.send(Message::Binary(frame))).await
                            .map_err(|_| failure("end-of-audio timeout"))?.map_err(|_| failure("end-of-audio failed"))?;
                        let now = tokio::time::Instant::now();
                        ended_at = Some(now);
                        deadline = now + Duration::from_secs(config.final_timeout_secs);
                        tracing::debug!(timeout_secs = config.final_timeout_secs, final_samples = pending.len(), "Doubao end-of-audio sent; waiting for final response");
                    }
                }
            }
            message = ws.next() => {
                let message = message.ok_or_else(|| failure("connection closed before final result"))?
                    .map_err(|_| failure("WebSocket receive failed"))?;
                match message {
                    Message::Binary(bytes) => {
                        let response = decode(&bytes)?;
                        if !eof { deadline = tokio::time::Instant::now() + Duration::from_secs(30); }
                        if let Some(text) = response.text {
                            if transcript != text {
                                transcript = text;
                                events.send(StreamingEvent::Preview { text: transcript.clone() }).await
                                    .map_err(|_| failure("event receiver closed"))?;
                            }
                        }
                        if response.last {
                            if !eof { return Err(failure("server finalized before end-of-audio")); }
                            if let Some(start) = ended_at {
                                tracing::info!(wait_ms = start.elapsed().as_millis(), "Doubao final response received");
                            }
                            events.send(StreamingEvent::Final { text: transcript, segment_id: 0 }).await
                                .map_err(|_| failure("event receiver closed"))?;
                            return Ok(());
                        }
                    }
                    Message::Ping(data) => {
                        tokio::time::timeout(IO_TIMEOUT, ws.send(Message::Pong(data))).await
                            .map_err(|_| failure("ping timeout"))?.map_err(|_| failure("ping failed"))?;
                    }
                    Message::Pong(_) => {},
                    Message::Close(_) => return Err(failure("connection closed before final result")),
                    _ => return Err(failure("unexpected WebSocket message")),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn backend() -> DoubaoTranscriber {
        DoubaoTranscriber::new(DoubaoConfig {
            api_key: Some("test-secret".into()),
            ..Default::default()
        })
        .unwrap()
    }

    fn response(text: Option<&str>, last: bool) -> Vec<u8> {
        // Independent server fixture: uncompressed JSON with signed sequence.
        let json = match text {
            Some(t) => json!({"result": {"text": t}}),
            None => json!({}),
        };
        let payload = serde_json::to_vec(&json).unwrap();
        let mut frame = vec![0x11, if last { 0x93 } else { 0x91 }, 0x10, 0];
        frame.extend((if last { -2i32 } else { 2i32 }).to_be_bytes());
        frame.extend((payload.len() as u32).to_be_bytes());
        frame.extend(payload);
        frame
    }

    #[test]
    fn request_wire_format_and_pcm() {
        let encoded = encode(2, -3, &pcm(&[-1.0, 0.0, 1.0, f32::NAN])).unwrap();
        assert_eq!(&encoded[..4], &[0x11, 0x23, 0x01, 0]);
        assert_eq!(&encoded[4..8], &(-3i32).to_be_bytes());
        let mut pcm = Vec::new();
        GzDecoder::new(&encoded[12..])
            .read_to_end(&mut pcm)
            .unwrap();
        assert_eq!(pcm, [0, 128, 0, 0, 255, 127, 0, 0]);
        let init = encode(1, 1, &serde_json::to_vec(&init_payload()).unwrap()).unwrap();
        assert_eq!(&init[..4], &[0x11, 0x11, 0x11, 0]);
        assert_eq!(init_payload()["request"]["enable_nonstream"], true);
    }

    #[test]
    fn decode_cumulative_final_and_missing_text() {
        assert_eq!(
            decode(&response(Some("你好，世界。"), true)).unwrap(),
            Response {
                text: Some("你好，世界。".into()),
                last: true
            }
        );
        assert_eq!(decode(&response(None, true)).unwrap().text, None);
        let mut zipped = encode(1, -2, br#"{"result":{"text":"hello"}}"#).unwrap();
        zipped[1] = 0x93;
        assert_eq!(decode(&zipped).unwrap().text.as_deref(), Some("hello"));
    }

    #[test]
    fn truncated_frames_and_compression_bomb_are_rejected() {
        let frame = response(Some("中文"), true);
        for len in 0..frame.len() {
            assert!(decode(&frame[..len]).is_err(), "length {len}");
        }
        let mut bomb = encode(1, 2, &vec![b' '; MAX_PAYLOAD + 1]).unwrap();
        bomb[1] = 0x91;
        assert!(decode(&bomb).is_err());
        assert!(decode(&[0x11, 0xf0, 0x10, 0, 0, 0, 1, 147])
            .unwrap_err()
            .to_string()
            .contains("403"));
    }

    #[test]
    fn config_preserves_ptt_and_redacts_secret() {
        assert!(!format!("{:?}", backend()).contains("test-secret"));
        let cfg: crate::config::Config =
            toml::from_str("engine = 'doubao'\n[doubao]\napi_key = 'key'").unwrap();
        assert!(!cfg.streaming_active()); // No typing while the hotkey is held.
        assert!(!cfg.on_demand_loading());
        assert!(DoubaoTranscriber::new(DoubaoConfig {
            api_key: Some("\n".into()),
            ..Default::default()
        })
        .is_err());
    }

    #[tokio::test]
    #[allow(clippy::result_large_err)] // tungstenite handshake callback fixes this error type.
    async fn mock_ws_revisions_commit_once_after_eof() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("ws://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut ws = tokio_tungstenite::accept_hdr_async(
                stream,
                |req: &tokio_tungstenite::tungstenite::handshake::server::Request, resp| {
                    assert_eq!(req.headers()["x-api-key"], "test-secret");
                    assert_eq!(
                        req.headers()["x-api-resource-id"],
                        "volc.seedasr.sauc.duration"
                    );
                    assert!(req.headers().contains_key("x-api-request-id"));
                    Ok(resp)
                },
            )
            .await
            .unwrap();
            assert!(matches!(
                ws.next().await.unwrap().unwrap(),
                Message::Binary(_)
            ));
            ws.send(Message::Binary(response(Some("我想去北京"), false)))
                .await
                .unwrap();
            ws.send(Message::Binary(response(Some("我想去背景"), false)))
                .await
                .unwrap();
            loop {
                if let Message::Binary(bytes) = ws.next().await.unwrap().unwrap() {
                    if bytes[1] & 2 != 0 {
                        ws.send(Message::Binary(response(Some("我想去北京。"), true)))
                            .await
                            .unwrap();
                        break;
                    }
                }
            }
        });
        let (tx, rx) = mpsc::channel(8);
        let mut h = backend().start_at(endpoint, rx);
        tx.send(vec![0.1; CHUNK_SAMPLES]).await.unwrap();
        for expected in ["我想去北京", "我想去背景"] {
            let event = tokio::time::timeout(Duration::from_secs(3), h.events.recv())
                .await
                .unwrap()
                .unwrap();
            assert!(matches!(event, StreamingEvent::Preview { text } if text == expected));
        }
        assert!(h.events.try_recv().is_err()); // No premature typing event.
        drop(tx);
        let events = tokio::time::timeout(Duration::from_secs(3), async {
            let mut finals = Vec::new();
            while let Some(event) = h.events.recv().await {
                match event {
                    StreamingEvent::Final { text, .. } => finals.push(text),
                    StreamingEvent::Error(err) => panic!("{err}"),
                    _ => {}
                }
            }
            finals
        })
        .await
        .unwrap();
        assert_eq!(events, ["我想去北京。"]);
        h.task.await.unwrap().unwrap();
        server.await.unwrap();
    }

    #[tokio::test]
    async fn cancel_interrupts_pending_handshake() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let (tx, rx) = mpsc::channel(8);
        let h = backend().start_at(format!("ws://{}", listener.local_addr().unwrap()), rx);
        h.cancel.send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(1), h.task)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        drop(tx);
    }

    #[tokio::test]
    async fn early_close_reports_error_without_committing_partial() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("ws://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut ws = tokio_tungstenite::accept_async(stream).await.unwrap();
            ws.next().await.unwrap().unwrap();
            ws.send(Message::Binary(response(Some("未完成"), false)))
                .await
                .unwrap();
            ws.close(None).await.unwrap();
        });
        let (_tx, rx) = mpsc::channel(8);
        let mut h = backend().start_at(endpoint, rx);
        let mut failed = false;
        tokio::time::timeout(Duration::from_secs(3), async {
            while let Some(event) = h.events.recv().await {
                assert!(!matches!(event, StreamingEvent::Final { .. }));
                if matches!(event, StreamingEvent::Error(_)) {
                    failed = true;
                }
            }
        })
        .await
        .unwrap();
        assert!(failed);
        server.await.unwrap();
        h.task.await.unwrap().unwrap();
    }
    async fn delayed_final(delay: Duration, timeout_secs: u64) -> Vec<StreamingEvent> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("ws://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut ws = tokio_tungstenite::accept_async(stream).await.unwrap();
            ws.next().await.unwrap().unwrap();
            let mut received_samples = 0;
            while let Some(Ok(Message::Binary(bytes))) = ws.next().await {
                let mut audio = Vec::new();
                GzDecoder::new(&bytes[12..])
                    .read_to_end(&mut audio)
                    .unwrap();
                received_samples += audio.len() / 2;
                if bytes[1] & 2 != 0 {
                    assert!(!audio.is_empty(), "EOS must carry the final audio chunk");
                    assert_eq!(
                        received_samples,
                        CHUNK_SAMPLES * 2,
                        "do not lose or duplicate audio"
                    );
                    tokio::time::sleep(delay).await;
                    let _ = ws
                        .send(Message::Binary(response(Some("完整的长句。"), true)))
                        .await;
                    break;
                }
            }
        });
        let mut backend = backend();
        backend.config.final_timeout_secs = timeout_secs;
        let (tx, rx) = mpsc::channel(8);
        let mut h = backend.start_at(endpoint, rx);
        tx.send(vec![0.1; CHUNK_SAMPLES * 2]).await.unwrap();
        drop(tx);
        let events = tokio::time::timeout(Duration::from_secs(30), async {
            let mut events = Vec::new();
            while let Some(event) = h.events.recv().await {
                events.push(event);
            }
            events
        })
        .await
        .unwrap();
        h.task.await.unwrap().unwrap();
        server.await.unwrap();
        events
    }

    #[tokio::test]
    async fn second_pass_can_finish_after_old_twenty_second_limit() {
        let events = delayed_final(
            Duration::from_secs(21),
            DoubaoConfig::default().final_timeout_secs,
        )
        .await;
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, StreamingEvent::Final { .. }))
                .count(),
            1
        );
        assert!(!events.iter().any(|e| matches!(e, StreamingEvent::Error(_))));
    }

    #[tokio::test]
    async fn configured_final_timeout_still_bounds_wait_without_typing() {
        let events = delayed_final(Duration::from_secs(2), 1).await;
        assert!(!events
            .iter()
            .any(|e| matches!(e, StreamingEvent::Final { .. })));
        assert!(events.iter().any(
            |e| matches!(e, StreamingEvent::Error(err) if err.to_string().contains("after 1s"))
        ));
    }
}
