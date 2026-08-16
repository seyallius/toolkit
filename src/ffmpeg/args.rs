//! module args - Builders for FFmpeg command-line arguments.

use std::path::Path;

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
    let mut a = vec!["-i".into(), path(input), "-c".into(), CODEC_COPY.into()];
    a.extend(overwrite(force));
    a.push(path(output));
    a
}

/// Extract a single scaled frame for album art.
pub fn extract_frame(input: &Path, output: &Path, size: u32) -> Vec<String> {
    vec![
        "-ss".into(),
        "00:00:01".into(),
        "-i".into(),
        path(input),
        "-frames:v".into(),
        "1".into(),
        "-vf".into(),
        VF_SCALE_SQUARE_TEMPLATE.replace("{size}", &size.to_string()),
        "-y".into(),
        path(output),
    ]
}

/// Extract an MP3 attached picture without decoding it.
pub fn extract_embedded_cover(input: &Path, output: &Path) -> Vec<String> {
    vec![
        "-i".into(),
        path(input),
        "-an".into(),
        "-vcodec".into(),
        CODEC_COPY.into(),
        "-y".into(),
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
    let mut a = vec!["-threads".into(), "auto".into(), "-i".into(), path(input)];
    if let Some(cover) = cover {
        a.extend(["-i".into(), path(cover)]);
    }
    a.extend([
        "-map".into(),
        "0:a:0".into(),
        "-c:a".into(),
        CODEC_MP3.into(),
        "-b:a".into(),
        format!("{bitrate}k"),
        "-id3v2_version".into(),
        ID3V2_VERSION.into(),
        "-write_id3v1".into(),
        WRITE_ID3V1.into(),
    ]);
    if cover.is_some() {
        a.extend([
            "-map".into(),
            "1:v:0".into(),
            "-c:v".into(),
            CODEC_COPY.into(),
            "-disposition:v:0".into(),
            "attached_pic".into(),
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
        vec!["-loop".into(), "1".into(), "-i".into(), path(image)]
    } else {
        vec![
            "-f".into(),
            FORMAT_LAVFI.into(),
            "-i".into(),
            COLOR_FILTER.into(),
        ]
    };
    a.extend([
        "-i".into(),
        path(audio),
        "-c:v".into(),
        CODEC_H264.into(),
        "-preset".into(),
        PRESET_ULTRAFAST.into(),
        "-tune".into(),
        TUNE_STILLIMAGE.into(),
        "-pix_fmt".into(),
        PIX_FMT_YUV420P.into(),
    ]);
    if image.is_some() {
        a.extend(["-vf".into(), VF_SCALE_EVEN.into()]);
    }
    a.extend([
        "-c:a".into(),
        CODEC_AAC.into(),
        "-b:a".into(),
        format!("{bitrate}k"),
        "-shortest".into(),
        "-movflags".into(),
        MOVFLAGS_FASTSTART.into(),
    ]);
    a.extend(overwrite(force));
    a.push(path(output));
    a
}

/// Convert an image to an even-dimension JPEG.
pub fn image_to_jpg(input: &Path, output: &Path) -> Vec<String> {
    vec![
        "-i".into(),
        path(input),
        "-vf".into(),
        VF_SCALE_EVEN_ONLY.into(),
        "-pix_fmt".into(),
        PIX_FMT_YUVJ420P.into(),
        "-y".into(),
        path(output),
    ]
}

/// Copy A/V streams and omit attached pictures/metadata.
pub fn strip_thumbnail(input: &Path, output: &Path) -> Vec<String> {
    vec![
        "-i".into(),
        path(input),
        "-map".into(),
        "0:v".into(),
        "-map".into(),
        "0:a?".into(),
        "-map_metadata".into(),
        "-1".into(),
        "-c".into(),
        CODEC_COPY.into(),
        "-y".into(),
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
    vec![
        "-loop".into(), "1".into(),   // ✅ Loop image indefinitely
        "-i".into(), path(image),     // Image input (looped)
        "-i".into(), path(media),     // Audio input
        "-c:v".into(), CODEC_H264.into(),
        "-preset".into(), PRESET_ULTRAFAST.into(),
        "-crf".into(), CRF_DEFAULT.into(),
        "-c:a".into(), CODEC_COPY.into(),
        "-shortest".into(),           // Stop when shortest input ends
        "-y".into(),
        path(output),
    ]
}

/// Attach a JPEG as an MP4 thumbnail.
pub fn attach_thumbnail(video: &Path, image: &Path, output: &Path) -> Vec<String> {
    vec![
        "-i".into(),
        path(video),
        "-i".into(),
        path(image),
        "-map".into(),
        "0:v".into(),
        "-map".into(),
        "0:a?".into(),
        "-map".into(),
        "1:v:0".into(),
        "-c".into(),
        CODEC_COPY.into(),
        "-disposition:v:1".into(),
        "attached_pic".into(),
        "-y".into(),
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
    vec![if force { "-y" } else { "-n" }.into()]
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
