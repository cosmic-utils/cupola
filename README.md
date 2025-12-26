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

## Contributing
Contributions are welcome! Please feel free to submit issues and pull requests.

## License
MIT

## Known Bugs/Issues
- ~~Single image modal blocks the use of the rest of the UI.~~
- While in gallery, no image selected, using the left and right arrow keys opens the single image modal to cycle the images.
- ~~Deleting an image, currently selected or not, while an image is selected doesn't always refresh the directory.~~
