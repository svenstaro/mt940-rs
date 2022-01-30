# Changelog

## 1.0.0

* Fixed parsing failure when a tag-like string exists in the value of a tag 86 (#51).

## 0.3.2 - 2019-09-24

* Added tag 28 (thanks @twistedfall in #17).

## 0.3.1 - 2018-12-03

* Added a new sanitizer function `strip_stuff_between_messages` in order to get rid of non-confortmant stuff between statements.
  Some banks and tools apparently separate statements with dashes or such. We can't have that!
* Added a convenience `sanitize` function that you can call if you just want your input sanitized but don't care exactly how.
* Added tons of tests and some benchmarks.
* Added official WebAssembly support.

## 0.3.0 - 2018-11-28

* Added sanitizers in order to robustly prepare data for parsing on a best-attempt basis.
* `sta2json` will now try to sanitize input by default before attempting to parse it.
* Added `-s` to `sta2json` which will make it strict and refuse invalid input.
  It won't try to sanitize input in this case.
* Added new `NonStandard` to `TransactionTypeIdentificationCode`.
  This allows for parsing even non-standard codes.
  This was added because every bank seems to be using some custom codes and the ones that are known don't seem to be formally standard anyway.
  They seem to be more of a default set.
  This solution seems like a sensible default.
