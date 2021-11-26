# grep_bin
`grep_bin` can search recursively a directory or multiple files for a sequence of bytes or ascii string.

## Usage
### Searching for a byte sequence in a file
`$ grep_bin test.bin fffe`

### Searching recursively a directory for a byte sequence
`$ grep_bin ~/Downloads FFFE`

### Filtering the filetypes
`$ grep_bin ~/Downloads FFfe0000 -f mp3 `

### Search for an ASCII string inside the binary
`$ grep_bin test.bin '"Hello World"'`

Search for an ASCII string with quotes included: `$ grep_bin test.bin '"This is a \"quote\""'`

### Search a byte sequence in multiple files
`$ grep_bin test1.bin test2.bin fFFe`

### Specify the number of bytes per line in the output
`$ grep_bin README.md "information" -c 32`

Output:
<pre>
README.md
00000320:  73 20 68 65 6C 70 20 <b>69  6E 66 6F 72 6D 61 74 69  6F 6E</b> 0A 0A 20 20 20 20  2D 56 2C 20 2D 2D 76 65   |s help <b>information</b>..    -V, --ve|
00000360:  73 69 6F 6E 20 <b>69 6E 66  6F 72 6D 61 74 69 6F 6E</b>  0A 0A 0A 4F 50 54 49 4F  4E 53 3A 0A 20 20 20 20   |sion <b>information</b>...OPTIONS:.    |
</pre>
* the characters in bold represent the colored match
### Help
```
$ grep_bin -h

grep_bin 1.1.1
LaBatata101 <labatata101@linuxmail.org>
Searches recursively a directory or multiple files for a sequence of bytes or ASCII string.

USAGE:
    grep_bin [OPTIONS] <FILE>... <PATTERN>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c <context_bytes_size>        Defines the number of bytes that will be printed in each line. [default: 16]
    -f <filetype>...               Filter the search by the file extensions.
                                   Examples of input: jpg, mp3, exe

ARGS:
    <FILE>...    The file path
    <PATTERN>    Can be a ascii string or a byte sequence.
                 Ascii strings should be passed inside quotes like so '"This is a string"'
                 Escaping quotes '"This is a \"quote\""'
                 All of these byte sequence are valid: f9b4ca, F9B4CA and f9B4Ca
```

# Building Manually
## Dependencies
- rustc(latest version)
- cargo

`$ git clone https://github.com/LaBatata101/grep_bin`

`$ cd grep_bin/`

`$ cargo build --release`

The final binary will be in `target/release/`

# Installing with Cargo
`cargo install grep_bin`
