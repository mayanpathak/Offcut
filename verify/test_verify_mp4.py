"""Proves that each check of verify_mp4.py tells a good file from a bad one.

    python verify/test_verify_mp4.py [TestVerifier.<case>]

A hosted runner cannot make a real export, so without this CI would never run
checks 7 to 11 (v5implementation D-209). It makes one good file from a generated
fixture with ffmpeg, then copies with one thing wrong, and each bad file must
fail its own check and pass every other: a file that fails for another reason
proves nothing. The verifier is run as a program, as a test runs it.

`pnpm fixtures` must have run. Everything is written to one temporary directory,
which is deleted however the run ends.
"""

import os
import re
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
VERIFIER = HERE / "verify_mp4.py"
MASK = HERE / "watermark_mask_720.png"
FIXTURES = Path(os.environ.get("OFFCUT_FIXTURES_DIR", HERE.parent / "fixtures/.gen"))
SOURCE = FIXTURES / "speech/speech_20s_long_pause.mp4"
DURATION_MS = 20_000
WHITE_FRAMES = 2
ALL_CHECKS = range(1, 12)

TARGET_LUFS = -14.0
SECRET_TITLE = "aTitleThatMustNotBePrinted"
SECRET_PLACE = "+27.1000+78.0000/"
SECRET_TIME = "2024-02-03T04:05:06.000000Z"

FIT_720 = "scale=720:1280:force_original_aspect_ratio=increase:out_range=tv,crop=720:1280,setsar=1"
FIT_1080 = "scale=1080:1920:force_original_aspect_ratio=increase:out_range=tv,crop=1080:1920,setsar=1"
# The mask's text as a watermark: where the mask is not its background, 60% of it over the frame.
MASK_LAYER = "[1:v]format=gray,split[colour][shape];[shape]geq=lum='if(eq(p(X,Y),128),0,153)'[alpha];[colour][alpha]alphamerge"

VIDEO = ["-c:v", "libx264", "-preset", "ultrafast", "-bf", "0", "-g", "30", "-r", "30", "-pix_fmt", "yuv420p"]
AUDIO = ["-c:a", "aac", "-b:a", "128k", "-ar", "48000", "-ac", "2"]
# No tag of the source and none of ffmpeg's: what the product writes is what these files must look like.
CLEAN = ["-map_metadata", "-1", "-fflags", "+bitexact", "-flags:v", "+bitexact", "-flags:a", "+bitexact", "-metadata:s:v:0", "encoder=test", "-movflags", "+faststart"]
COPY_VIDEO = ["-c:v", "copy", "-fflags", "+bitexact", "-flags:a", "+bitexact", "-movflags", "+faststart"]
COPY_ALL = ["-c", "copy", "-fflags", "+bitexact", "-movflags", "+faststart"]


def ffmpeg(*arguments):
    done = subprocess.run(["ffmpeg", "-v", "error", "-nostdin", "-y", *map(str, arguments)], capture_output=True, text=True, encoding="utf-8", errors="replace", check=False)
    if done.returncode != 0:
        raise RuntimeError("ffmpeg failed: " + (done.stderr.strip().splitlines() or ["no message"])[-1])


def integrated_lufs(path):
    done = subprocess.run(
        ["ffmpeg", "-nostdin", "-i", str(path), "-map", "0:a:0", "-af", "ebur128=peak=none", "-f", "null", "-"],
        capture_output=True, text=True, encoding="utf-8", errors="replace", check=True,
    )
    return float(re.findall(r"I:\s+(-?[\d.]+) LUFS", done.stderr)[-1])


class TestVerifier(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if not SOURCE.exists():
            raise unittest.SkipTest("the generated fixtures are missing: run pnpm fixtures") if os.environ.get("CI") is None else RuntimeError("the generated fixtures are missing")
        cls.dir = Path(tempfile.mkdtemp(prefix="offcut-verify-"))
        cls.addClassCleanup(shutil.rmtree, cls.dir, ignore_errors=True)
        cls.gain = "volume=%.2fdB" % (TARGET_LUFS - integrated_lufs(SOURCE))
        cls.plain = cls.make("plain.mp4", "-i", SOURCE, "-vf", FIT_720, "-af", cls.gain, *VIDEO, *AUDIO, *CLEAN)
        cls.creator = cls.make("creator.mp4", "-i", SOURCE, "-vf", FIT_1080, "-af", cls.gain, *VIDEO, *AUDIO, *CLEAN)
        cls.free = cls.make(
            "free.mp4", "-i", SOURCE, "-i", MASK,
            "-filter_complex", "[0:v]%s[frame];%s[mark];[frame][mark]overlay=format=yuv420[out]" % (FIT_720, MASK_LAYER),
            "-map", "[out]", "-map", "0:a:0", "-af", cls.gain, *VIDEO, *AUDIO, *CLEAN,
        )

    @classmethod
    def make(cls, name, *arguments):
        out = cls.dir / name
        ffmpeg(*arguments, out)
        return out

    def verify(self, path, profile, expected=DURATION_MS, source=SOURCE):
        done = subprocess.run(
            [sys.executable, str(VERIFIER), str(path), "--profile", profile, "--expected-duration-ms", str(expected), "--source", str(source)],
            capture_output=True, text=True, encoding="utf-8", errors="replace", check=False,
        )
        return done.returncode, done.stdout + done.stderr

    def assert_verdict(self, output, status, failing=()):
        """Each check in `failing` fails and every other passes: the file fails for its own reason."""
        for number in ALL_CHECKS:
            line = ("FAIL %d" if number in failing else "PASS %d") % number
            self.assertIn(line, output)
        self.assertEqual(status, 1 if failing else 0, output)

    # --- Good files -----------------------------------------------------------

    def test_a_good_free_file_passes_all_eleven(self):
        status, output = self.verify(self.free, "free")
        self.assert_verdict(output, status)
        self.assertIn("INFO 8: %d flashes compared, largest offset 0 frames" % WHITE_FRAMES, output)
        self.assertNotIn("not measured", output)

    def test_a_good_creator_file_passes_all_eleven(self):
        status, output = self.verify(self.creator, "creator")
        self.assert_verdict(output, status)
        self.assertIn("INFO 7: integrated", output)
        self.assertIn("INFO 9: watermark correlation", output)
        self.assertIn("INFO 10:", output)

    def test_without_a_source_it_says_what_it_did_not_measure(self):
        done = subprocess.run(
            [sys.executable, str(VERIFIER), str(self.creator), "--profile", "creator", "--expected-duration-ms", str(DURATION_MS)],
            capture_output=True, text=True, encoding="utf-8", errors="replace", check=False,
        )
        self.assert_verdict(done.stdout, done.returncode)
        self.assertIn("INFO 8: no --source, not measured", done.stdout)
        self.assertIn("INFO 11: no --source", done.stdout)

    # --- Check 7 --------------------------------------------------------------

    def test_too_quiet_fails_loudness(self):
        out = self.make("quiet.mp4", "-i", self.free, "-af", "volume=-6dB", *COPY_VIDEO, *AUDIO)
        self.assert_verdict(self.verify(out, "free")[1], 1, failing=[7])

    def test_a_mono_voice_doubled_without_the_fix_fails_loudness(self):
        # What every export read before D-204: 3 LU over the target, the "before" of known issue 1.
        out = self.make("loud.mp4", "-i", self.free, "-af", "volume=3dB", *COPY_VIDEO, *AUDIO)
        status, output = self.verify(out, "free")
        self.assert_verdict(output, status, failing=[7])

    # --- Check 8 --------------------------------------------------------------

    def test_delayed_audio_fails_sync(self):
        out = self.make("late.mp4", "-i", self.free, "-af", "adelay=100:all=1,atrim=0:20", *COPY_VIDEO, *AUDIO)
        self.assert_verdict(self.verify(out, "free")[1], 1, failing=[8])

    def test_a_shortened_pause_fails_the_timeline_though_its_duration_is_given(self):
        out = self.make(
            "shortened.mp4", "-i", self.free,
            "-filter_complex",
            "[0:v]trim=0:7,setpts=PTS-STARTPTS[a];[0:v]trim=8:20,setpts=PTS-STARTPTS[b];[a][b]concat=n=2:v=1:a=0[v];"
            "[0:a]atrim=0:7,asetpts=PTS-STARTPTS[c];[0:a]atrim=8:20,asetpts=PTS-STARTPTS[d];[c][d]concat=n=2:v=0:a=1[s]",
            "-map", "[v]", "-map", "[s]", *VIDEO, *AUDIO, *CLEAN,
        )
        status, output = self.verify(out, "free", expected=19_000)
        self.assert_verdict(output, status, failing=[8])

    def test_a_source_whose_audio_starts_late_is_read_on_its_own_timeline(self):
        # A phone's recording: the audio starts after the video. An export that keeps the gap is in sync.
        source = self.make("late_source.mp4", "-i", SOURCE, "-itsoffset", "0.3", "-i", SOURCE, "-map", "0:v:0", "-map", "1:a:0", *COPY_ALL)
        out = self.make("kept_gap.mp4", "-i", self.free, "-af", "adelay=300:all=1,atrim=0:20", *COPY_VIDEO, *AUDIO)
        status, output = self.verify(out, "free", source=source)
        self.assert_verdict(output, status)

    def test_a_white_frame_two_frames_late_fails_sync(self):
        out = self.make(
            "moved.mp4", "-i", self.free,
            "-vf", "drawbox=enable='eq(n,60)':x=0:y=0:w=iw:h=ih:color=gray:t=fill,drawbox=enable='eq(n,62)':x=0:y=0:w=iw:h=ih:color=white:t=fill",
            "-c:a", "copy", *VIDEO, *CLEAN,
        )
        status, output = self.verify(out, "free")
        self.assert_verdict(output, status, failing=[8])
        self.assertIn("2 frames from the source", output)

    # --- Check 9 --------------------------------------------------------------

    def test_a_free_file_with_no_watermark_fails(self):
        status, output = self.verify(self.plain, "free")
        self.assert_verdict(output, status, failing=[9])

    def test_a_creator_file_with_the_watermark_fails(self):
        out = self.make(
            "marked.mp4", "-i", self.creator, "-i", MASK,
            "-filter_complex", "%s,scale=1080:1920[mark];[0:v][mark]overlay=format=yuv420[out]" % MASK_LAYER,
            "-map", "[out]", "-map", "0:a:0", "-c:a", "copy", *VIDEO, *CLEAN,
        )
        status, output = self.verify(out, "creator")
        self.assert_verdict(output, status, failing=[9])

    # --- Check 10 -------------------------------------------------------------

    def test_too_many_bits_fail_the_bitrate(self):
        out = self.make(
            "fat.mp4", "-i", self.free, "-c:a", "copy",
            "-b:v", "20M", "-minrate", "20M", "-maxrate", "20M", "-bufsize", "10M", "-x264-params", "nal-hrd=cbr:force-cfr=1",
            *VIDEO, *CLEAN,
        )
        status, output = self.verify(out, "free")
        self.assert_verdict(output, status, failing=[10])

    # --- Check 11 -------------------------------------------------------------

    def test_a_title_fails_and_its_value_is_not_printed(self):
        out = self.make("titled.mp4", "-i", self.free, *COPY_ALL, "-metadata", "title=" + SECRET_TITLE)
        status, output = self.verify(out, "free")
        self.assert_verdict(output, status, failing=[11])
        self.assertIn("title", output)
        self.assertNotIn(SECRET_TITLE, output)

    def test_a_location_fails_and_its_value_is_not_printed(self):
        out = self.make("placed.mp4", "-i", self.free, *COPY_ALL, "-metadata", "location=" + SECRET_PLACE)
        status, output = self.verify(out, "free")
        self.assert_verdict(output, status, failing=[11])
        self.assertIn("location", output)
        self.assertNotIn(SECRET_PLACE, output)

    def test_the_sources_creation_time_fails_and_its_value_is_not_printed(self):
        source = self.make("dated_source.mp4", "-i", SOURCE, *COPY_ALL, "-metadata", "creation_time=" + SECRET_TIME)
        out = self.make("dated.mp4", "-i", self.free, *COPY_ALL, "-metadata", "creation_time=" + SECRET_TIME)
        status, output = self.verify(out, "free", source=source)
        self.assert_verdict(output, status, failing=[11])
        self.assertIn("the source's creation_time", output)
        self.assertNotIn(SECRET_TIME, output)

    # --- Exit status ----------------------------------------------------------

    def test_without_numpy_it_exits_2_and_says_what_to_run(self):
        stub = self.dir / "no_numpy"
        stub.mkdir(exist_ok=True)
        (stub / "numpy.py").write_text('raise ImportError("hidden by the test")\n', encoding="utf-8")
        done = subprocess.run(
            [sys.executable, str(VERIFIER), str(self.free), "--profile", "free", "--expected-duration-ms", str(DURATION_MS)],
            capture_output=True, text=True, encoding="utf-8", errors="replace", check=False,
            env={**os.environ, "PYTHONPATH": str(stub)},
        )
        self.assertEqual(done.returncode, 2)
        self.assertIn("pip install -r verify/requirements.txt", done.stderr)


if __name__ == "__main__":
    unittest.main()
