# 🎹 BeeSynth

Using the BeeSynth Project, you can write and play your own music using a special musical notation.

It's just a text file with a header which describes tempo and channel count, and a body which contains notes for each channel:
```
#!/bin/beesynth         ; Required shebang at the very first line

; Required attributes:
@bpm      : 120         ; Beats (number of quarter notes) per minute
@channels : theme bass  ; Channels that will be played

; Notes of the channel 'theme':
@theme: !Q:E3 E:0 .W:A4
        W:G4# Q:0 ~H:A2b

; Notes of the channel 'bass':
@bass: !E:F3b Q:A3#

; Continue the channel 'theme':
@theme: H:A2b

```

You can play this file just as follows:
```cmd
beesynth.exe N:\\Folder\\notes.txt
```
The only supported command-line arguments are `--iopl` (see at the [BeeWave](./BEEWAVE.md) page) and `--switch-interval=N` which is used to switch the channel each N nanoseconds to emulate polyphonic sound:
```cmd
beesynth.exe --iopl --switch-interval=2000000 N:\\Folder\\notes.txt
```

### Markup syntax:
* **Shebang** is required and must be `#!/bin/beesynth` at the very first line.
* **Comments** are written as `#` at the arbitrary place of the line.
* **Attributes** are written as `@name: value`.  
  There are two required attributes:
  - `@bpm` - beats (number of quarter notes) per minute. E.g., `@bpm: 120`.
  - `@channels` - space-separated list of channels that will be played. E.g., `@channels: ch1 ch2`.

  All other attributes are treated as channel names. Channels are played simultaneously.
### Note format
**Notes** are written as `[STYLE:]DURATION:NOTE[MODIFIER]`:
* `STYLE` is optional and can be one of the following:
    - Absent - non-legato.
    - `!` - staccato.
    - `~` - legato.
    - `.` - prolongated note (x1.5 of the base duration).
* `DURATION` is required and can be one of the following:
    - `W` - 𝅝 - whole note.
    - `H` - 𝅗𝅥 - half note.
    - `Q` - 𝅘𝅥 - quarter note.
    - `E` - 𝅘𝅥𝅮 - eighth note.
    - `S` - 𝅘𝅥𝅯 - sixteenth note.
    - `T` - 𝅘𝅥𝅰 - thirty-second note.
    - `X` - 𝅘𝅥𝅱 - sixty-fourth note.
* `NOTE` is required and can be one of the following:
    - `Cn`, `Dn`, `En`, `Fn`, `Gn`, `An`, `Bn` - where `n` is the octave number.
    - `0` - silence.
* `MODIFIER` is optional and can be one of the following:
    - `#`, `s` or `♯` - diesis.
    - `b` or `♭` - bemolle.