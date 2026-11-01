"""An independent check of an exported MP4.

    verify_mp4.py <file> --profile free|creator --expected-duration-ms N [--source <fixture>]

N is how long the export should be. With the identity map that is the source's
duration; it comes from the test and is separate from --source, which is what
checks 8 and 11 compare with (v5implementation D-210). One line is printed per
check, "PASS <n>" or "FAIL <n>: <reason>"; a line starting "INFO <n>:" reports a
reading and judges nothing. The exit status is 0 only when every check passes;
1 when one fails; 2 when a tool is missing or the arguments are wrong.

This program judges what the app wrote, so it shares nothing with it: it runs
the ffprobe and ffmpeg command-line tools, reads the top-level boxes of the file
itself, and does its arithmetic with NumPy, which the product does not use.
"""

import argparse
import json
import shutil
import struct
import subprocess
import sys
from fractions import Fraction
from pathlib import Path

try:
    import numpy as np
except ImportError:
    print("verify_mp4: NumPy cannot be imported: run pip install -r verify/requirements.txt", file=sys.stderr)
    sys.exit(2)

# What each profile's video must measure (width, height).
PROFILE_SIZES = {"creator": (1080, 1920), "free": (720, 1280)}
FRAME_RATE = Fraction(30, 1)
# How far the video's length may be from the expected one: one frame, 33.4 ms.
VIDEO_TOLERANCE_MS = Fraction(334, 10)
# How far the audio's length may be from the video's: one AAC frame of 1,024
# samples at 48 kHz, 21.33 ms, rounded up as the video's frame is.
AUDIO_TOLERANCE_MS = Fraction(2134, 100)

# Check 7: the integrated loudness of the stream as it is, in stereo (TS section 27.2).
LOUDNESS_TARGET, LOUDNESS_TOLERANCE = -14.0, 1.0
# Check 8: RMS envelopes in hops of 10 ms, over the first and the last 5 s of
# the export, searched within 1 s either way (v5implementation D-205).
AUDIO_RATE = 48_000
ENVELOPE_HOP_SAMPLES = 480
HOP_MS = 10
SYNC_WINDOW_MS, SYNC_LAG_HOPS, SYNC_LIMIT_MS = 5000, 100, 20
MIN_ENVELOPE_CORR = 0.5  # (assumption, P130)
SILENT_WINDOW_DBFS = -50.0  # (assumption)
# A white frame: nearly white, and clearly brighter than the frames around it. Read
# in P130: the fixtures' white is 234, on scenes up to 195, so 39 above; an export's
# is 254. A cut to a bright scene does not count: that scene raises the median too.
FLASH_MIN, FLASH_ABOVE = 225, 30
FLASH_SURROUND = 30
# Check 9: what stays still over frames across the clip, against the mask (D-206).
MASK = Path(__file__).resolve().parent / "watermark_mask_720.png"
MASK_WIDTH, MASK_HEIGHT = 720, 1280
MASK_BACKGROUND = 128
MASK_MARGIN = 4
HIGH_PASS_SIZE = 9
WATERMARK_FRAMES = 15  # (assumption)
WATERMARK_FREE_MIN, WATERMARK_CREATOR_MAX = 0.5, 0.2  # (assumption, TS section 39.3 item 14)
# Check 10: 1.5 times the target of each profile, held here (D-207).
BITRATE_LIMIT = {"creator": 12_000_000, "free": 6_000_000}
# Check 11 (D-100, D-208). A reason names a tag's key, never its value: a
# source may carry where it was filmed.
PLACE_KEYS = ("location", "iso6709")
TEXT_KEYS = ("title", "comment", "description", "artist", "album")
SOURCE_KEYS = ("title", "comment", "encoder", "creation_time")
EPOCH = "1970-01-01T00:00:00"
PLACE_BYTES = (b"\xa9xyz", b"ISO6709")
READ_PIECE = 1024 * 1024
READ_OVERLAP = 7


class Failed(Exception):
    """A check that did not pass; the message is its reason."""


def run(command):
    """Runs a tool and returns (exit status, stdout, stderr), both as text."""
    done = subprocess.run(command, capture_output=True, text=True, encoding="utf-8", errors="replace", check=False)
    return done.returncode, done.stdout, done.stderr


def run_bytes(command):
    """Runs a tool and returns (exit status, stdout as bytes, stderr as text)."""
    done = subprocess.run(command, capture_output=True, check=False)
    return done.returncode, done.stdout, done.stderr.decode("utf-8", "replace")


def info(number, text):
    """A reading, for a person tuning a threshold: it judges nothing."""
    print("INFO %d: %s" % (number, text))


def last_line(err):
    return (err.strip().splitlines() or ["no message"])[-1]


def pcm(path):
    """The audio as mono float32 at 48 kHz, on the file's own timeline.

    ffmpeg applies the file's edit lists. A phone's audio often starts after its
    video (0.305 s on the second recording): raw output would drop that gap, so
    it is filled with silence, as the product does.
    """
    status, out, err = run_bytes(
        ["ffmpeg", "-v", "error", "-nostdin", "-i", path, "-map", "0:a:0", "-ac", "1", "-ar", str(AUDIO_RATE), "-af", "aresample=async=1:first_pts=0", "-f", "f32le", "-"]
    )
    if status != 0:
        raise Failed("ffmpeg cannot read the audio: " + last_line(err))
    return np.frombuffer(out, dtype="<f4")


def grey(path, filt, width, height):
    """The frames that come out of `filt`, as an array of (frame, row, column) bytes."""
    status, out, err = run_bytes(
        ["ffmpeg", "-v", "error", "-nostdin", "-i", path, "-map", "0:v:0", "-vf", filt + ",format=gray", "-fps_mode", "passthrough", "-f", "rawvideo", "-"]
    )
    if status != 0:
        raise Failed("ffmpeg cannot read the frames: " + last_line(err))
    return np.frombuffer(out, dtype=np.uint8).reshape(-1, height, width)


def probe(path):
    """What ffprobe says of the container and its streams."""
    status, out, err = run(["ffprobe", "-v", "error", "-print_format", "json", "-show_format", "-show_streams", path])
    if status != 0:
        raise Failed("ffprobe cannot read the file: " + (err.strip().splitlines() or ["no message"])[-1])
    return json.loads(out)


def top_level_boxes(path):
    """The top-level boxes of an ISO base media file, as (type, offset), in file order."""
    boxes = []
    with open(path, "rb") as file:
        file.seek(0, 2)
        end = file.tell()
        offset = 0
        while offset + 8 <= end:
            file.seek(offset)
            size, kind = struct.unpack(">I4s", file.read(8))
            if size == 1:
                large = file.read(8)
                if len(large) < 8:
                    break
                size = struct.unpack(">Q", large)[0]
                minimum = 16
            elif size == 0:
                size = end - offset
                minimum = 8
            else:
                minimum = 8
            if size < minimum:
                break
            boxes.append((kind.decode("latin-1"), offset))
            offset += size
    return boxes


def only_stream(info, kind):
    streams = [s for s in info.get("streams", []) if s.get("codec_type") == kind]
    if len(streams) != 1:
        raise Failed("%d %s streams, expected 1" % (len(streams), kind))
    return streams[0]


def seconds(stream, what):
    value = stream.get("duration")
    if value is None:
        raise Failed("the %s stream has no duration" % what)
    return Fraction(value)


def check_1(path, info, _args):
    """MP4 with exactly one video and one audio stream; moov before mdat."""
    names = info.get("format", {}).get("format_name", "").split(",")
    if "mp4" not in names:
        raise Failed("the container is %s, not MP4" % ",".join(names))
    only_stream(info, "video")
    only_stream(info, "audio")
    others = [s.get("codec_type") for s in info["streams"] if s.get("codec_type") not in ("video", "audio")]
    if others:
        raise Failed("a stream that is neither video nor audio: " + ", ".join(str(o) for o in others))
    offsets = {}
    for kind, offset in top_level_boxes(path):
        offsets.setdefault(kind, offset)
    if "moov" not in offsets or "mdat" not in offsets:
        raise Failed("no moov box or no mdat box at the top level")
    if offsets["moov"] > offsets["mdat"]:
        raise Failed("moov after mdat: moov at byte %d, mdat at byte %d" % (offsets["moov"], offsets["mdat"]))


def check_2(_path, info, args):
    """H.264, yuv420p, square pixels, no rotation, the profile's size."""
    video = only_stream(info, "video")
    faults = []
    if video.get("codec_name") != "h264":
        faults.append("the video codec is %s, not h264" % video.get("codec_name"))
    if video.get("pix_fmt") != "yuv420p":
        faults.append("the pixel format is %s, not yuv420p" % video.get("pix_fmt"))
    aspect = video.get("sample_aspect_ratio")
    # 0:1 is how ffprobe writes "not stated".
    if aspect not in (None, "1:1", "0:1", "N/A"):
        faults.append("the pixels are not square: sample aspect ratio %s" % aspect)
    if "rotate" in {key.lower() for key in video.get("tags", {})}:
        faults.append("a rotation tag")
    for side in video.get("side_data_list", []):
        if "display matrix" in str(side.get("side_data_type", "")).lower() or "rotation" in side:
            faults.append("a display matrix")
    size = (video.get("width"), video.get("height"))
    expected = PROFILE_SIZES[args.profile]
    if size != expected:
        faults.append("the size is %sx%s, expected %dx%d for %s" % (size[0], size[1], expected[0], expected[1], args.profile))
    if faults:
        raise Failed("; ".join(faults))


def check_3(path, info, args):
    """Every frame 1/30 s after the one before it; as many frames as the duration has."""
    video = only_stream(info, "video")
    time_base = Fraction(video.get("time_base", "0/1"))
    if time_base <= 0:
        raise Failed("the video stream has no time base")
    command = ["ffprobe", "-v", "error", "-select_streams", "v:0", "-show_entries", "frame=pts", "-print_format", "json", path]
    status, out, err = run(command)
    if status != 0:
        raise Failed("ffprobe cannot read the frames: " + (err.strip().splitlines() or ["no message"])[-1])
    stamps = [frame.get("pts") for frame in json.loads(out).get("frames", [])]
    if any(not isinstance(stamp, int) for stamp in stamps):
        raise Failed("a frame without a presentation time")
    step = 1 / FRAME_RATE
    for index in range(1, len(stamps)):
        delta = (stamps[index] - stamps[index - 1]) * time_base
        if delta != step:
            raise Failed("frame %d comes %s s after frame %d, not 1/30 s" % (index, delta, index - 1))
    expected = -(-args.expected_duration_ms * 30 // 1000)
    if len(stamps) != expected:
        raise Failed("%d frames, expected %d" % (len(stamps), expected))


def check_4(_path, info, _args):
    """AAC-LC, 48,000 Hz, 2 channels."""
    audio = only_stream(info, "audio")
    found = (audio.get("codec_name"), audio.get("profile"), audio.get("sample_rate"), audio.get("channels"))
    if found != ("aac", "LC", "48000", 2):
        raise Failed("the audio is %s %s at %s Hz with %s channels, expected aac LC at 48000 Hz with 2" % found)


def check_5(_path, info, args):
    """The video as long as the source, to a frame; the audio as long as the video, to an AAC frame."""
    video = seconds(only_stream(info, "video"), "video") * 1000
    audio = seconds(only_stream(info, "audio"), "audio") * 1000
    faults = []
    off = abs(video - args.expected_duration_ms)
    if off > VIDEO_TOLERANCE_MS:
        faults.append("the video is %.3f ms long, %.3f ms from the expected %d" % (video, off, args.expected_duration_ms))
    apart = abs(audio - video)
    if apart > AUDIO_TOLERANCE_MS:
        faults.append("the audio is %.3f ms long and the video %.3f ms: %.3f ms apart" % (audio, video, apart))
    if faults:
        raise Failed("; ".join(faults))


def check_6(path, _info, _args):
    """Both streams decode from start to end without an error."""
    status, _out, err = run(["ffmpeg", "-v", "error", "-nostdin", "-i", path, "-f", "null", "-"])
    lines = [line for line in err.splitlines() if line.strip()]
    if status != 0 or lines:
        first = lines[0] if lines else "exit status %d" % status
        raise Failed("decoding reports %d error line(s); the first: %s" % (len(lines), first))


def check_7(path, _info, _args):
    """Integrated loudness, as ffmpeg reads the stereo stream, within 1 LU of -14 LUFS."""
    status, _out, err = run(["ffmpeg", "-nostdin", "-i", path, "-map", "0:a:0", "-af", "ebur128=peak=none", "-f", "null", "-"])
    readings = [line.split("I:")[1] for line in err.splitlines() if line.strip().startswith("I:")]
    if status != 0 or not readings:
        raise Failed("ffmpeg gave no integrated loudness")
    lufs = float(readings[-1].split()[0])
    info(7, "integrated %.1f LUFS" % lufs)
    if abs(lufs - LOUDNESS_TARGET) > LOUDNESS_TOLERANCE:
        raise Failed("%.1f LUFS, not within %.0f LU of %.0f" % (lufs, LOUDNESS_TOLERANCE, LOUDNESS_TARGET))


def envelope(samples):
    """RMS over each hop of 10 ms: linear, so that speech outweighs the room tone the chain removes."""
    hops = len(samples) // ENVELOPE_HOP_SAMPLES
    frames = samples[: hops * ENVELOPE_HOP_SAMPLES].reshape(hops, ENVELOPE_HOP_SAMPLES)
    return np.sqrt(np.mean(frames.astype(np.float64) ** 2, axis=1))


def correlation(a, b):
    """Pearson's correlation; 0 when either side does not vary."""
    a, b = a - a.mean(), b - b.mean()
    scale = np.sqrt((a * a).sum() * (b * b).sum())
    return float((a * b).sum() / scale) if scale > 0 else 0.0


def best_lag(ours, source, start, stop):
    """Where `source` lies against `ours[start:stop]`, in hops with a fraction, and how well it fits there."""
    scores = []
    for lag in range(-SYNC_LAG_HOPS, SYNC_LAG_HOPS + 1):
        # Only what both have: at a lag past either end, the window is shorter.
        first = max(start, -lag)
        last = min(stop, len(source) - lag)
        scores.append(correlation(ours[first:last], source[first + lag : last + lag]) if last - first > 1 else -1.0)
    at = int(np.argmax(scores))
    fraction = 0.0
    if 0 < at < len(scores) - 1:
        before, peak, after = scores[at - 1], scores[at], scores[at + 1]
        bend = before - 2 * peak + after
        if bend < 0:
            fraction = (before - after) / (2 * bend)
    return at - SYNC_LAG_HOPS + fraction, scores[at]


def check_audio_sync(path, source):
    ours, theirs = envelope(pcm(path)), envelope(pcm(source))
    hops = SYNC_WINDOW_MS // HOP_MS
    windows = [(0, len(ours))] if len(ours) <= 2 * hops else [(0, hops), (len(ours) - hops, len(ours))]
    faults = []
    for start, stop in windows:
        at_s = start * HOP_MS / 1000
        part = theirs[start:stop]
        loudest = float(part.max()) if len(part) else 0.0
        if loudest <= 0 or 20 * np.log10(loudest) < SILENT_WINDOW_DBFS:
            info(8, "window at %.1f s is silent in the source, not measured" % at_s)
            continue
        lag, fit = best_lag(ours, theirs, start, stop)
        offset = lag * HOP_MS
        info(8, "window at %.1f s: audio offset %.1f ms, correlation %.2f" % (at_s, offset, fit))
        if abs(offset) > SYNC_LIMIT_MS:
            faults.append("the audio at %.1f s is %.1f ms from the source's" % (at_s, offset))
        elif fit < MIN_ENVELOPE_CORR:
            faults.append("the audio at %.1f s does not follow the source's: correlation %.2f" % (at_s, fit))
    return faults


def brightness(path, filt):
    """The mean brightness of each frame, from a picture of 16 x 16."""
    return grey(path, filt, 16, 16).mean(axis=(1, 2))


def flashes(means):
    """The first frame of each run of white frames."""
    found = []
    half = FLASH_SURROUND // 2
    for index, mean in enumerate(means):
        around = means[max(0, index - half) : index + half + 1]
        if mean >= FLASH_MIN and mean - np.median(around) >= FLASH_ABOVE and (not found or index - found[-1] > 1):
            found.append(index)
    return found


def check_video_sync(path, source):
    ours = flashes(brightness(path, "scale=16:16"))
    # A source may be of variable frame rate: it is read at the export's.
    theirs = flashes(brightness(source, "fps=30,scale=16:16:flags=area"))
    if not theirs and not ours:
        info(8, "no white frame in the source, video offset not compared")
        return []
    if len(ours) != len(theirs):
        return ["%d white frames, and the source has %d" % (len(ours), len(theirs))]
    largest = max(abs(a - b) for a, b in zip(ours, theirs))
    info(8, "%d flashes compared, largest offset %d frames" % (len(theirs), largest))
    return [] if largest <= 1 else ["a white frame is %d frames from the source's" % largest]


def check_8(path, _info, args):
    """Audio within 20 ms of the source over the first and the last 5 s; white frames within one frame."""
    if args.source is None:
        info(8, "no --source, not measured")
        return
    faults = check_audio_sync(path, args.source) + check_video_sync(path, args.source)
    if faults:
        raise Failed("; ".join(faults))


def high_pass(picture):
    """`picture` less a box blur of itself: a still, smooth background must not count."""
    margin = HIGH_PASS_SIZE // 2
    padded = np.pad(picture.astype(np.float64), margin + 1, mode="edge")
    total = padded.cumsum(axis=0).cumsum(axis=1)
    size = HIGH_PASS_SIZE
    blurred = (total[size:, size:] - total[:-size, size:] - total[size:, :-size] + total[:-size, :-size]) / (size * size)
    return picture - blurred[: picture.shape[0], : picture.shape[1]]


def check_9(path, _info, args):
    """The watermark's pattern is in what stays still (free), or is not (creator)."""
    if not MASK.exists():
        raise Failed("the mask is missing: " + MASK.name)
    mask = grey(str(MASK), "scale=%d:%d" % (MASK_WIDTH, MASK_HEIGHT), MASK_WIDTH, MASK_HEIGHT)[0]
    rows, columns = np.nonzero(mask != MASK_BACKGROUND)
    top, bottom = max(rows.min() - MASK_MARGIN, 0), min(rows.max() + MASK_MARGIN + 1, MASK_HEIGHT)
    left, right = max(columns.min() - MASK_MARGIN, 0), min(columns.max() + MASK_MARGIN + 1, MASK_WIDTH)

    means = brightness(path, "scale=16:16")
    white = set(flashes(means))
    plain = [index for index in range(len(means)) if index not in white]
    picks = sorted({plain[round(i * (len(plain) - 1) / (WATERMARK_FRAMES - 1))] for i in range(WATERMARK_FRAMES)})
    chosen = "+".join("eq(n\\,%d)" % index for index in picks)
    frames = grey(path, "select='%s',scale=%d:%d:flags=area" % (chosen, MASK_WIDTH, MASK_HEIGHT), MASK_WIDTH, MASK_HEIGHT)
    still = frames.mean(axis=0)

    box = (slice(top, bottom), slice(left, right))
    fit = correlation(high_pass(still[box]).ravel(), high_pass(mask[box].astype(np.float64)).ravel())
    info(9, "watermark correlation %.2f" % fit)
    if args.profile == "free" and fit < WATERMARK_FREE_MIN:
        raise Failed("no watermark: correlation %.2f, under %.1f" % (fit, WATERMARK_FREE_MIN))
    if args.profile == "creator" and fit > WATERMARK_CREATOR_MAX:
        raise Failed("a watermark: correlation %.2f, over %.1f" % (fit, WATERMARK_CREATOR_MAX))


def check_10(path, info_, args):
    """The video's bitrate over the whole clip is at most 1.5 times the profile's target."""
    status, out, err = run(["ffprobe", "-v", "error", "-select_streams", "v:0", "-show_entries", "packet=size", "-of", "csv=p=0", path])
    if status != 0:
        raise Failed("ffprobe cannot read the packets: " + last_line(err))
    total = sum(int(line.strip().strip(",")) for line in out.splitlines() if line.strip())
    rate = total * 8 / float(seconds(only_stream(info_, "video"), "video"))
    info(10, "%.1f Mb/s" % (rate / 1_000_000))
    limit = BITRATE_LIMIT[args.profile]
    if rate > limit:
        raise Failed("%.1f Mb/s, over %.1f for %s" % (rate / 1_000_000, limit / 1_000_000, args.profile))


def tags_of(probed):
    """Every tag, at format and stream level, as (where, key in lower case, value)."""
    found = [("format", key.lower(), value) for key, value in probed.get("format", {}).get("tags", {}).items()]
    for index, stream in enumerate(probed.get("streams", [])):
        found += [("stream %d" % index, key.lower(), value) for key, value in stream.get("tags", {}).items()]
    return found


def has_place_bytes(path):
    """Whether the bytes of a location atom are anywhere in the file: it may be 92 MB, so it is read in pieces."""
    with open(path, "rb") as file:
        tail = b""
        while True:
            piece = file.read(READ_PIECE)
            if not piece:
                return None
            for mark in PLACE_BYTES:
                if mark in tail + piece:
                    return mark
            tail = piece[-READ_OVERLAP:]


def check_11(path, info_, args):
    """No title, comment, location or tag from the source; no creation time but the epoch."""
    faults = []
    tags = tags_of(info_)
    for where, key, value in tags:
        if any(place in key for place in PLACE_KEYS):
            faults.append("%s: a location tag, %s" % (where, key))
        if key in TEXT_KEYS and value.strip():
            faults.append("%s: a %s tag" % (where, key))
        if key == "creation_time" and not value.startswith(EPOCH):
            faults.append("%s: a creation_time other than the epoch" % where)
    if args.source is None:
        info(11, "no --source, the tags of the source not measured")
    else:
        theirs = {(key, value) for _where, key, value in tags_of(probe(args.source))}
        for where, key, value in tags:
            if key in SOURCE_KEYS and (key, value) in theirs and not (key == "creation_time" and value.startswith(EPOCH)):
                faults.append("%s: the source's %s" % (where, key))
    mark = has_place_bytes(path)
    if mark is not None:
        faults.append("the bytes of a location atom")
    if faults:
        raise Failed("; ".join(faults))


CHECKS = [check_1, check_2, check_3, check_4, check_5, check_6, check_7, check_8, check_9, check_10, check_11]


def main(argv):
    parser = argparse.ArgumentParser(description="Checks an MP4 that Offcut exported.")
    parser.add_argument("file")
    parser.add_argument("--profile", required=True, choices=sorted(PROFILE_SIZES))
    parser.add_argument("--expected-duration-ms", required=True, type=int, dest="expected_duration_ms")
    # What checks 8 and 11 compare with. A run without it says what it did not measure.
    parser.add_argument("--source", default=None)
    args = parser.parse_args(argv)
    if args.expected_duration_ms <= 0:
        parser.error("--expected-duration-ms must be above 0")
    for tool in ("ffprobe", "ffmpeg"):
        if shutil.which(tool) is None:
            print("verify_mp4: %s is not on the PATH" % tool, file=sys.stderr)
            return 2

    try:
        info = probe(args.file)
        unreadable = None
    except (Failed, OSError, ValueError) as error:
        info, unreadable = {}, str(error)

    passed = True
    for number, check in enumerate(CHECKS, start=1):
        try:
            if unreadable is not None:
                raise Failed(unreadable)
            check(args.file, info, args)
            print("PASS %d" % number)
        except (Failed, OSError, ValueError, KeyError, IndexError) as error:
            passed = False
            print("FAIL %d: %s" % (number, error))
    return 0 if passed else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
