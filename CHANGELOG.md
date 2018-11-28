# Changelog

## 0.3.0 - 2018-11-28

* Add sanitizers in order to robustly prepare data for parsing on a best-attempt basis.
* `sta2json` will now try to sanitize input by default before attempting to parse it.
* Add `-s` to `sta2json` which will make it strict and refuse invalid input.
  It won't try to sanitize input in this case.
* Add new `NonStandard` to `TransactionTypeIdentificationCode`.
  This allows for parsing even non-standard codes.
  This was added because every bank seems to be using some custom codes and the ones that are known don't seem to be formally standard anyway.
  They seem to be more of a default set.
  This solution seems like a sensible default.
