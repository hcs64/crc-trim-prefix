Say you have the crc32 for a file, but you only have the data for the beginning (prefix) of the file. crc-trim-prefix will compute the crc32 for the missing suffix, which might help in locating it.

Usage: `crc-trim-prefix prefix_file target_size target_crc`

Example:

```
$ echo -n "Hello," > prefix.txt

$ echo -n " world!" > suffix.txt

$ cat prefix.txt suffix.txt > target.txt

$ wc -c target.txt
13 target.txt

$ crc32 target.txt
ebe6c6e6

$ crc-trim-prefix prefix.txt 13 ebe6c6e6
suffix len 7 crc 9297dfa9

$ crc32 suffix.txt
9297dfa9
```
