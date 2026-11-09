#!/bin/sh
# scripts/gen-long-fixture.sh - the 45-minute local fixtures (v6.1implementation D-430), D1 only.
#
# Writes into fixtures/.gen-local/ (git-ignored; never committed, never in CI):
#   long_45min_4k_hevc.mov     3840x2160 HEVC Main (hvc1), 4 Mbps, 30 fps, about 1.5 GB
#   long_45min_1080p_h264.mp4  1920x1080 H.264, 16 Mbps (a phone's rate, past 4 GiB), about 5.6 GB
#   hevc_main10_2s.mov         2 s of HEVC Main 10, for the hvcC reads
#   long_audio_16k.wav         the 45 minutes of sound alone, 16 kHz mono, for TE-18
# The sound is the reference clip's (D-64): its whole audio track, then 5 s of room tone,
# repeated to 45:00. The video is a 10 s segment copied in a loop, so the whole run takes
# about two minutes. Needs about 7.5 GB free. Usage: sh scripts/gen-long-fixture.sh
set -eu
ROOT=$(cd "$(dirname "$0")/.." && pwd)
OUT=$ROOT/fixtures/.gen-local
REF=${OFFCUT_REFERENCE_CLIP:-$ROOT/testclips/speech_scriptA_landscape_720p.mp4}
[ -f "$REF" ] || { echo "the reference clip $REF does not exist" >&2; exit 3; }
git -C "$ROOT" check-ignore -q "$OUT/x" || { echo "fixtures/.gen-local/ is not git-ignored" >&2; exit 2; }
mkdir -p "$OUT/.tmp"
T=$OUT/.tmp
G="-y -hide_banner -loglevel error -nostdin"
D=2700

# One loop period: the reference clip's whole sound (74.517 s as decoded), then 5 s of room tone.
ffmpeg $G -i "$REF" -vn -ac 2 -ar 48000 -c:a pcm_s16le "$T/ref.wav"
ffmpeg $G -f lavfi -i "anoisesrc=color=pink:amplitude=0.001:sample_rate=48000:seed=7:duration=5" -ac 2 \
  -c:a pcm_s16le "$T/room5.wav"
ffmpeg $G -i "$T/ref.wav" -i "$T/room5.wav" -filter_complex "[0:a][1:a]concat=n=2:v=0:a=1[a]" -map "[a]" \
  -c:a pcm_s16le "$T/period.wav"
# 45 minutes of it, AAC-LC 128 kbps 48 kHz stereo (a phone's track).
ffmpeg $G -stream_loop -1 -i "$T/period.wav" -t $D -c:a aac -b:a 128k -ar 48000 -ac 2 "$T/long_audio.m4a"

# 4K HEVC, 4 Mbps: a 10 s segment, then copied in a loop.
ffmpeg $G -f lavfi -i "testsrc2=size=3840x2160:rate=30:duration=10" -c:v libx265 -preset ultrafast -b:v 4M \
  -x265-params "keyint=30:min-keyint=30:log-level=error" -pix_fmt yuv420p -tag:v hvc1 "$T/seg4k.mp4"
ffmpeg $G -stream_loop -1 -i "$T/seg4k.mp4" -i "$T/long_audio.m4a" -map 0:v -map 1:a -c copy -t $D -tag:v hvc1 \
  -movflags +faststart "$OUT/long_45min_4k_hevc.mov"

# 1080p H.264 at about 16 Mbps, so the file passes 4 GiB.
ffmpeg $G -f lavfi -i "testsrc2=size=1920x1080:rate=30:duration=10,noise=alls=24:allf=t" -c:v libx264 \
  -preset ultrafast -b:v 16M -minrate 16M -maxrate 16M -bufsize 16M -g 30 -bf 0 -pix_fmt yuv420p "$T/seg1080.mp4"
ffmpeg $G -stream_loop -1 -i "$T/seg1080.mp4" -i "$T/long_audio.m4a" -map 0:v -map 1:a -c copy -t $D \
  -movflags +faststart "$OUT/long_45min_1080p_h264.mp4"

# 2 s of 10-bit HEVC (Main 10, hvc1), as a phone writes it.
ffmpeg $G -f lavfi -i "testsrc2=size=1920x1080:rate=30:duration=2" \
  -f lavfi -i "sine=frequency=440:sample_rate=48000:duration=2" -c:v libx265 -preset ultrafast \
  -x265-params "log-level=error" -pix_fmt yuv420p10le -tag:v hvc1 -c:a aac -b:a 128k -movflags +faststart \
  "$OUT/hevc_main10_2s.mov"

# The sound alone, 16 kHz mono, for TE-18's windows.
ffmpeg $G -i "$T/long_audio.m4a" -ac 1 -ar 16000 -c:a pcm_s16le "$OUT/long_audio_16k.wav"
rm -rf "$T"
echo "gen-long-fixture: done, in $OUT"
