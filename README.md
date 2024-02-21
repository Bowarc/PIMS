# Internal memory scanning

This is just for learning purposes.

The goal is to have a value scan similar to cheat engine

then maybe use it to create patterns ?

that would be pog

## Current state :
Im able to get all the memory pages used by the target app from the dll

Internal scan but for some reason the good pages are skip and if i disable the check, i get STATUS_ACCESS_VIOLATION

Page rights checks needs to be better

To verify that it was at least possible to scan the page of a given variable of dummy.exe i made a simple test in the start of scanner.dll

It finds the page if the given variable, then scans it
it seems to not trigger STATUS_ACCESS_VIOLATION, so it means that i can scan it, just the check arnt good yet


Notes

VirtualQuery could be usefull to scan a lot of the memory without triggering STATUS_ACCESS_VIOLATION, maybe ?
addr % page len gives us the current page index. right ?

https://github.com/darfink/region-rs this could be a good learning source 