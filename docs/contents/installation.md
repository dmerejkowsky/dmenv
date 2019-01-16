# Installing dmenv

## Using the pre-compiled binaries

The easiest way is to download the matching binary from the [releases page](https://github.com/TankerHQ/dmenv/releases) for your platform and put it
somewhere on in your $PATH.

### Linux, macOS

```console
cd ~/.local/bin
curl --fail -L https://github.com/TankerHQ/dmenv/releases/download/v0.8.4/dmenv-<platform> -o dmenv
chmod u+x dmenv
```
Note: replace `<platform>` by your current platform: `linux`, or `osx`.

### Windows

Download the `dmenv-windows.exe` [from the release
page](https://github.com/TankerHQ/dmenv/releases) and save it for instance in
`c:\path\to\python\Scripts\dmenv.exe`. (This folder should already be in you PATH if you
used the default settings when installing Python).

## Installing from source

If you prefer, you can also [install rust](https://www.rust-lang.org/en-US/install.html) and install dmenv with `cargo install dmenv`.