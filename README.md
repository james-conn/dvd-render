# dvd-render

`dvd-render` is a GPU accelerated text renderer for (headless) terminal emulators.
You can read the documentation [here](https://docs.rs/dvd-render/latest/dvd_render/).

## Usage Example

The entry point to this crate is the `GridSequence` type.
A `GridSequence` represents... well, sequence of grids at a given font size:

```rust
// create an empty sequence of grids at 100 pt font size
let seq = GridSequence::new(Pt(100.0));

// alternatively, you can specify the font size in pixels like this
let seq = GridSequence::new(Px(50.0));
// or if you want distorted text (for some reason?) you could do this
let seq = GridSequence::new((Px(50.0), Px(20.0));
```

Let's make a grid for our sequence:

```rust
// initialize an empty grid
// note: these const generics are gone in the main branch, but
// no release to crates.io has been made with them gone yet
let grid = Grid::<20, 5>::default();

// add the letter 'A' in the top left corner of the grid
grid.set(0, 0, GridCell::new('A'));
```

In order to add a grid to a sequence, it must be wrapped in a `Frame`.
The `Frame` type is a bit of a misnomer, it represents the number of frames that a grid will be rendered for.
Let's just display our grid for a single frame.

```rust
// show our grid for a single frame
// note: you could also use `Frame::variable` to specify how many frames a grid should
// display for, this makes the renderer more efficient for identical consequtive grids
let frame = Frame::single(grid);
```

Now we can add our `Frame` to the sequence and initialize the renderer:

```rust
seq.append(frame);

let font = ab_glyph::FontRef::try_from_slice(include_bytes!("some_font.ttf"));
let renderer = WgpuRenderer::new(font, seq).await;
```

Finally, if we have the `video` feature enabled, we can render a video of our grid:

```rust
let encoder = DvdEncoder::new(renderer);
encoder.save_video_to("video_output.mkv");
```

Congratulations, you've rendered a video of a terminal headlessly!
