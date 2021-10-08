# Oct 4th, 2021
Started working on a prototype of a terminal emulator, the first part is to get a window opened
and render stuff on its surface. With the help of `wgpu` and `wgpu_glyph`, it's a piece of cake.

The problem is when I started a `pty` pair, the whole application choke because I tried to read from
`ptm` right in the event loop. Can't handle it differently because I don't have a proper
architecture.

# Oct 5th, 2021

![](./_meta/oct-05.png)

Finally I managed to build a better architecture for the app, it's now have 2 different module
to handle two different stuff: `AppFrontend`, solely for rendering, and `AppBackend` to handle
the creation and communication with the `ptm`.

Line break and spacing characters are not handled. The performance is horrible when it come to
receiving user's input. I'm not sure if I should write to `ptm` every keystroke or not,
but I guess that's how it should. So the problem must be with the way I render a single big ass
text buffer to the screen.

# Oct 6th, 2021

![](./_meta/oct-06.gif)

After seeing my screenshot, friend of mine showed me a version of his own terminal emulator
([mos/terminal](https://github.com/MQuy/mos/tree/master/src/apps/terminal)), I know what you're
thinking, yes, making a terminal emulator is just a trivial thing that people do these days in
their free time.

Well, that helped me a lot. Turned out writing every key stroke to `ptm` is the right thing
to do. But the most important thing is to render the text buffer as an actual character grid.

That's easy. So I came up with a rough implementation just to see how it actually works. I also
passing a raw buffer from `AppBackend` to `AppFrontend` instead of converting them on every event.

The performance improved a lot! And another problem just popped up, how to handle control characters?

I mean, where should I handle it, in `AppBackend` or `AppFrontend`? And I think I need an actual
`Cursor`.

# Oct 7th, 2021
Not much progress for today, I built a key code to character mapping, so the terminal can now
send proper characters to the `ptm`. It can handle things like `Ctrl + C` as well as the tab complete! Yay.

Some keys still not being handled, like the Fn row or the Home/End/PgUp/PgDown keys.

Next is to handle some control characters when reading back from `ptm`.

Btw, terminal exit command now being handled properly.

# Oct 8th, 2021

![](./_meta/oct-08.png)

Some update to the module names for better understanding and organization:

- **Shell:** It was `AppBackend`, is the module that hold the `pty` pair, send commands to `ptm` and read from `ptm`.
- **Terminal:** It was `AppFrontend`, is the module that provide display and most of the interaction.
- **Cursor:** The cursor in the terminal, new characters coming from the Shell/`ptm` will be inserted at cursor position.
- **Characters:** The module provide some utilities to work with characters, like translate key code to input chars,...

```
┌────────┐        ┌──────────────────┐
│  ptm   │◀──────▶│  device::Shell   │
└─┬────▲─┘        └──────┬────▲──────┘
  │    │               read   │
┌─▼────┴─┐               │  write
│  pts   │        ┏━━━━━━▼━━━━┻━━━━━━┓
├────────┤        ┃ winit::EventLoop ┃
│ $SHELL │        ┗━━━━━━━━┳━━━━━━━━━┛
└────────┘                 │udpate
                           │
                  ┌────────▼─────────┐
                  │terminal::Terminal│
                  └─────┬────────────┘
                        │
                  ┌─────▼────┐
                  │  Cursor  │
                  ├──────────┴────┐
                  │ CharacterGrid │
                  └───────────────┘
```

Howerver, the current implementation not really using any cursor, and the rendered cursor is the fake one.

The terminal now also draw the background of each character of the grid. Although this might need to get refactored
later on if the performance is not great.

The `set_data` method of the `terminal` now being changed to be able to handle control characters later on. Next up
will be scrolling and a proper cursor handling.
