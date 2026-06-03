// m3u8 播放列表解析模块
// 解析 HLS Master Playlist 和 Media Playlist
// 支持加密分片、字幕轨道

use anyhow::{bail, Result};
use tracing::debug;

use super::{Segment, StreamQuality};

/// 解析 Master Playlist（多码率）
///
/// 格式示例:
/// #EXT-X-STREAM-INF:BANDWIDTH=1280000,RESOLUTION=720x480
/// stream_480p.m3u8
/// #EXT-X-STREAM-INF:BANDWIDTH=2560000,RESOLUTION=1280x720
/// stream_720p.m3u8
pub fn parse_master_playlist(content: &str, base_url: &str) -> Result<Vec<StreamQuality>> {
    let mut qualities = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();

        if line.starts_with("#EXT-X-STREAM-INF:") {
            // 解析属性
            let attrs = &line[19..];
            let bandwidth = extract_attribute(attrs, "BANDWIDTH")
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(0);

            let resolution = extract_attribute(attrs, "RESOLUTION");
            let (width, height) = if let Some(res) = resolution {
                parse_resolution(res)
            } else {
                (0, 0)
            };

            let label = if height > 0 {
                format!("{}p", height)
            } else {
                format!("{}kbps", bandwidth / 1000)
            };

            // 下一行是播放列表 URL
            i += 1;
            if i < lines.len() {
                let playlist_url = resolve_url(lines[i].trim(), base_url);
                qualities.push(StreamQuality {
                    label,
                    bandwidth,
                    width,
                    height,
                    url: playlist_url,
                });
            }
        }

        i += 1;
    }

    debug!("解析到 {} 个码率", qualities.len());
    Ok(qualities)
}

/// 解析 Media Playlist（分片列表）
///
/// 格式示例:
/// #EXTM3U
/// #EXT-X-TARGETDURATION:10
/// #EXTINF:9.009,
/// segment_001.ts
/// #EXTINF:9.009,
/// segment_002.ts
/// #EXT-X-ENDLIST
pub fn parse_media_playlist(content: &str, base_url: &str) -> Result<Vec<Segment>> {
    let mut segments = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let mut current_duration: f64 = 0.0;
    let mut current_encrypted = false;
    let mut current_key_url: Option<String> = None;
    let mut current_key_iv: Option<Vec<u8>> = None;
    let mut segment_index: u32 = 0;

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();

        if line.starts_with("#EXT-X-KEY:") {
            // 加密信息
            let attrs = &line[11..];
            let method = extract_attribute(attrs, "METHOD").unwrap_or_default();

            if method != "NONE" {
                current_encrypted = true;
                current_key_url = extract_attribute(attrs, "URI").map(|s| s.to_string());
                current_key_iv = extract_attribute(attrs, "IV")
                    .and_then(|s| hex_decode(s).ok());
            } else {
                current_encrypted = false;
                current_key_url = None;
                current_key_iv = None;
            }
        } else if line.starts_with("#EXTINF:") {
            // 分片时长
            let duration_str = &line[8..];
            current_duration = duration_str
                .split(',')
                .next()
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
        } else if line.starts_with("#EXT-X-MEDIA-SEQUENCE:") {
            // 起始序号
            let seq_str = &line[22..];
            segment_index = seq_str.parse::<u32>().unwrap_or(0);
        } else if !line.starts_with('#') && !line.is_empty() {
            // 分片 URL（解析相对路径为绝对 URL）
            let url = resolve_url(line, base_url);
            segments.push(Segment {
                url,
                index: segment_index,
                duration: current_duration,
                encrypted: current_encrypted,
                key_url: current_key_url.clone(),
                key_iv: current_key_iv.clone(),
            });
            segment_index += 1;
            current_duration = 0.0;
        }

        i += 1;
    }

    debug!("解析到 {} 个分片", segments.len());
    Ok(segments)
}

/// 解析 DASH MPD 清单（使用 quick-xml）
pub fn parse_mpd(content: &str, base_url: &str) -> Result<(Vec<StreamQuality>, Vec<Segment>)> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut qualities = Vec::new();
    let mut segments = Vec::new();

    // 检测是否为 DASH
    if !content.contains("<MPD") {
        bail!("不是有效的 MPD 清单");
    }

    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);

    // 解析状态
    let mut in_representation = false;
    let mut in_segment_template = false;
    let mut in_base_url = false;
    let mut current_base_url = base_url.to_string();

    // SegmentTemplate 状态
    let mut tpl_initialization: Option<String> = None;
    let mut tpl_media: Option<String> = None;
    let mut tpl_timescale: u64 = 1;
    let mut tpl_start_number: u64 = 1;
    let mut segment_durations: Vec<f64> = Vec::new();

    // Representation 属性
    let mut rep_bandwidth: u64 = 0;
    let mut rep_width: u32 = 0;
    let mut rep_height: u32 = 0;
    let mut rep_id: String = String::new();

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                // 标记 BaseURL 标签（用于 Event::Text 处理）
                if tag_name.eq_ignore_ascii_case("BaseURL") {
                    in_base_url = true;
                }
                handle_open_tag(
                    &tag_name, e,
                    &mut in_representation, &mut in_segment_template,
                    &mut current_base_url,
                    &mut tpl_initialization, &mut tpl_media,
                    &mut tpl_timescale, &mut tpl_start_number,
                    &mut segment_durations,
                    &mut rep_bandwidth, &mut rep_width, &mut rep_height, &mut rep_id,
                )?;
            }
            Ok(Event::Empty(ref e)) => {
                // 自闭合标签（如 <S d="90000" />）
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                handle_open_tag(
                    &tag_name, e,
                    &mut in_representation, &mut in_segment_template,
                    &mut current_base_url,
                    &mut tpl_initialization, &mut tpl_media,
                    &mut tpl_timescale, &mut tpl_start_number,
                    &mut segment_durations,
                    &mut rep_bandwidth, &mut rep_width, &mut rep_height, &mut rep_id,
                )?;
                // 自闭合标签立即触发关闭逻辑
                handle_close_tag(
                    &tag_name,
                    &mut in_representation, &mut in_segment_template,
                    &rep_bandwidth, &rep_width, &rep_height, &rep_id,
                    &current_base_url, &tpl_initialization, &tpl_media,
                    tpl_start_number, &segment_durations,
                    &mut qualities, &mut segments,
                );
            }
            Ok(Event::Text(ref t)) => {
                // 仅处理 BaseURL 标签内的文本内容（避免其他标签的文本污染 base_url）
                if in_base_url {
                    let text = t.unescape().unwrap_or_default().to_string();
                    if !text.is_empty() {
                        current_base_url = resolve_url(&text, &current_base_url);
                    }
                    in_base_url = false;
                }
            }
            Ok(Event::End(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                handle_close_tag(
                    &tag_name,
                    &mut in_representation, &mut in_segment_template,
                    &rep_bandwidth, &rep_width, &rep_height, &rep_id,
                    &current_base_url, &tpl_initialization, &tpl_media,
                    tpl_start_number, &segment_durations,
                    &mut qualities, &mut segments,
                );
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                bail!("MPD 解析错误: {}", e);
            }
            _ => {}
        }
        buf.clear();
    }

    debug!("MPD 解析: {} 个质量, {} 个分片", qualities.len(), segments.len());
    Ok((qualities, segments))
}

/// 处理开标签（Start 或 Empty）
fn handle_open_tag(
    tag_name: &str,
    e: &quick_xml::events::BytesStart,
    in_representation: &mut bool,
    in_segment_template: &mut bool,
    current_base_url: &mut String,
    tpl_initialization: &mut Option<String>,
    tpl_media: &mut Option<String>,
    tpl_timescale: &mut u64,
    tpl_start_number: &mut u64,
    segment_durations: &mut Vec<f64>,
    rep_bandwidth: &mut u64,
    rep_width: &mut u32,
    rep_height: &mut u32,
    rep_id: &mut String,
) -> Result<()> {
    match tag_name {
        "Representation" => {
            *in_representation = true;
            *rep_bandwidth = 0;
            *rep_width = 0;
            *rep_height = 0;
            rep_id.clear();

            for attr in e.attributes().flatten() {
                match attr.key.as_ref() {
                    b"bandwidth" => {
                        *rep_bandwidth = String::from_utf8_lossy(&attr.value).parse().unwrap_or(0);
                    }
                    b"width" => {
                        *rep_width = String::from_utf8_lossy(&attr.value).parse().unwrap_or(0);
                    }
                    b"height" => {
                        *rep_height = String::from_utf8_lossy(&attr.value).parse().unwrap_or(0);
                    }
                    b"id" => {
                        *rep_id = String::from_utf8_lossy(&attr.value).to_string();
                    }
                    _ => {}
                }
            }
        }
        "SegmentTemplate" => {
            *in_segment_template = true;
            *tpl_initialization = None;
            *tpl_media = None;
            *tpl_timescale = 1;
            *tpl_start_number = 1;
            segment_durations.clear();

            for attr in e.attributes().flatten() {
                match attr.key.as_ref() {
                    b"initialization" => {
                        *tpl_initialization = Some(String::from_utf8_lossy(&attr.value).to_string());
                    }
                    b"media" => {
                        *tpl_media = Some(String::from_utf8_lossy(&attr.value).to_string());
                    }
                    b"timescale" => {
                        *tpl_timescale = String::from_utf8_lossy(&attr.value).parse().unwrap_or(1);
                    }
                    b"startNumber" => {
                        *tpl_start_number = String::from_utf8_lossy(&attr.value).parse().unwrap_or(1);
                    }
                    _ => {}
                }
            }
        }
        "S" => {
            // SegmentTimeline 的 S 元素（分片时长）
            if *in_segment_template {
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"d" {
                        let dur: u64 = String::from_utf8_lossy(&attr.value).parse().unwrap_or(0);
                        let seconds = dur as f64 / *tpl_timescale as f64;
                        segment_durations.push(seconds);
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// 处理闭标签
fn handle_close_tag(
    tag_name: &str,
    in_representation: &mut bool,
    in_segment_template: &mut bool,
    rep_bandwidth: &u64,
    rep_width: &u32,
    rep_height: &u32,
    rep_id: &str,
    current_base_url: &str,
    tpl_initialization: &Option<String>,
    tpl_media: &Option<String>,
    tpl_start_number: u64,
    segment_durations: &[f64],
    qualities: &mut Vec<StreamQuality>,
    segments: &mut Vec<Segment>,
) {
    match tag_name {
        "Representation" => {
            if *in_representation {
                let label = if *rep_height > 0 {
                    format!("{}p", rep_height)
                } else if *rep_bandwidth > 0 {
                    format!("{}kbps", rep_bandwidth / 1000)
                } else if !rep_id.is_empty() {
                    rep_id.to_string()
                } else {
                    "default".to_string()
                };

                qualities.push(StreamQuality {
                    label,
                    bandwidth: *rep_bandwidth,
                    width: *rep_width,
                    height: *rep_height,
                    url: current_base_url.to_string(),
                });

                // 生成分片 URL 列表
                if let Some(ref media_tpl) = tpl_media {
                    let init_url = tpl_initialization.as_ref().map(|init| {
                        expand_template(init, tpl_start_number, 0, rep_id)
                    });

                    // 初始化分片
                    if let Some(init) = init_url {
                        segments.push(Segment {
                            url: resolve_url(&init, current_base_url),
                            index: 0,
                            duration: 0.0,
                            encrypted: false,
                            key_url: None,
                            key_iv: None,
                        });
                    }

                    // 媒体分片
                    if !segment_durations.is_empty() {
                        for (i, &dur) in segment_durations.iter().enumerate() {
                            let seg_num = tpl_start_number + i as u64;
                            let url = expand_template(media_tpl, seg_num, i, rep_id);
                            segments.push(Segment {
                                url: resolve_url(&url, current_base_url),
                                index: (i + 1) as u32,
                                duration: dur,
                                encrypted: false,
                                key_url: None,
                                key_iv: None,
                            });
                        }
                    } else {
                        // 没有 SegmentTimeline，假设单个分片
                        let url = expand_template(media_tpl, tpl_start_number, 0, rep_id);
                        segments.push(Segment {
                            url: resolve_url(&url, current_base_url),
                            index: 1,
                            duration: 0.0,
                            encrypted: false,
                            key_url: None,
                            key_iv: None,
                        });
                    }
                }

                *in_representation = false;
            }
        }
        "SegmentTemplate" => {
            *in_segment_template = false;
        }
        _ => {}
    }
}

/// 展开 SegmentTemplate 中的占位符
/// $Number$ → 分片序号
/// $RepresentationID$ → 表示 ID
/// $Time$ → 时间戳
fn expand_template(template: &str, number: u64, _index: usize, rep_id: &str) -> String {
    let mut result = template.to_string();
    result = result.replace("$RepresentationID$", rep_id);
    // 处理 $Time$ 占位符（DASH SegmentTemplate 时间戳模式）
    result = result.replace("$Time$", &number.to_string());

    // 处理带零填充的格式: $Number%05d$（必须在 $Number$ 之前处理）
    // DASH 格式: $Number%0Nd$ 表示零填充 N 位
    let mut processed = String::new();
    let mut remaining = result.as_str();
    while let Some(start) = remaining.find("$Number") {
        processed.push_str(&remaining[..start]);
        let after_number = &remaining[start + 7..];
        if after_number.starts_with('%') {
            // 带格式: $Number%05d$
            if let Some(end) = after_number.find('$') {
                let fmt_spec = &after_number[1..end]; // 去掉 %
                if let Some(width_str) = fmt_spec.strip_suffix('d') {
                    if let Some(width_str) = width_str.strip_prefix('0') {
                        if let Ok(width) = width_str.parse::<usize>() {
                            processed.push_str(&format!("{:0width$}", number, width = width));
                            remaining = &after_number[end + 1..];
                            continue;
                        }
                    }
                }
                // 格式解析失败，保留原文
                processed.push_str("$Number");
                remaining = after_number;
            } else {
                processed.push_str("$Number");
                remaining = after_number;
            }
        } else if after_number.starts_with('$') {
            // 普通: $Number$
            processed.push_str(&number.to_string());
            remaining = &after_number[1..];
        } else {
            processed.push_str("$Number");
            remaining = after_number;
        }
    }
    processed.push_str(remaining);
    processed
}

// ==================== 辅助函数 ====================

/// 从属性字符串中提取值
fn extract_attribute<'a>(attrs: &'a str, name: &str) -> Option<&'a str> {
    let pattern = format!("{}=", name);
    let start = attrs.find(&pattern)?;
    let after = &attrs[start + pattern.len()..];

    if after.starts_with('"') {
        let value_start = 1;
        let value_end = after[1..].find('"')? + 1;
        Some(&after[value_start..value_end])
    } else {
        let end = after.find(|c: char| c == ',' || c == ' ' || c == '\n').unwrap_or(after.len());
        Some(&after[..end])
    }
}

/// 解析分辨率字符串（如 "1280x720"）
fn parse_resolution(s: &str) -> (u32, u32) {
    let parts: Vec<&str> = s.split('x').collect();
    if parts.len() == 2 {
        let width = parts[0].parse().unwrap_or(0);
        let height = parts[1].parse().unwrap_or(0);
        (width, height)
    } else {
        (0, 0)
    }
}

/// 解析相对 URL 为绝对 URL
fn resolve_url(url: &str, base_url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        return url.to_string();
    }

    // 基于 base_url 解析相对路径
    if let Ok(base) = url::Url::parse(base_url) {
        if let Ok(resolved) = base.join(url) {
            return resolved.to_string();
        }
    }

    url.to_string()
}

/// 十六进制解码
fn hex_decode(s: &str) -> Result<Vec<u8>> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    let mut bytes = Vec::new();
    let mut i = 0;
    let chars: Vec<char> = s.chars().collect();

    while i + 1 < chars.len() {
        let high = chars[i].to_digit(16)
            .ok_or_else(|| anyhow::anyhow!("无效的十六进制字符: '{}'", chars[i]))? as u8;
        let low = chars[i + 1].to_digit(16)
            .ok_or_else(|| anyhow::anyhow!("无效的十六进制字符: '{}'", chars[i + 1]))? as u8;
        bytes.push((high << 4) | low);
        i += 2;
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_master_playlist() {
        let content = r#"#EXTM3U
#EXT-X-STREAM-INF:BANDWIDTH=1280000,RESOLUTION=720x480
stream_480p.m3u8
#EXT-X-STREAM-INF:BANDWIDTH=2560000,RESOLUTION=1280x720
stream_720p.m3u8"#;

        let qualities = parse_master_playlist(content, "https://example.com/master.m3u8").unwrap();
        assert_eq!(qualities.len(), 2);
        assert_eq!(qualities[0].label, "480p");
        assert_eq!(qualities[1].label, "720p");
    }

    #[test]
    fn test_parse_media_playlist() {
        let content = r#"#EXTM3U
#EXT-X-TARGETDURATION:10
#EXTINF:9.009,
segment_001.ts
#EXTINF:9.009,
segment_002.ts
#EXTINF:3.003,
segment_003.ts
#EXT-X-ENDLIST"#;

        let segments = parse_media_playlist(content, "https://example.com/live.m3u8").unwrap();
        assert_eq!(segments.len(), 3);
        assert!((segments[0].duration - 9.009).abs() < 0.01);
    }

    #[test]
    fn test_extract_attribute() {
        let attrs = r#"BANDWIDTH=1280000,RESOLUTION="720x480",CODECS="avc1""#;
        assert_eq!(extract_attribute(attrs, "BANDWIDTH"), Some("1280000"));
        assert_eq!(extract_attribute(attrs, "RESOLUTION"), Some("720x480"));
        assert_eq!(extract_attribute(attrs, "CODECS"), Some("avc1"));
    }

    #[test]
    fn test_parse_mpd_with_base_url() {
        let mpd = r#"<?xml version="1.0"?>
<MPD mediaPresentationDuration="PT30S" type="static">
  <Period>
    <BaseURL>https://cdn.example.com/dash/</BaseURL>
    <AdaptationSet mimeType="video/mp4">
      <Representation bandwidth="1280000" width="720" height="480" id="v1">
        <BaseURL>video_480p/</BaseURL>
      </Representation>
      <Representation bandwidth="2560000" width="1280" height="720" id="v2">
        <BaseURL>video_720p/</BaseURL>
      </Representation>
    </AdaptationSet>
  </Period>
</MPD>"#;

        let (qualities, _segments) = parse_mpd(mpd, "https://example.com/master.mpd").unwrap();
        assert_eq!(qualities.len(), 2);
        assert_eq!(qualities[0].label, "480p");
        assert_eq!(qualities[0].bandwidth, 1280000);
        assert_eq!(qualities[1].label, "720p");
        assert_eq!(qualities[1].bandwidth, 2560000);
    }

    #[test]
    fn test_parse_mpd_with_segment_template() {
        let mpd = r#"<?xml version="1.0"?>
<MPD mediaPresentationDuration="PT30S" type="static">
  <Period>
    <AdaptationSet mimeType="video/mp4">
      <SegmentTemplate timescale="90000" initialization="init_$RepresentationID$.mp4" media="seg_$RepresentationID$_$Number$.m4s" startNumber="1">
        <SegmentTimeline>
          <S d="90000" />
          <S d="90000" />
          <S d="45000" />
        </SegmentTimeline>
      </SegmentTemplate>
      <Representation bandwidth="1280000" width="720" height="480" id="v1" />
    </AdaptationSet>
  </Period>
</MPD>"#;

        let (qualities, segments) = parse_mpd(mpd, "https://example.com/master.mpd").unwrap();
        assert_eq!(qualities.len(), 1);
        assert_eq!(qualities[0].label, "480p");

        // 1 init segment + 3 media segments
        assert_eq!(segments.len(), 4);
        assert_eq!(segments[0].index, 0); // init
        assert_eq!(segments[1].index, 1);
        assert!((segments[1].duration - 1.0).abs() < 0.01); // 90000/90000
        assert!((segments[3].duration - 0.5).abs() < 0.01); // 45000/90000
    }

    #[test]
    fn test_expand_template() {
        assert_eq!(
            expand_template("seg_$Number$.m4s", 5, 0, "v1"),
            "seg_5.m4s"
        );
        assert_eq!(
            expand_template("seg_$RepresentationID$_$Number$.m4s", 3, 0, "v1"),
            "seg_v1_3.m4s"
        );
        let result = expand_template("seg_$Number%05d$.m4s", 42, 0, "v1");
        eprintln!("expand_template result: '{}'", result);
        assert_eq!(result, "seg_00042.m4s");
    }

    #[test]
    fn test_parse_mpd_invalid() {
        let result = parse_mpd("<html>not an mpd</html>", "https://example.com/");
        assert!(result.is_err());
    }
}
