# 🌊 BeeWave

It's the part of the BeeSynth Project that implements the sound player for different audio formats.  
It accepts the audio file as an input, attempts to convert it to the PCM WAV file using FFmpeg, extracts amplitude samples, performs filtering and send the result to a PC speaker.

The player has two modes:
* **Amplitude mode (Amp)** - performs resampling of the waveform into a square signal with a sampling depth of 1 bit. This mode produces the sound closest to the original.
* **Frequency mode (Freq)** - extracts the most valueable frequencies from the waveform and play them on the PC speaker. This mode produces more pleasant sound, but with loss of details - just as we hum the melody motif.

In each mode, we can use appropriated filters to tune the sound:
* Amplitude-mode filters:
  - ⭱ [Low-pass filter](https://en.wikipedia.org/wiki/Low-pass_filter) - cuts frequencies <span style="text-decoration:underline">higher</span> than the given.
  - ⭳ [High-pass filter](https://en.wikipedia.org/wiki/High-pass_filter) - cuts frequencies <span style="text-decoration:underline">lower</span> than the given.
  - 🍰 Bakery - resamples the waveform into a square signal with 1-bit depth as a final stage before playing.
* Frequency-mode filters:
  - Frequency extractor - extracts the most valueable frequencies from the waveform using Fourier expansion.
  - Note matcher - produces a mapping of the extracted frequencies to the nearest notes.

We can chain filters in a pipeline. The main rule is that the output of the previous filter must match the input of the next filter.

Another setting is the type of a sound emitter backend. There are two types:
* IOCTL-based - sends requests to the kernel driver to change the state of the speaker. It doesn't require time for initialization, it's a simple and straightforward way, but it costs: each request is a complicated operation that may require a context switch and consumes a lot of CPU time, so it becomes difficult to achieve realtime low-latency output with a good precision.
* IOPL-based - finds and patches the IOPL flag in the EFLAGS register of the worker thread to get access to privileged instructions from usermode. It allows us to deal with a PC speaker without a kernel driver, so we can achive extremely low-latency audio output with the best possible precision. But it requires some time for initialization (depends on size of your RAM).

We can use all of these settings using command line arguments:
```
Options:

  --iopl
      Patch IOPL to deal with I/O ports from usermode
      to achive extremely low-latency speaker control.
      Requires some time to initialize.
      Default is IOCTL-based I/O which doesn't require time
      to initialize but has a lot higher latency and less throughput.

  --switch-interval=<nsec>
      Channel switch interval in nanoseconds.
      Default is 20'000'000 (20 msec).
      Example: --switch-interval=35000000  # Switch the channel each 35 msec.
      Applicable for the frequency mode only, ignored otherwise.

Filtering:

  --low-pass=<value>
      Perform low-pass filtering with the given frequency in Hz.
      Passes frequencies LOWER than the given one.
      Example: --low-pass=4000  # Drops frequencies above 4 kHz
      Amp -> [Low-Pass] -> Amp

  --high-pass=<value>
      Perform high-pass filtering with the given frequency in Hz.
      Passes frequencies HIGHER than the given one.
      Example: --high-pass=300  # Drops frequencies below 300 Hz
      Amp -> [High-Pass] -> Amp.

  --bake-simple
      Bake amplitude samples into the Up/Down sequence
      using the simple strategy (Up if sample is > 0 and Down otherwise).
      Must be the last filter in the chain.
      Amp -> [Bake] -> Position

  --bake-diff=<value>
      Bake amplitude samples into the Up/Down sequence
      using the differential strategy (switch pos if diff(curr, prev) > value).
      Value is a percentage ratio between the current and previous amplitudes.
      Uses as the default bakery with the value of 5%.
      Must be the last filter in the chain.
      Example: --bake-diff=10  # Switch the position if the difference is > 10%
      Amp -> [Bake] -> Position

  --extract-freq=[min=<value>,max=<value>,sampling=<value>,step=<value>,channels=<value>]
      Switch to the frequency mode.
      Find the most valueable frequencies at each point of time
      and send the speaker using PIT-timer.
      Flags (all flags are optional):
          * min=<value> - Drop frequencies below the given one.
          * max=<value> - Drop frequencies above the given one.
          * sampling=<value> - Size of the sampling window to perform FFT,
                               in number of samples, default is 4096 samples.
          * step=<value> - Shift the sampling window by the given number of samples.
                           Default is 32 samples.
          * channels=<value> - Number of peaks to extract.
                               Will play with the given number of channels:
                               monophony if 1, polyphony if >= 2.
                               Default is 2.
      Example: --extract-freq=min=300,max=4000,sampling=4096,step=32,channels=2
      Amp -> [FFT] -> Freq

  --note-matcher
      Find the nearest note for each amplitude sample and use it.
      Freq -> [Note Matcher] -> Freq

Examples:

  beesynth.exe N:\\Folder\\Music.mp3
  beesynth.exe --low-pass=100 --high-pass=4000 --bake-diff=5 N:\\Folder\\Music.mp3
  beesynth.exe --extract-freq=min=100,max=3000 --note-matcher N:\\Folder\\Music.mp3
```