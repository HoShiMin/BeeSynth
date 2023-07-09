pub(crate) fn print_help() {
    println!("Usage: {} [options] <file>\n", std::env::args().next().unwrap());
    println!(
r#"<file> can be either a synth file or WAV, MP3, FLAC, MIDI, XM"
       or any other file which is convertible to WAV using ffmpeg.
       \"Synth\" file is a file with the following format:
       #!/bin/beesynth     # Shebang and required signature
       @bpm: 120           # Beats per minute, required
       @channels: ch1 ch2  # Active channels
       @ch1: !Q:E3   E:0    W:A4  # Notes of the channel 1
             ...
       @ch2: !E:F3b  Q:A3#
             ...

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
"#
    );
}