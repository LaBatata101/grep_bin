# grep_bin
Search a sequence of bytes in a binary file

## Usage
### Searching for a byte sequence in a file
`$ grep_bin -f test.bin -s fffe`

### Searching recursively in a directory for a byte sequence
`$ grep_bin -f ~/Downloads -s FFFE`

### Filtering the filetypes
`$ grep_bin -f ~/Downloads -s FFfe0000  -t mp3`

### Help
```
$ grep_bin -h

grep_bin 0.1.0

USAGE:
    grep_bin [OPTIONS] -f <filepath>... -s <search>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f <filepath>...        The file path
    -t <filetype>...        Filter the file types.
    -s <search>             The sequence of bytes to be searched in file.
                            Example of valid inputs: f9b4ca, F9B4CA and f9B4Ca are all valid.
```