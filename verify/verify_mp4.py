"""An independent check of an exported MP4.

    verify_mp4.py <file> --profile free|creator --expected-duration-ms N [--source <fixture>]

N is the duration of the source clip: an export is never shorter than its
recording. One line is printed per check, "PASS <n>" or "FAIL <n>: <reason>",
then one line naming the checks this version does not make. The exit status
is 0 only when every check that is made passes; 2 when the tools are missing
or the arguments are wrong.

This program judges what the app wrote, so it shares nothing with it: it
runs the ffprobe and ffmpeg command-line tools and reads the top-level
boxes of the file itself. It uses the standard library only (V5 adds NumPy).
"""

import argparse
import json
import shutil
import struct
import subprocess
import sys
from fractions import Fraction

# What each profile's video must measure (width, height).
PROFILE_SIZES = {"creator": (1080, 1920), "free": (720, 1280)}
FRAME_RATE = Fraction(30, 1)
# How far the video's length may be from the expected one: one frame, 33.4 ms.
VIDEO_TOLERANCE_MS = Fraction(334, 10)
# How far the audio's length may be from the video's: one AAC frame of 1,024
# samples at 48 kHz, 21.33 ms, rounded up as the video's frame is.
AUDIO_TOLERANCE_MS = Fraction(2134, 100)
NOT_IMPLEMENTED = "checks 7-11 are not implemented in this version: loudness (7), sync and timeline (8), watermark (9), bitrate (10), metadata (11)"


class Failed(Exception):
    """A check that did not pass; the message is its reason."""


def run(command):
    """Runs a tool and returns (exit status, stdout, stderr), both as text."""
    done = subprocess.run(command, capture_output=True, text=True, encoding="utf-8", errors="replace", check=False)
    return done.returncode, done.stdout, done.stderr


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


CHECKS = [check_1, check_2, check_3, check_4, check_5, check_6]


def main(argv):
    parser = argparse.ArgumentParser(description="Checks an MP4 that Offcut exported.")
    parser.add_argument("file")
    parser.add_argument("--profile", required=True, choices=sorted(PROFILE_SIZES))
    parser.add_argument("--expected-duration-ms", required=True, type=int, dest="expected_duration_ms")
    # V5: checks 8 and 11 compare with the source.
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
        except (Failed, OSError, ValueError, KeyError) as error:
            passed = False
            print("FAIL %d: %s" % (number, error))
    print(NOT_IMPLEMENTED)
    return 0 if passed else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
