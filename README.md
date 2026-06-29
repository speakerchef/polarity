# Polarity

- Simple desktop app that generates customizable audio visualizations.
- Intended for people who want pretty visuals for their music just with a few clicks.
- Does not intend to replace meticulous handmade animations or visualizations.
- Uses a generator based approach where your audio itself generates the visuals; A module-based structure where each module generates a specific type of visual.

---

> [!NOTE]
> Currently in a super early prototyping stage.

### V1 MVP Roadmap (Complete)

- [x] Desktop app shell
- [x] Audio file import (Formats: WAV, MP3, OGGVORBIS)
- [x] Preview Canvas
- [x] Stereometer Generator option
  - [x] 4 modes/styles: Bipolar, Scaled bipolar, lissajous, scaled lissajous
  - [x] Controllable LPF, BPF, HPF filters
  - [x] 3-band mode and band-isolated rendering via above filters
  - [x] per-band color.
  - [x] Single color
  - [x] Tunable point density for primary and trace generators
  - [x] Editable point size
  - [x] Tunable animation scale-factor
- [x] Post-FX
  - [x] Bloom / Sparkle
- [x] Transport bar with playback controls + File path + Loop mode toggle
- [x] Fullscreen mode
- [x] Themes
- [x] Save/Load presets + default bank of presets
- [x] Timeline with waveform view
- [x] Click to skip inside timeline
- [x] Simple MP4 Export pipeline to combine audio + generated motion

### V2 Roadmap (In Progress)

- [ ] Build/packaging system to get `.app`
- [x] Simple and Advanced mode toggle
- [x] Particle / Fluid simulation motion generator
  - [x] No-gravity based fluid Canvas
  - [ ] Multiple force generator positions
    - [x] Centered
    - [ ] From sides
    - [ ] Corners
  - [x] Envelope follower based force generator
  - [x] Tunable envelope controls
  - [x] Visualize waves through particle compression and expansion
  - [x] Have dynamic velocity based color generation
  - [x] Color gradient ordering options
  - [x] Extra color options (added inversion and luminance mode)
  - [x] Vignette effect
  - [ ] Different particle counts
- [ ] Live/RT mode
- [ ] FPS counter
- [ ] Multi-generator views (not priority)
- [ ] Preferences pane with adjustable defaults
  - [ ] audio input options for live mode
  - [ ] Undo levels
  - [ ] Selectable themes
- [ ] Project save/load ability with custom format:
  - [ ] container that also stores the audio file path (or audio copied into project container)
- [ ] Undo/Redo system
- [ ] Post-FX
  - [ ] Chromatic aberration
  - [ ] Phosphor

### Author

Polarity is designed and developed by Sohan Nair, a.k.a speakerchef.

### License and Brand

The source code is licensed under GPLv3. See [LICENSE](./LICENSE).

The Polarity name, logo, and visual identity are reserved by the author. See [TRADEMARKS](./
TRADEMARKS.md)
