//! module args - Builders for FFmpeg command-line arguments.

use std::path::Path;

// --------------------------------- Macro --------------------------------- //

/// Create a Vec<String> from string literals and values that implement Display.
///
/// # Examples
/// ```
/// let input = Path::new("video.mp4");
/// let args = args!["-i", input, "-c", "copy"];
/// assert_eq!(args, vec!["-i", "video.mp4", "-c", "copy"]);
/// ```
#[macro_export]
macro_rules! args {
    // Base case: empty
    () => {
        Vec::<String>::new()
    };
    // One or more arguments
    ($($arg:expr),+ $(,)?) => {{
        let mut v = Vec::new();
        $(v.push($arg.to_string());)*
        v
    }};
}

// --------------------------------- Types, Constants & Variables ------------------------------- //

/// Codec name for stream copy (no re-encoding).
const CODEC_COPY: &str = "copy";

/// Codec name for MP3 encoding via LAME.
const CODEC_MP3: &str = "libmp3lame";

/// Codec name for H.264 encoding.
const CODEC_H264: &str = "libx264";

/// Codec name for AAC encoding.
const CODEC_AAC: &str = "aac";

/// Preset for ultrafast encoding.
const PRESET_ULTRAFAST: &str = "ultrafast";

/// Tune for still image video.
const TUNE_STILLIMAGE: &str = "stillimage";

/// Pixel format for YUV 4:2:0 planar.
const PIX_FMT_YUV420P: &str = "yuv420p";

/// Pixel format for YUVJ 4:2:0 planar (JPEG).
const PIX_FMT_YUVJ420P: &str = "yuvj420p";

/// Format name for lavfi (libavfilter input).
const FORMAT_LAVFI: &str = "lavfi";

/// Filtergraph for a black video with 1280x720 resolution, 1 frame per second.
const COLOR_FILTER: &str = "color=c=black:s=1280x720:r=1";

/// Filtergraph to scale to even dimensions only (no SAR or format conversion).
const VF_SCALE_EVEN_ONLY: &str = "scale=trunc(iw/2)*2:trunc(ih/2)*2";

/// Filtergraph to scale to even dimensions, set SAR to 1, and convert to yuv420p.
const VF_SCALE_EVEN: &str = "scale=trunc(iw/2)*2:trunc(ih/2)*2,setsar=1,format=yuv420p";

/// Filtergraph for scaling an image to a square size while preserving aspect ratio, then to rgb24.
const VF_SCALE_SQUARE_TEMPLATE: &str =
    "scale={size}:{size}:force_original_aspect_ratio=decrease,format=rgb24";

/// ID3v2 version to use.
const ID3V2_VERSION: &str = "3";

/// Write ID3v1 tag as well.
const WRITE_ID3V1: &str = "1";

/// Movflags for faststart.
const MOVFLAGS_FASTSTART: &str = "+faststart";

/// Constant rate factor for x264 encoding.
const CRF_DEFAULT: &str = "23";

// ----------------------------------------- Public API ----------------------------------------- //

/// Stream-copy a transport stream into an MP4 container.
pub fn remux_copy(input: &Path, output: &Path, force: bool) -> Vec<String> {
    let mut a = args!["-i", path(input), "-c", CODEC_COPY];
    a.extend(overwrite(force));
    a.push(path(output));
    a
}

/// Extract a single scaled frame for album art.
pub fn extract_frame(input: &Path, output: &Path, size: u32) -> Vec<String> {
    args![
        "-ss",
        "00:00:01",
        "-i",
        path(input),
        "-frames:v",
        "1",
        "-vf",
        VF_SCALE_SQUARE_TEMPLATE.replace("{size}", &size.to_string()),
        "-y",
        path(output),
    ]
}

/// Extract an MP3 attached picture without decoding it.
pub fn extract_embedded_cover(input: &Path, output: &Path) -> Vec<String> {
    args![
        "-i",
        path(input),
        "-an",
        "-vcodec",
        CODEC_COPY,
        "-y",
        path(output),
    ]
}

/// Encode the first audio track as tagged MP3, optionally adding cover art.
pub fn encode_mp3(
    input: &Path,
    cover: Option<&Path>,
    output: &Path,
    bitrate: u32,
    force: bool,
) -> Vec<String> {
    let mut a = args!["-threads", "auto", "-i", path(input)];
    if let Some(cover) = cover {
        a.extend(["-i".into(), path(cover)]);
    }
    a.extend(args![
        "-map",
        "0:a:0",
        "-c:a",
        CODEC_MP3,
        "-b:a",
        format!("{bitrate}k"),
        "-id3v2_version",
        ID3V2_VERSION,
        "-write_id3v1",
        WRITE_ID3V1,
    ]);
    if cover.is_some() {
        a.extend(args![
            "-map",
            "1:v:0",
            "-c:v",
            CODEC_COPY,
            "-disposition:v:0",
            "attached_pic",
        ]);
    }
    a.extend(overwrite(force));
    a.push(path(output));
    a
}

/// Produce an H.264 MP4 from cover art (or a black video) and MP3 audio.
pub fn encode_mp4(
    image: Option<&Path>,
    audio: &Path,
    output: &Path,
    bitrate: u32,
    force: bool,
) -> Vec<String> {
    let mut a = if let Some(image) = image {
        args!["-loop", "1", "-i", path(image)]
    } else {
        args!["-f", FORMAT_LAVFI, "-i", COLOR_FILTER,]
    };
    a.extend(args![
        "-i",
        path(audio),
        "-c:v",
        CODEC_H264,
        "-preset",
        PRESET_ULTRAFAST,
        "-tune",
        TUNE_STILLIMAGE,
        "-pix_fmt",
        PIX_FMT_YUV420P,
    ]);
    if image.is_some() {
        a.extend(args!["-vf", VF_SCALE_EVEN]);
    }
    a.extend(args![
        "-c:a",
        CODEC_AAC,
        "-b:a",
        format!("{bitrate}k"),
        "-shortest",
        "-movflags",
        MOVFLAGS_FASTSTART,
    ]);
    a.extend(overwrite(force));
    a.push(path(output));
    a
}

/// Convert an image to an even-dimension JPEG.
pub fn image_to_jpg(input: &Path, output: &Path) -> Vec<String> {
    args![
        "-i",
        path(input),
        "-vf",
        VF_SCALE_EVEN_ONLY,
        "-pix_fmt",
        PIX_FMT_YUVJ420P,
        "-y",
        path(output),
    ]
}

/// Copy A/V streams and omit attached pictures/metadata.
pub fn strip_thumbnail(input: &Path, output: &Path) -> Vec<String> {
    args![
        "-i",
        path(input),
        "-map",
        "0:v",
        "-map",
        "0:a?",
        "-map_metadata",
        "-1",
        "-c",
        CODEC_COPY,
        "-y",
        path(output),
    ]
}

/// Encode a looped image with the supplied media's audio.
///
/// Creates a video where a still image is displayed for the entire
/// duration of the audio track. Uses ultrafast preset and CRF 23
/// for quick encoding suitable for thumbnails/previews.
///
/// # Arguments
/// * `image` - Path to the source image (should be even-dimension JPEG).
/// * `media` - Path to the audio/video source providing the audio stream.
/// * `output` - Destination path for the encoded MP4.
///
/// # Returns
/// Vector of FFmpeg CLI arguments ready for process execution.
#[rustfmt::skip]
pub fn encode_loop(image: &Path, media: &Path, output: &Path) -> Vec<String> {
    args![
        "-loop", "1",   // ✅ Loop image indefinitely
        "-i", path(image),     // Image input (looped)
        "-i", path(media),     // Audio input
        "-c:v", CODEC_H264,
        "-preset", PRESET_ULTRAFAST,
        "-crf", CRF_DEFAULT,
        "-c:a", CODEC_COPY,
        "-shortest",           // Stop when shortest input ends
        "-y",
        path(output),
    ]
}

/// Attach a JPEG as an MP4 thumbnail.
pub fn attach_thumbnail(video: &Path, image: &Path, output: &Path) -> Vec<String> {
    args![
        "-i",
        path(video),
        "-i",
        path(image),
        "-map",
        "0:v",
        "-map",
        "0:a?",
        "-map",
        "1:v:0",
        "-c",
        CODEC_COPY,
        "-disposition:v:1",
        "attached_pic",
        "-y",
        path(output),
    ]
}

// -------------------------------------- Internal Helpers -------------------------------------- //

/// Converts a path to a lossy string suitable for FFmpeg arguments.
fn path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// Returns `-y` or `-n` based on the `force` flag.
fn overwrite(force: bool) -> Vec<String> {
    args![if force { "-y" } else { "-n" }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    #[test]
    fn remux_uses_copy() {
        assert_eq!(
            remux_copy(Path::new("a.ts"), Path::new("out/a.mp4"), false),
            vec!["-i", "a.ts", "-c", CODEC_COPY, "-n", "out/a.mp4"]
        );
    }
    #[test]
    fn mp3_with_cover_maps_picture() {
        let a = encode_mp3(
            Path::new("a.mkv"),
            Some(Path::new("c.jpg")),
            Path::new("a.mp3"),
            320,
            true,
        );
        assert!(a
            .windows(2)
            .any(|x| x == ["-disposition:v:0", "attached_pic"]));
    }
}
