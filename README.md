# luks-volume-key-hash

luks-volume-key-hash is a simple tool to calculate the hash for [systemd's `fixate-volume-key` crypttab option](https://www.freedesktop.org/software/systemd/man/latest/crypttab.html#fixate-volume-key=).

The normal way to compute the hash is to add the `tpm2-measure-pcr=yes` crypttab option to the relevant volumes, reboot, and then inspect `/run/log/systemd/tpm2-measure.log` or `/usr/lib/systemd/systemd-pcrlock cel`. However, this is not very convenient and if there are >16 volumes with this option in crypttab, it can fail due to `systemd-pcrextend.socket`'s `MaxConnectionsPerSource=16` option. luks-volume-key-hash calculates the volume key hash directly without any TPM involvement.

## Building from source

1. Make sure Rust, clang, and the cryptsetup development package are installed.

2. Build the executable from source.

    ```bash
    cargo build --release
    ```

    The output binary is written to `target/release/luks-volume-key-hash`.

## Computing the volume key hash

The hash depends on the name of the volume specified in the crypttab (the first column). If the name is `luks-<lowercase UUID>` (systemd's default), then run:

```bash
sudo luks-volume-key-hash /dev/disk/by-uuid/<UUID>
```

Otherwise, pass in the name explicitly:

```bash
sudo luks-volume-key-hash -n <name> /dev/disk/by-uuid/<UUID>
```

It will interactively prompt for the LUKS passphrase and then print the volume key hash to stdout.

## Contributing

([AI policy](https://github.com/chenxiaolong/chenxiaolong/blob/master/AI_POLICY.md))

Bug fix pull requests are welcome and much appreciated!

If you are interested in implementing a new feature and would like to see it included in luks-volume-key-hash, please open an issue to discuss it first.

## License

luks-volume-key-hash is licensed under GPL-3.0-only. Please see [`LICENSE`](./LICENSE) for the full license text.
