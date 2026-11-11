use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use regex::Regex;
use roxmltree::{Document, Node};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::env;
use std::sync::{mpsc, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

const API: &str = "https://lrclib.net/api";
const AMLL: &str = "https://raw.githubusercontent.com/amll-dev/amll-ttml-db/main/spotify-lyrics";
const NETEASE: &str = "https://music.163.com/api";
const USER_AGENT: &str = "QS-Music-Player/1.1 (https://github.com/zonicisalive/quickshell-music-player/)";
const FALLBACK_GRACE: Duration = Duration::from_millis(2500);

type Lines = Vec<Value>;
type ProviderResult = Result<Lines, String>;

fn encode(value: &str) -> String {
    utf8_percent_encode(value, NON_ALPHANUMERIC).to_string()
}

fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

fn request(url: &str, netease: bool, timeout: Duration) -> Result<String, String> {
    let agent = ureq::AgentBuilder::new().timeout(timeout).build();
    let mut retry = false;
    loop {
        let req = agent.get(url);
        let req = if netease {
            req.set("User-Agent", "Mozilla/5.0")
                .set("Referer", "https://music.163.com")
        } else {
            req.set("User-Agent", USER_AGENT)
        };
        match req.call() {
            Ok(resp) => return resp.into_string().map_err(|e| e.to_string()),
            Err(ureq::Error::Status(429, resp)) if !retry => {
                retry = true;
                let delay = resp.header("Retry-After")
                    .and_then(|s| s.parse::<f64>().ok())
                    .filter(|d| d.is_finite())
                    .unwrap_or(1.0)
                    .clamp(0.0, 6.0);
                thread::sleep(Duration::from_secs_f64(delay));
            }
            Err(error) => return Err(error.to_string()),
        }
    }
}

fn get_json(url: &str, netease: bool) -> Result<Value, String> {
    let raw = request(url, netease, Duration::from_secs(6))?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

fn parse_lrc(lrc: &str) -> Lines {
    static STAMP: OnceLock<Regex> = OnceLock::new();
    let re = STAMP.get_or_init(|| Regex::new(r"\[(\d{1,3}):(\d{2}(?:[.:]\d{1,3})?)\]").unwrap());
    let mut lines = Vec::new();
    for raw in lrc.lines() {
        let stamps: Vec<_> = re.captures_iter(raw).collect();
        if stamps.is_empty() {
            continue;
        }
        let text = raw[stamps.last().unwrap().get(0).unwrap().end()..].trim();
        for stamp in stamps {
            let minutes = stamp[1].parse::<f64>().unwrap_or(0.0);
            let seconds = stamp[2].replace(':', ".").parse::<f64>().unwrap_or(0.0);
            lines.push(json!({"t": round3(minutes * 60.0 + seconds), "text": text}));
        }
    }
    lines.sort_by(|a, b| f64_value(a, "t").total_cmp(&f64_value(b, "t")));
    lines
}

fn normalized(value: &str) -> String {
    value.to_lowercase().chars()
        .map(|ch| if ch.is_alphanumeric() || ch.is_whitespace() || ch == '_' { ch } else { ' ' })
        .collect::<String>()
        .trim().to_string()
}

fn overlaps(wanted: &str, got: &str) -> bool {
    let wanted = normalized(wanted);
    let got = normalized(got);
    if wanted.is_empty() || got.is_empty() { return false; }
    if wanted.contains(&got) || got.contains(&wanted) { return true; }
    let significant: HashSet<&str> = wanted.split_whitespace().filter(|w| w.chars().count() > 3).collect();
    got.split_whitespace().any(|w| significant.contains(w))
}

fn str_value<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

fn f64_value(v: &Value, key: &str) -> f64 {
    v.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

fn first_line_time(lines: &[Value]) -> Option<f64> {
    lines.iter().find(|v| !str_value(v, "text").trim().is_empty())
        .and_then(|v| v.get("t").and_then(Value::as_f64))
}

fn pick_by_consensus(candidates: Vec<(Lines, f64)>, duration: Option<f64>) -> Lines {
    let mut groups: Vec<(f64, Vec<(f64, Lines)>)> = Vec::new();
    for (lines, length) in candidates {
        let Some(start) = first_line_time(&lines) else { continue };
        // Same half-second buckets and neighbouring-bucket folding as lyrics.py.
        let mut key = (start * 2.0).round() / 2.0;
        for near in [key - 0.5, key + 0.5] {
            if groups.iter().any(|(existing, _)| *existing == near) {
                key = near;
                break;
            }
        }
        let gap = if let Some(d) = duration.filter(|d| *d != 0.0) {
            if length != 0.0 { (length - d).abs() } else { 0.0 }
        } else { 0.0 };
        if let Some((_, entries)) = groups.iter_mut().find(|(k, _)| *k == key) {
            entries.push((gap, lines));
        } else {
            groups.push((key, vec![(gap, lines)]));
        }
    }
    let Some((_, mut group)) = groups.into_iter().max_by_key(|(_, rows)| rows.len()) else {
        return vec![];
    };
    group.sort_by(|a, b| a.0.total_cmp(&b.0));
    group.remove(0).1
}

fn take_lrclib_candidates(
    payload: Value, title: &str, artist: &str,
    seen: &mut HashSet<String>, candidates: &mut Vec<(Lines, f64)>
) {
    let collection = if let Value::Array(items) = payload { items } else { vec![payload] };
    for candidate in collection {
        if !candidate.is_object() { continue; }
        let id = candidate.get("id").cloned().unwrap_or(Value::Null).to_string();
        if !seen.insert(id) { continue; }
        if str_value(&candidate, "syncedLyrics").is_empty()
            || !overlaps(title, str_value(&candidate, "trackName"))
            || !overlaps(artist, str_value(&candidate, "artistName")) { continue; }
        let lines = parse_lrc(str_value(&candidate, "syncedLyrics"));
        if !lines.is_empty() { candidates.push((lines, f64_value(&candidate, "duration"))); }
    }
}

fn find_lyrics(title: &str, artist: &str, album: &str, duration: Option<f64>) -> ProviderResult {
    let mut urls = Vec::new();
    if !album.is_empty() {
        if let Some(duration) = duration.filter(|n| *n != 0.0) {
            urls.push(format!("{API}/get?track_name={}&artist_name={}&album_name={}&duration={}",
                encode(title), encode(artist), encode(album), duration.trunc() as i64));
        }
    }
    urls.push(format!("{API}/search?track_name={}&artist_name={}", encode(title), encode(artist)));
    let broad = format!("{API}/search?q={}", encode(&format!("{title} {artist}")));

    let (tx, rx) = mpsc::channel();
    for (idx, url) in urls.iter().enumerate() {
        let tx = tx.clone();
        let url = url.clone();
        thread::spawn(move || { let _ = tx.send((idx, get_json(&url, false))); });
    }
    drop(tx);
    let mut results: Vec<Option<Result<Value, String>>> = (0..urls.len()).map(|_| None).collect();
    for _ in 0..urls.len() {
        if let Ok((idx, result)) = rx.recv() { results[idx] = Some(result); }
    }
    let mut last_error = None;
    let mut had_response = false;
    let mut seen = HashSet::new();
    let mut candidates: Vec<(Lines, f64)> = Vec::new();
    for result in results.into_iter().flatten() {
        match result {
            Ok(payload) => { take_lrclib_candidates(payload, title, artist, &mut seen, &mut candidates); had_response = true; }
            Err(error) => last_error = Some(error),
        }
    }
    if candidates.is_empty() {
        thread::sleep(Duration::from_millis(250));
        match get_json(&broad, false) {
            Ok(payload) => { take_lrclib_candidates(payload, title, artist, &mut seen, &mut candidates); had_response = true; }
            Err(error) => last_error = Some(error),
        }
    }
    if !candidates.is_empty() { return Ok(pick_by_consensus(candidates, duration)); }
    if !had_response {
        if let Some(error) = last_error { return Err(error); }
    }
    Ok(vec![])
}

fn clock_seconds(value: &str) -> Option<f64> {
    let value = value.trim().strip_suffix('s').unwrap_or(value.trim());
    let parts: Vec<&str> = value.split(':').collect();
    if parts.is_empty() || parts.len() > 3 { return None; }
    let mut total = 0.0;
    for (i, part) in parts.iter().enumerate() {
        let n: f64 = part.parse().ok()?;
        if i + 1 < parts.len() && (!part.chars().all(|c| c.is_ascii_digit()) || n < 0.0) {
            return None;
        }
        total = total * 60.0 + n;
    }
    if total.is_finite() { Some(round3(total)) } else { None }
}

fn attr_role(node: Node<'_, '_>) -> String {
    node.attributes().find(|a| a.name() == "role")
        .map(|a| a.value().to_string()).unwrap_or_default()
}

#[derive(Debug)]
struct Word {
    begin: Option<f64>,
    end: Option<f64>,
    text: String,
    tail: String,
}

fn leaf_spans(node: Node<'_, '_>, words: &mut Vec<Word>) {
    for child in node.children().filter(|n| n.is_element() && n.tag_name().name() == "span") {
        if ["x-bg", "x-translation", "x-roman"].contains(&attr_role(child).as_str()) { continue; }
        if child.children().any(|n| n.is_element() && n.tag_name().name() == "span") {
            leaf_spans(child, words);
            continue;
        }
        let text = child.children().find(|n| n.is_text()).and_then(|n| n.text()).unwrap_or("");
        if text.trim().is_empty() { continue; }
        let tail = child.next_sibling().filter(|n| n.is_text()).and_then(|n| n.text()).unwrap_or("");
        words.push(Word {
            begin: child.attribute("begin").and_then(clock_seconds),
            end: child.attribute("end").and_then(clock_seconds),
            text: text.to_owned(),
            tail: tail.to_owned(),
        });
    }
}

fn parse_ttml(raw: &str) -> ProviderResult {
    let doc = Document::parse(raw).map_err(|e| e.to_string())?;
    let mut lines = Vec::new();
    for node in doc.descendants().filter(|n| n.is_element() && n.tag_name().name() == "p") {
        let Some(start) = node.attribute("begin").and_then(clock_seconds) else { continue };
        let mut spans = Vec::new();
        leaf_spans(node, &mut spans);
        let mut words = Vec::new();
        let mut text = String::new();
        for span in spans {
            let Some(begin) = span.begin else { continue };
            let end = span.end.unwrap_or(begin);
            words.push(json!({
                "t": begin,
                "d": round3((end - begin).max(0.0)),
                "text": span.text,
                "sp": span.tail.chars().next().is_some_and(char::is_whitespace)
            }));
            // The original script joins each syllable with its tail spacing.
            text.push_str(&span.text);
            text.push_str(&span.tail);
        }
        let text = text.trim();
        if !text.is_empty() && !words.is_empty() {
            lines.push(json!({"t": start, "text": text, "words": words}));
        } else {
            let plain: String = node.descendants().filter(|n| n.is_text()).filter_map(|n| n.text()).collect();
            if !plain.trim().is_empty() { lines.push(json!({"t": start, "text": plain.trim()})); }
        }
    }
    lines.sort_by(|a, b| f64_value(a, "t").total_cmp(&f64_value(b, "t")));
    Ok(lines)
}

fn fetch_amll(spotify_id: &str) -> ProviderResult {
    if spotify_id.len() != 22 || !spotify_id.bytes().all(|b| b.is_ascii_alphanumeric()) {
        return Ok(vec![]);
    }
    let url = format!("{AMLL}/{spotify_id}.ttml");
    match request(&url, false, Duration::from_millis(2500)) {
        Ok(raw) => parse_ttml(&raw).or(Ok(vec![])),
        Err(error) => {
            // AMLL is an optional source. An unavailable CDN is a miss, not a fatal failure.
            let _ = error;
            Ok(vec![])
        }
    }
}

fn credit_line(line: &str) -> bool {
    // Recognize metadata credit lines without dropping real lyrics containing a colon.
    static CREDIT: OnceLock<Regex> = OnceLock::new();
    let re = CREDIT.get_or_init(|| Regex::new(concat!(
        r"(?i)^\s*(?:作词|作曲|编曲|制作人|制作|监制|混音|母带|录音|和声|和音|吉他|贝斯|鼓|键盘|弦乐|",
        r"出品|发行|策划|统筹|企划|OP|SP|词|曲|",
        r"lyricist|lyrics?|composer|composed by|written by|writer|producer|produced by|",
        r"arranger|arranged by|mix(?:ing)?(?: engineer)?|master(?:ing)?(?: engineer)?|recording|",
        r"vocal(?: producer)?|guitar|bass|drums|keyboards?|strings|publisher|label)",
        r"\s*(?:by)?\s*[:：]"
    )).unwrap());
    re.is_match(line)
}

fn fetch_netease(title: &str, artist: &str, duration: Option<f64>) -> ProviderResult {
    let url = format!("{NETEASE}/search/get?s={}&type=1&limit=5", encode(&format!("{title} {artist}")));
    let response = get_json(&url, true)?;
    let songs = response.pointer("/result/songs").and_then(Value::as_array);
    let mut matches = Vec::new();
    for song in songs.into_iter().flatten() {
        if !overlaps(title, str_value(song, "name")) { continue; }
        let names = song.get("artists").and_then(Value::as_array)
            .map(|artists| artists.iter().map(|a| str_value(a, "name")).collect::<Vec<_>>().join(" "))
            .unwrap_or_default();
        if !overlaps(artist, &names) { continue; }
        let gap = duration.filter(|d| *d != 0.0)
            .map(|d| (f64_value(song, "duration") / 1000.0 - d).abs())
            .unwrap_or(0.0);
        matches.push((gap, song));
    }
    matches.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (gap, song) in matches {
        if duration.is_some_and(|d| d != 0.0 && gap > 20.0) { break; }
        let id = match song.get("id") {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Number(n)) => n.to_string(),
            _ => continue,
        };
        let url = format!("{NETEASE}/song/lyric?id={id}&lv=1&kv=1&tv=-1");
        let payload = get_json(&url, true)?;
        let lrc = payload.pointer("/lrc/lyric").and_then(Value::as_str).unwrap_or("");
        let lines: Lines = parse_lrc(lrc).into_iter()
            .filter(|line| !credit_line(str_value(line, "text"))).collect();
        if !lines.is_empty() { return Ok(lines); }
    }
    Ok(vec![])
}

fn emit(payload: Value) {
    println!("{}", payload);
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let title = args.first().map(|s| s.trim()).unwrap_or("");
    let artist = args.get(1).map(|s| s.trim()).unwrap_or("");
    let album = args.get(2).map(|s| s.trim()).unwrap_or("");
    let request_id = args.get(4).map(|s| s.trim()).unwrap_or("").to_string();
    let spotify_id = args.get(5).map(|s| s.trim()).unwrap_or("");
    if title.is_empty() || artist.is_empty() {
        emit(json!({"status":"no_info", "requestId":request_id}));
        return;
    }
    let duration = args.get(3).and_then(|s| s.parse::<f64>().ok()).filter(|d| d.is_finite());

    let (tx, rx) = mpsc::channel::<(&'static str, ProviderResult)>();
    let id = spotify_id.to_string();
    let sender = tx.clone();
    thread::spawn(move || { let _ = sender.send(("amll", fetch_amll(&id))); });
    let (title_l, artist_l, album_l) = (title.to_owned(), artist.to_owned(), album.to_owned());
    let sender = tx.clone();
    thread::spawn(move || {
        let _ = sender.send(("lrclib", find_lyrics(&title_l, &artist_l, &album_l, duration)));
    });
    let (title_n, artist_n) = (title.to_owned(), artist.to_owned());
    thread::spawn(move || { let _ = tx.send(("netease", fetch_netease(&title_n, &artist_n, duration))); });

    let priority = ["amll", "lrclib", "netease"];
    let mut results: HashMap<&str, ProviderResult> = HashMap::new();
    let mut last_error: Option<String> = None;
    let mut fallback_since: Option<Instant> = None;
    loop {
        // The best completed provider wins once higher-priority providers have finished.
        for name in priority {
            match results.get(name) {
                None => break,
                Some(Ok(lines)) if !lines.is_empty() => {
                    emit(json!({
                        "status": "ok", "source": name,
                        "worded": lines.iter().any(|line| line.get("words").is_some_and(|v| v.as_array().is_some_and(|a| !a.is_empty()))),
                        "lines": lines, "requestId": request_id
                    }));
                    return;
                }
                _ => {}
            }
        }
        let fallback = priority.iter().find_map(|name| match results.get(name) {
            Some(Ok(lines)) if !lines.is_empty() => Some(*name),
            _ => None,
        });
        if let Some(name) = fallback {
            match fallback_since {
                None => fallback_since = Some(Instant::now()),
                Some(since) if since.elapsed() >= FALLBACK_GRACE => {
                    let lines = results.get(name).unwrap().as_ref().unwrap();
                    emit(json!({
                        "status": "ok", "source": name,
                        "worded": lines.iter().any(|line| line.get("words").is_some_and(|v| v.as_array().is_some_and(|a| !a.is_empty()))),
                        "lines": lines, "requestId": request_id
                    }));
                    return;
                }
                _ => {}
            }
        }
        if results.len() == priority.len() { break; }
        let wait = if fallback.is_some() { Duration::from_millis(50) } else { Duration::from_millis(250) };
        match rx.recv_timeout(wait) {
            Ok((name, result)) => {
                if let Err(error) = &result { last_error = Some(error.clone()); }
                results.insert(name, result);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    if let Some(message) = last_error {
        emit(json!({"status":"error", "message":message, "requestId":request_id}));
    } else {
        emit(json!({"status":"not_found", "requestId":request_id}));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lrc_multiple_timestamps() {
        let lines = parse_lrc("[00:01.25][00:02.50] Hello\n[00:03.0] World");
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0]["t"], 1.25);
        assert_eq!(lines[1]["t"], 2.5);
        assert_eq!(lines[0]["text"], "Hello");
    }

    #[test]
    fn ttml_word_timing() {
        let xml = r#"<tt xmlns:ttm="http://www.w3.org/ns/ttml#metadata"><body><div>
          <p begin="00:01.0"><span begin="1.0s" end="1.5s">Hel</span><span begin="1.5s" end="2s">lo</span> world</p>
          <p begin="00:03.0">Plain line</p>
          <p begin="00:04.0"><span ttm:role="x-bg" begin="4s">Background</span><span begin="4s" end="5s">Main</span></p>
          </div></body></tt>"#;
        let lines = parse_ttml(xml).unwrap();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0]["words"].as_array().unwrap().len(), 2);
        assert_eq!(lines[0]["words"][0]["text"], "Hel");
        assert_eq!(lines[0]["words"][0]["sp"], false);
        assert_eq!(lines[0]["words"][1]["sp"], true);
        assert_eq!(lines[2]["text"], "Main");
    }

    #[test]
    fn lrclib_consensus() {
        let entries = vec![
            (vec![json!({"t": 12.0, "text":"a"})], 200.0),
            (vec![json!({"t": 12.2, "text":"b"})], 201.0),
            (vec![json!({"t": 3.0, "text":"c"})], 200.0),
        ];
        assert_eq!(pick_by_consensus(entries, Some(201.0))[0]["text"], "b");
    }
}
