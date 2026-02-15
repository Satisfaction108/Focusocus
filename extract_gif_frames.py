#!/usr/bin/env python3
from PIL import Image
import os

# Create output directory
os.makedirs("src-tauri/resources/frames", exist_ok=True)

# All GIFs are 512x512, we keep them at full quality (no resize here)
# The Rust code will scale them down for display

# Extract MittenSpawnEffect.gif frames
spawn_gif = Image.open('assets/MittenSpawnEffect.gif')
print(f"MittenSpawnEffect.gif: {spawn_gif.size[0]}x{spawn_gif.size[1]}, {spawn_gif.n_frames} frames, {spawn_gif.info.get('duration', 100)}ms per frame")

for i in range(spawn_gif.n_frames):
    spawn_gif.seek(i)
    frame = spawn_gif.convert('RGBA')
    output_path = f"src-tauri/resources/frames/spawn_frame{i+1}.png"
    frame.save(output_path, 'PNG', optimize=False)

print(f"✅ Extracted {spawn_gif.n_frames} spawn frames")

# Extract MittenYawning.gif frames
yawn_gif = Image.open('assets/MittenYawning.gif')
print(f"MittenYawning.gif: {yawn_gif.size[0]}x{yawn_gif.size[1]}, {yawn_gif.n_frames} frames, {yawn_gif.info.get('duration', 100)}ms per frame")

for i in range(yawn_gif.n_frames):
    yawn_gif.seek(i)
    frame = yawn_gif.convert('RGBA')
    output_path = f"src-tauri/resources/frames/yawn_frame{i+1}.png"
    frame.save(output_path, 'PNG', optimize=False)

print(f"✅ Extracted {yawn_gif.n_frames} yawn frames")

# Extract MittenIdle.gif frames
idle_gif = Image.open('assets/MittenIdle.gif')
print(f"MittenIdle.gif: {idle_gif.size[0]}x{idle_gif.size[1]}, {idle_gif.n_frames} frames, {idle_gif.info.get('duration', 100)}ms per frame")

for i in range(idle_gif.n_frames):
    idle_gif.seek(i)
    frame = idle_gif.convert('RGBA')
    output_path = f"src-tauri/resources/frames/idle_frame{i+1}.png"
    frame.save(output_path, 'PNG', optimize=False)

print(f"✅ Extracted {idle_gif.n_frames} idle frames")
print(f"\nAnimation: Spawn ({spawn_gif.n_frames}) → Yawn ({yawn_gif.n_frames}) → Idle loop ({idle_gif.n_frames})")

