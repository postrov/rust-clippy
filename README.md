A pointless rust rewrite of [cliphist](https://github.com/sentriz/cliphist). Should be a drop in replacement for it in integration scripts fron cliphist contrib (e.g. `cliphist-rofi`).

## Problems

- [x] Cursor seems to go from first to last, unlike in go version.
- [x] Separate errors for each operation looks like an overkill, they overlap a lot
  - [x] There are plenty of unwraps to be removed
- [x] Writing newlines should be platform independent (nvm: rust uses \n everywhere)
- [x] Image format is not detected
- [x] Decode uses '\t' instead of constant
- [x] Delete commands are not implemented yet
  - [x] Delete last
  - [x] Delete query
  - [x] Delete(stdin)
- [x] Trim length could be much simpler
- [ ] Use it!
