use std::path::Path;

fn path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn overwrite(force: bool) -> Vec<String> {
    vec![if force { "-y" } else { "-n" }.into()]
}


/// Stream-copy a transport stream into an MP4 container.
pub fn remux_copy(input: &Path, output: &Path, force: bool) -> Vec<String> {
    let mut a = vec!["-i".into(), path(input), "-c".into(), "copy".into()];
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
        format!("scale={size}:{size}:force_original_aspect_ratio=decrease,format=rgb24"),
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
        "copy".into(),
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
        "libmp3lame".into(),
        "-b:a".into(),
        format!("{bitrate}k"),
        "-id3v2_version".into(),
        "3".into(),
        "-write_id3v1".into(),
        "1".into(),
    ]);
    if cover.is_some() {
        a.extend([
            "-map".into(),
            "1:v:0".into(),
            "-c:v".into(),
            "copy".into(),
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
            "lavfi".into(),
            "-i".into(),
            "color=c=black:s=1280x720:r=1".into(),
        ]
    };
    a.extend([
        "-i".into(),
        path(audio),
        "-c:v".into(),
        "libx264".into(),
        "-preset".into(),
        "ultrafast".into(),
        "-tune".into(),
        "stillimage".into(),
        "-pix_fmt".into(),
        "yuv420p".into(),
    ]);
    if image.is_some() {
        a.extend([
            "-vf".into(),
            "scale=trunc(iw/2)*2:trunc(ih/2)*2,setsar=1,format=yuv420p".into(),
        ]);
    }
    a.extend([
        "-c:a".into(),
        "aac".into(),
        "-b:a".into(),
        format!("{bitrate}k"),
        "-shortest".into(),
        "-movflags".into(),
        "+faststart".into(),
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
        "scale=trunc(iw/2)*2:trunc(ih/2)*2".into(),
        "-pix_fmt".into(),
        "yuvj420p".into(),
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
        "copy".into(),
        "-y".into(),
        path(output),
    ]
}

/// Encode a looped image with the supplied media's audio.
pub fn encode_loop(image: &Path, media: &Path, output: &Path) -> Vec<String> {
    vec![
        "-loop".into(),
        "1".into(),
        "-i".into(),
        path(image),
        "-i".into(),
        path(media),
        "-c:v".into(),
        "libx264".into(),
        "-preset".into(),
        "ultrafast".into(),
        "-crf".into(),
        "23".into(),
        "-c:a".into(),
        "copy".into(),
        "-shortest".into(),
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
        "copy".into(),
        "-disposition:v:1".into(),
        "attached_pic".into(),
        "-y".into(),
        path(output),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    #[test]
    fn remux_uses_copy() {
        assert_eq!(
            remux_copy(Path::new("a.ts"), Path::new("out/a.mp4"), false),
            vec!["-i", "a.ts", "-c", "copy", "-n", "out/a.mp4"]
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
