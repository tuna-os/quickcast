#!/usr/bin/env python3
"""Verify the encoded synthetic fixture, including actual decoded overlay pixels."""
import json
import subprocess

path = "test-artifacts/composite.mp4"
info = json.loads(subprocess.check_output([
    "ffprobe", "-v", "error", "-show_streams", "-show_format", "-of", "json", path
]))
assert {stream["codec_name"] for stream in info["streams"]} == {"h264", "aac"}
assert 1.9 < float(info["format"]["duration"]) < 2.2
raw = subprocess.check_output([
    "ffmpeg", "-v", "error", "-ss", "0.5", "-i", path,
    "-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgb24", "-"
])
assert len(raw) == 640 * 360 * 3
for label, x, y in [("screen", 100, 100), ("webcam", 570, 285), ("transparent corner", 507, 227), ("transparent corner", 627, 347)]:
    offset = (y * 640 + x) * 3
    red, green, blue = raw[offset:offset + 3]
    if label != "webcam":
        assert blue > 180 and red < 50, (label, red, green, blue)
    else:
        assert red > 180 and blue < 50, (label, red, green, blue)
print("Verified H.264/AAC, duration, screen pixels and circular webcam overlay with transparent corners")
