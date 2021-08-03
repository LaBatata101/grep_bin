# grep_bin
Search a sequence of bytes in a binary file

## Usage
### Searching for a byte sequence in a file
`$ grep_bin -f test.bin -b fffe`

### Searching recursively a directory for a byte sequence
`$ grep_bin -f ~/Downloads -b FFFE`

### Filtering the filetypes
`$ grep_bin -f ~/Downloads -b FFfe0000  -t mp3`

### Search for a ASCII string inside the binary
`$ grep_bin -f test.bin -s "Hello World"`

### Help
```
$ grep_bin -h

grep_bin 0.1.0
Searches for a sequence of bytes  or a ASCII string in a binary file.
If a directory is provided grep_bin will search every file in the directory recursively.

USAGE:
    grep_bin [OPTIONS] -b <bytes> -f <filepath>... -s <string>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b <bytes>              The sequence of bytes to be searched in the file.
                            Example of valid inputs: f9b4ca, F9B4CA and f9B4Ca are all valid.
    -f <filepath>...        The file path
    -t <filetype>...        Filter the search by the file extensions.
                            Examples of input: jpg, mp3, exe
    -s <string>             Search for ASCII string inside the file
```

# Building Manually
`$ git clone https://github.com/LaBatata101/grep_bin`

`$ cd grep_bin/`

`$ cargo build --release`