Nothing much to say now.

The basic architecture (kind of) so far is:

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

See [DEVLOG](DEVLOG.md) for the progress.
