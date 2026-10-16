#!/bin/sh
# fixtures/gen_fixtures.sh - generated and derived fixtures (v3implementation §17, D-76, D-77, D-100, D-111)
#
# Builds the 25 fixtures of fixtures/manifest.json into fixtures/.gen/ (git-ignored),
# the PCM of the five speech fixtures, and STAMP last. Run by `pnpm fixtures`.
# Exit codes: 2 tool version, 3 reference clip, 4 recipe failure, 5 probe mismatch.
set -eu
ROOT=$(cd "$(dirname "$0")/.." && pwd)
OUT=${OFFCUT_FIXTURES_DIR:-$ROOT/fixtures/.gen}
MANIFEST=$ROOT/fixtures/manifest.json
SCRIPT=$ROOT/fixtures/gen_fixtures.sh
hash() { sha256sum "$1" | cut -d' ' -f1; }

# Step 1: ffmpeg 9.0 (Windows prints "9.0.2-essentials...", Linux static builds print "n9.0.2...").
ver=$(ffmpeg -hide_banner -version | head -1)
case "$ver" in
  "ffmpeg version 9.0"* | "ffmpeg version n9.0"*) ;;
  *) echo "ffmpeg 9.0 is needed, found: $ver" >&2; exit 2 ;;
esac
FFMPEG_VERSION=$(echo "$ver" | cut -d' ' -f3 | sed 's/^n//')

# Step 2: the reference clip (D-64, D-76). Never another clip.
SAMPLE_CLIP_PATH=$(sed -n 's/^export const SAMPLE_CLIP_PATH = "\([^"]*\)";$/\1/p' "$ROOT/web/src/net/asset-fetch.ts" | head -1)
[ -n "$SAMPLE_CLIP_PATH" ] || { echo "SAMPLE_CLIP_PATH was not found in web/src/net/asset-fetch.ts" >&2; exit 3; }
EXPECT_PREFIX=$(echo "$SAMPLE_CLIP_PATH" | sed -n 's/.*\.\([0-9a-f]\{16\}\)\.[^.]*$/\1/p')
[ -n "$EXPECT_PREFIX" ] || { echo "no 16-hex hash segment in $SAMPLE_CLIP_PATH" >&2; exit 3; }
CACHED=$ROOT/fixtures/.cache/$SAMPLE_CLIP_PATH
if [ -n "${OFFCUT_REFERENCE_CLIP:-}" ]; then
  REF=$OFFCUT_REFERENCE_CLIP
elif [ -f "$ROOT/testclips/speech_scriptA_landscape_720p.mp4" ]; then
  REF=$ROOT/testclips/speech_scriptA_landscape_720p.mp4
elif [ -f "$CACHED" ]; then
  REF=$CACHED
elif [ -n "${VITE_ASSET_BASE_URL:-}" ]; then
  mkdir -p "$(dirname "$CACHED")"
  curl --fail --silent --show-error --location --retry 3 --output "$CACHED.part" "$VITE_ASSET_BASE_URL/$SAMPLE_CLIP_PATH" \
    || { rm -f "$CACHED.part"; echo "the reference clip could not be downloaded from the asset host" >&2; exit 3; }
  mv "$CACHED.part" "$CACHED"
  REF=$CACHED
else
  echo "the reference clip is in neither testclips/ nor fixtures/.cache/, and VITE_ASSET_BASE_URL is not set" >&2
  exit 3
fi
[ -f "$REF" ] || { echo "the reference clip $REF does not exist" >&2; exit 3; }
REF_SHA=$(hash "$REF")
case "$REF_SHA" in
  "$EXPECT_PREFIX"*) ;;
  *) echo "the reference clip's SHA-256 does not start with $EXPECT_PREFIX: $REF" >&2; exit 3 ;;
esac

# Step 3: up to date?
names() { node -e 'const m=JSON.parse(require("fs").readFileSync(process.argv[1],"utf8")); for (const f of m.fixtures) if (process.argv[2]!=="pcm"||f.pcm===true) console.log(f.name);' "$MANIFEST" "${1:-all}"; }
COUNT=$(names | wc -l | tr -d ' ')
STAMP_TEXT="{ \"ffmpeg\": \"$FFMPEG_VERSION\", \"script_sha256\": \"$(hash "$SCRIPT")\", \"manifest_sha256\": \"$(hash "$MANIFEST")\", \"reference_sha256\": \"$REF_SHA\", \"generated\": $COUNT }"
if [ -f "$OUT/STAMP" ] && [ "$(cat "$OUT/STAMP")" = "$STAMP_TEXT" ]; then
  complete=yes
  for n in $(names); do [ -f "$OUT/$n" ] || complete=no; done
  for n in $(names pcm); do [ -f "$OUT/$n.pcm48.f32" ] || complete=no; done
  if [ "$complete" = yes ]; then echo "fixtures are up to date"; exit 0; fi
fi

# Step 4: clean.
rm -rf "$OUT"
mkdir -p "$OUT/speech" "$OUT/.tmp"
TMP=$OUT/.tmp

# Input-side flags go before the first -i; output flags after the last -i (§17.1).
# Unquoted on purpose below: no value holds a space.
GLOBAL="-y -hide_banner -loglevel error -nostdin"
OUTB="-fflags +bitexact -flags:v +bitexact -flags:a +bitexact -map_metadata -1 -map_chapters -1"
OUTF="-threads 1 $OUTB"
# The two 1080p clips only: x264 is deterministic for a fixed thread count, and one
# thread takes four minutes for the pair on D1 (v3changelog, Prompt 63).
OUTF4="-threads 4 $OUTB"
H264="-c:v libx264 -bf 0 -g 30 -pix_fmt yuv420p"
FAST="-preset veryfast"
AAC="-c:a aac -b:a 128k -ar 48000 -ac 2"
FS="-movflags +faststart"
video() { echo "-f lavfi -i testsrc2=size=$1x$2:rate=$3:duration=$4"; }
tone() { echo "-f lavfi -i sine=frequency=${2:-440}:sample_rate=48000:duration=$1"; }
src() { echo "$(video "$1" "$2" "$3" "$4") $(tone "$4")"; }
enc() { id=$1; shift; ffmpeg "$@" || { echo "recipe $id failed" >&2; exit 4; }; }

# Step 5: the synthetic fixtures of §17.2, in manifest order.
enc ok_vfr $GLOBAL $(src 1280 720 30 10) $OUTF -vf "select='not(eq(mod(n\,4)\,3))'" -fps_mode passthrough \
  $H264 $FAST $AAC $FS "$OUT/ok_vfr.mp4"
enc ok_rotated_90 $GLOBAL $(src 1280 720 30 3) $OUTF $H264 $FAST $AAC $FS "$TMP/rotated.mp4"
enc ok_rotated_90 $GLOBAL -display_rotation 90 -i "$TMP/rotated.mp4" $OUTF -c copy $FS "$OUT/ok_rotated_90.mov"
enc ok_60fps $GLOBAL $(src 1280 720 60 3) $OUTF $H264 $FAST $AAC $FS "$OUT/ok_60fps.mp4"
enc ok_90s_exact $GLOBAL $(src 320 180 30 90) $OUTF $H264 $FAST $AAC $FS "$OUT/ok_90s_exact.mp4"
enc ok_moov_at_end $GLOBAL $(src 320 180 30 3) $OUTF $H264 $FAST $AAC "$OUT/ok_moov_at_end.mp4"
enc ok_portrait_3x4 $GLOBAL $(src 1080 1440 30 3) $OUTF $H264 $FAST $AAC $FS "$OUT/ok_portrait_3x4.mp4"
enc ok_two_video_first_disabled $GLOBAL $(video 320 180 30 3) $(video 640 360 30 3) $(tone 3) $OUTF \
  -map 0:v -map 1:v -map 2:a $H264 $FAST $AAC -disposition:v:0 0 -disposition:v:1 default $FS \
  "$OUT/ok_two_video_first_disabled.mp4"
enc rej_too_long_91s $GLOBAL $(src 320 180 30 91) $OUTF $H264 $FAST $AAC $FS "$OUT/rej_too_long_91s.mp4"
enc rej_4k $GLOBAL $(src 3840 2160 30 1) $OUTF $H264 -preset ultrafast $AAC $FS "$OUT/rej_4k.mp4"
enc rej_hevc $GLOBAL $(src 1280 720 30 2) $OUTF -c:v libx265 -tag:v hvc1 -g 30 -pix_fmt yuv420p \
  -x265-params log-level=error $AAC $FS "$OUT/rej_hevc.mov"
enc rej_av1 $GLOBAL $(src 320 180 30 2) $OUTF -c:v libaom-av1 -cpu-used 8 -row-mt 0 -g 30 -pix_fmt yuv420p \
  $AAC $FS "$OUT/rej_av1.mp4"
enc rej_vp9 $GLOBAL $(src 320 180 30 2) $OUTF -c:v libvpx-vp9 -deadline realtime -g 30 -pix_fmt yuv420p \
  -c:a libopus -b:a 96k -ar 48000 -ac 2 "$OUT/rej_vp9.webm"
enc rej_no_audio $GLOBAL $(video 320 180 30 2) $OUTF $H264 $FAST -an $FS "$OUT/rej_no_audio.mp4"
enc rej_two_audio $GLOBAL $(src 320 180 30 2) $(tone 2 880) $OUTF -map 0:v -map 1:a -map 2:a $H264 $FAST $AAC $FS \
  "$OUT/rej_two_audio.mp4"
enc rej_mp3_audio $GLOBAL $(src 320 180 30 2) $OUTF $H264 $FAST -c:a libmp3lame -b:a 128k -ar 48000 -ac 2 $FS \
  "$OUT/rej_mp3_audio.mp4"
enc rej_120fps $GLOBAL $(src 320 180 120 2) $OUTF $H264 $FAST $AAC $FS "$OUT/rej_120fps.mp4"
enc rej_no_video $GLOBAL $(tone 2) $OUTF $AAC $FS "$OUT/rej_no_video.m4a"
enc rej_truncated $GLOBAL $(src 320 180 30 2) $OUTF $H264 $FAST $AAC $FS "$TMP/truncated.mp4"
head -c 1500 "$TMP/truncated.mp4" > "$OUT/rej_truncated.mp4"
enc rej_silent $GLOBAL $(video 320 180 30 5) -f lavfi -i anullsrc=channel_layout=stereo:sample_rate=48000 $OUTF \
  -t 5 $H264 $FAST $AAC $FS "$OUT/rej_silent.mp4"
enc rej_h264_444 $GLOBAL $(src 1280 720 30 2) $OUTF -c:v libx264 -profile:v high444 -pix_fmt yuv444p -bf 0 -g 30 $FAST \
  $AAC $FS "$OUT/rej_h264_444.mp4"

# Step 6: the speech fixtures derived from the reference clip (§17.3). `fps=30` comes
# first, so frame numbers count 30 fps output frames. No metadata of REF is kept (D-100).
WHITE="drawbox=x=0:y=0:w=iw:h=ih:color=white:t=fill"
enc speech_60s_portrait $GLOBAL -ss 0.960 -i "$REF" $OUTF4 -t 60 \
  -vf "fps=30,crop=ih*9/16:ih,scale=1080:1920:flags=lanczos,$WHITE:enable='eq(n\,150)+eq(n\,1650)'" \
  $H264 $FAST $AAC $FS "$OUT/speech/speech_60s_portrait.mp4"
enc speech_60s_landscape $GLOBAL -ss 0.960 -i "$REF" $OUTF4 -t 60 \
  -vf "fps=30,scale=1920:1080:flags=lanczos,$WHITE:enable='eq(n\,150)+eq(n\,1650)'" \
  $H264 $FAST $AAC $FS "$OUT/speech/speech_60s_landscape.mp4"

# Room tone: generated pink noise at about -60 dBFS, a stand-in for a recorded room (D-76).
ROOM="anoisesrc=color=pink:amplitude=0.001:sample_rate=48000:seed=7:duration=30"
AF="aformat=sample_fmts=fltp:sample_rates=48000:channel_layouts=stereo"
# A video piece of REF, by output frame number at 30 fps: from frame $2 for $3 frames.
vpiece() { echo "[$1]trim=start_frame=$2:end_frame=$(($2 + $3)),setpts=PTS-STARTPTS[$4];"; }
# An audio piece of REF, by seconds: from $2 for $3 seconds.
apiece() { echo "[$1]atrim=start=$2:duration=$3,asetpts=PTS-STARTPTS,$AF[$4];"; }
# A piece of room tone of $2 seconds.
rpiece() { echo "[$1]atrim=duration=$2,asetpts=PTS-STARTPTS,$AF[$3];"; }

# No events: REF 46.600-61.000 (sentences 12-14), frames 1398-1829. Cut by frame number:
# an input seek into the VFR source lands after a frame boundary and loses a frame.
FC="[0:v]fps=30,trim=start_frame=1398:end_frame=1830,setpts=PTS-STARTPTS,scale=1280:720[v];"
FC="$FC[0:a]atrim=start=46.600:duration=14.400,asetpts=PTS-STARTPTS,$AF[a]"
enc speech_20s_noevents $GLOBAL -i "$REF" $OUTF -filter_complex "$FC" -map "[v]" -map "[a]" \
  $H264 $FAST $AAC $FS "$OUT/speech/speech_20s_noevents.mp4"

# Long pause: REF 9.160-14.160, 5 s of room tone, REF 18.100-28.100. The video under the
# pause is REF 14.160-19.160: frames 275-574, then 543-842 of REF at 30 fps.
FC="[0:v]fps=30,split=2[v0][v1];$(vpiece v0 275 300 va)$(vpiece v1 543 300 vb)"
FC="$FC[va][vb]concat=n=2:v=1:a=0,scale=1280:720,$WHITE:enable='eq(n\,60)+eq(n\,450)'[v];"
FC="$FC[0:a]asplit=2[a0][a1];[1:a]asplit=1[r0];"
FC="$FC$(apiece a0 9.160 5.000 p1)$(rpiece r0 5.000 t1)$(apiece a1 18.100 10.000 p2)"
FC="$FC[p1][t1][p2]concat=n=3:v=0:a=1[a]"
enc speech_20s_long_pause $GLOBAL -i "$REF" -f lavfi -i "$ROOM" $OUTF -filter_complex "$FC" -map "[v]" -map "[a]" \
  $H264 $FAST $AAC $FS "$OUT/speech/speech_20s_long_pause.mp4"

# Sparse: five 1.100 s pieces of REF (from 9.160, 29.040, 46.840, 55.420, 69.120) placed at
# 0.5, 7.0, 13.5, 19.5 and 26.0 s, room tone between them. The video under each stretch is
# REF continuing from the piece placed in it; before the first piece it is the 0.5 s of REF
# before that piece. Frames at 30 fps: 260+210, 872+195, 1406+180, 1663+195, 2074+120.
FC="[0:v]fps=30,split=5[v0][v1][v2][v3][v4];"
FC="$FC$(vpiece v0 260 210 w0)$(vpiece v1 872 195 w1)$(vpiece v2 1406 180 w2)$(vpiece v3 1663 195 w3)$(vpiece v4 2074 120 w4)"
FC="$FC[w0][w1][w2][w3][w4]concat=n=5:v=1:a=0,scale=1280:720,$WHITE:enable='eq(n\,15)+eq(n\,885)'[v];"
FC="$FC[0:a]asplit=5[a0][a1][a2][a3][a4];[1:a]asplit=6[r0][r1][r2][r3][r4][r5];"
FC="$FC$(rpiece r0 0.500 t0)$(apiece a0 9.160 1.100 p0)$(rpiece r1 5.400 t1)$(apiece a1 29.040 1.100 p1)"
FC="$FC$(rpiece r2 5.400 t2)$(apiece a2 46.840 1.100 p2)$(rpiece r3 4.900 t3)$(apiece a3 55.420 1.100 p3)"
FC="$FC$(rpiece r4 5.400 t4)$(apiece a4 69.120 1.100 p4)$(rpiece r5 2.900 t5)"
FC="$FC[t0][p0][t1][p1][t2][p2][t3][p3][t4][p4][t5]concat=n=11:v=0:a=1[a]"
enc speech_30s_sparse $GLOBAL -i "$REF" -f lavfi -i "$ROOM" $OUTF -filter_complex "$FC" -map "[v]" -map "[a]" \
  $H264 $FAST $AAC $FS "$OUT/speech/speech_30s_sparse.mp4"

# Step 7: PCM of the speech fixtures, for the Rust DSP tests (Rust has no AAC decoder, D-77).
for n in $(names pcm); do
  enc "$n.pcm48.f32" $GLOBAL -i "$OUT/$n" -map 0:a:0 -ac 1 -ar 48000 -f f32le "$OUT/$n.pcm48.f32"
done
rm -rf "$TMP"

# Step 8: every probe field of the manifest, as ffprobe reports it.
node -e '
const { execFileSync } = require("child_process");
const fs = require("fs");
const [manifestPath, out] = process.argv.slice(1);
const bad = [];
for (const f of JSON.parse(fs.readFileSync(manifestPath, "utf8")).fixtures) {
  if (!fs.existsSync(`${out}/${f.name}`)) { bad.push(`${f.name}: missing`); continue; }
  if (f.probe === undefined) continue;
  const info = JSON.parse(execFileSync("ffprobe", ["-v", "error", "-show_entries",
    "stream=codec_type,codec_name,width,height,nb_frames:format=duration", "-of", "json", `${out}/${f.name}`], { encoding: "utf8" }));
  const video = info.streams.filter((s) => s.codec_type === "video");
  const audio = info.streams.filter((s) => s.codec_type === "audio");
  const seen = {
    video_codec: video[0]?.codec_name, audio_codec: audio[0]?.codec_name,
    width: video[0]?.width, height: video[0]?.height,
    frames: video[0]?.nb_frames === undefined ? undefined : Number(video[0].nb_frames),
    video_streams: video.length, audio_streams: audio.length,
    duration_ms: Math.round(Number(info.format.duration) * 1000),
  };
  for (const [key, want] of Object.entries(f.probe)) {
    const got = seen[key];
    const ok = key === "duration_ms" ? got !== undefined && Math.abs(got - want) <= 50 : got === want;
    if (!ok) bad.push(`${f.name}: ${key} is ${String(got)}, the manifest says ${String(want)}`);
  }
}
for (const line of bad) console.error(line);
process.exitCode = bad.length === 0 ? 0 : 5;
' "$MANIFEST" "$OUT" || exit 5

# Step 9: the stamp, last, so that an interrupted run leaves none (§17.4).
printf '%s\n' "$STAMP_TEXT" > "$OUT/STAMP.part"
mv "$OUT/STAMP.part" "$OUT/STAMP"
echo "generated $COUNT fixtures in $OUT"
