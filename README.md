# COSMIC Viewer
A fast, native image viewer, built with the COSMIC desktop environment in mind, but works on all DE's.

## Features
- Gallery: This is the default view. It enables the user to brows images as in a grid with quick thumbnail previews.
- Single Image Modal: Selecting an image from the Gallery will open a popup for the user to view the selected image.
  - The user can zoom and scroll around the zoomed image.
- Fast Loading: Concurrent image decoding with LRU caching.
- Keyboard Navigation: Navigate images without taking your hands off the keyboard.
- Native Desktop Environment Integration: Follows your desktop theme and conventions.

## Dependencies
- Rust 2024 Edition [installation](https://rust-lang.org/install)
- libxkbcommon-dev
- just

## Screenshots
![](./screenshots/about.png)
![](./screenshots/gallery_view.png)
![](./screenshots/responsive_gallery_1.png)
![](./screenshots/responsive_gallery_2.png)
![](./screenshots/responsive_gallery_3.png)
![](./screenshots/zoomed_in_scrollbars.png)
![](./screenshots/zoomed_out.png)
![](./screenshots/open_directory.png)
![](./screenshots/view_menu.png)
![](./screenshots/navigate_menu.png)

## Installation

---

### From Source 
```bash
git clone https://codeberg.org/bhh32/cosmic-viewer.git
cd cosmic-viewer
sudo just install
```

## Supported Formats

| Format | Extension   | Works/Needs Testing/Planned |
|--------|-------------|-----------------------------|
| PNG    | .png        | works         |
| JPEG   | .jpg, .jpeg | works         |
| GIF    | .gif        | works         |
| WebP   | .webp       | works         |
| BMP    | .bmp        | works         |
| TIFF   | .tif, .tiff | works         |
| ICO    | .ico        | works         |
| RAW    | .raw, .cr2, .cr3, .nef, .arw, .dng, .orf, .rw2 | needs testing |
| HEIC/HEIF | .heic, .heif (requires --features heif) | planned |

## Usage
```bash
# CLI methods

# Just open the viewer to the last directory selected
cosmic-viewer

# Open the viewer to a directory
cosmic-viewer ~/Pictures/wallpapers

# Open the viewer to a specific image
cosmic-viewer ~/Pictures/wallpapers/superman_wallpaper.png
```

If you have it installed, using the `just install` command, you use it just like you would any other image viewer application. If it's set as the default for opening images, it will start with an image opened from the file explorer application.

## Keyboard Shortcuts
| Key | Action |
|-----|--------|
| ← / → | Previous/Next image |
| ↑ / ↓ | Gallery - Focus image above/below currently focused |
| Ctrl + '=' / Ctrl + '-' | Zoom In/Out (single image modal open) |
| Ctrl + F | Fit in Window (single image modal) open |
| Ctrl + 0 | Zoom to 100% (single image modal only, not the same as `Fit in Window`) |
| ESC | Close Single View Modal |
| Ctrl + Q or Alt + F4 | Close the application |

## Configuration Files
Settings are stored at the standard XDG config location:
- ~/.config/cosmic/org.codeberg.bhh32.CosmicViewer/

## Building for Development
just build              # Debug build
just build-release      # Release build
just run                # Run in release (can test better)
cargo fmt               # Format code
cargo clippy            # Run linter

## Roadmap

The goal is to build a fast, private image manager that helps organize photo libraries through tags, locations, and smart detection without shipping photos to someone else's cloud. All ML-based features run locally using embedded models. No cloud AI, no third-party LLMs, nothing leaves your computer.

### In Progress
- [ ] Theme switching

### Core Features
- [ ] Slideshow with play/pause and timer controls
- [ ] Delete image with confirmation
- [ ] Copy image to clipboard
- [ ] Sort by name, date, size
- [ ] Filename search/filter
- [ ] Zoom slider
- [ ] Animated GIF playback
- [ ] Drag and drop to open folders/images
- [ ] Recent folders menu
- [ ] Set as wallpaper

### Editing
- [ ] Rotate 90/270 degrees
- [ ] Flip horizontal/vertical
- [ ] Crop
- [ ] Save and Save As

### Annotations
- [ ] Freehand drawing
- [ ] Highlighter
- [ ] Arrows and shapes
- [ ] Text labels
- [ ] Blur/redact regions

### Organization
- [ ] Manual tagging
- [ ] Boolean tag queries (AND/OR/NOT)
- [ ] Smart collections (saved filters)
- [ ] Duplicate detection (SHA256 + perceptual hash)
- [ ] Example-based auto-tagging using local embedded ML (faces, pets, objects)
- [ ] Color search by dominant color

### Location
- [ ] Map view with clustered pins
- [ ] Location heat map
- [ ] Click-to-filter by map region
- [ ] Timeline view

### Metadata and Export
- [ ] EXIF viewer/editor
- [ ] Export presets
- [ ] Batch format conversion

### Privacy
- [ ] GPS metadata viewer
- [ ] Strip location data from images
- [ ] Privacy audit for sensitive metadata

### Completed
- [x] Gallery keyboard navigation (arrow keys, Enter to select)
- [x] Visual focus indicator on thumbnails
- [x] Directory watching for external file changes
- [x] Zoom controls with fit-to-window
- [x] Thumbnail caching with configurable sizes
- [x] Modal single-image view with navigation
- [x] Fullscreen mode
- [x] Settings page UI
- [x] Slideshow in single view modal (not full implementation)

## Contributing
Contributions are welcome! Please feel free to submit issues and pull requests.

## License
MIT

## Known Bugs/Issues
- Slideshow skips the first image due to starting before single image modal can popup.
- ~~Single image modal blocks the use of the rest of the UI.~~
- ~~While in gallery, no image selected, using the left and right arrow keys opens the single image modal to cycle the images.~~
- ~~Deleting an image externally, currently selected or not, while any image is selected doesn't always refresh the directory.~~
