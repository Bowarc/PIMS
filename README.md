# Process Internal Memory Scanner

This is just for learning purposes.

The goal is to have a value scan similar to cheat engine

then maybe use it to create patterns ?

that would be pog

## Current state :
Working fine

gui under construction

Notes

VirtualQuery could be usefull to scan a lot of the memory without triggering STATUS_ACCESS_VIOLATION, maybe ?
addr % page len gives us the current page index. right ?

https://github.com/darfink/region-rs this could be a good learning source 