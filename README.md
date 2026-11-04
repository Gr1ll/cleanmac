# cleanmac

## Setup

```bash
cargo install --path .
```

## Usage

```bash
# list caches and delete them with y/n
cleanmac caches
cleanmac caches --dry-run
cleanmac caches --all

# list biggest files from specified path
cleanmac biggest
cleanmac biggest ~/Downloads --depth 2

# list apps not installed via homebrew
cleanmac apps
```
