Nothing much to say now.

The basic architecture (kind of) so far is:

```
pty -> AppBackend
           |
           v
       Event Loop --> AppFrontend -> Render
```