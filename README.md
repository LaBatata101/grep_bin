# grep_bin
`grep_bin` can search recursively a directory or multiple files for a sequence of bytes or ascii string.

## Usage
### Searching for a byte sequence in a file
`$ grep_bin test.bin fffe`

### Searching recursively a directory for a byte sequence
`$ grep_bin ~/Downloads FFFE`

### Filtering the filetypes
`$ grep_bin ~/Downloads FFfe0000 -f mp3 `

### Search for a ASCII string inside the binary
`$ grep_bin test.bin "Hello World"`

### Search a byte sequence in multiple files
`$ grep_bin test1.bin test2.bin fFFe`

### Help
```
$ grep_bin -h

grep_bin 1.0.2
LaBatata101 <labatata101@linuxmail.org>
Searches for a sequence of bytes  or a ASCII string in a binary file.
If a directory is provided grep_bin will search every file in the directory recursively.

USAGE:
    grep_bin [OPTIONS] <FILE>... <PATTERN>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f <filetype>...        Filter the search by the file extensions.
                            Examples of input: jpg, mp3, exe

ARGS:
    <FILE>...    The file path
    <PATTERN>    Can be a ascii string or a byte sequence.
                 Ascii strings should be passed inside quotes like so "This is a string"
                 All of these byte sequence are valid: f9b4ca, F9B4CA and f9B4Ca
```

# Building Manually
`$ git clone https://github.com/LaBatata101/grep_bin`

`$ cd grep_bin/`

`$ cargo build --release`

# Installing with Cargo
`cargo install grep_bin`
