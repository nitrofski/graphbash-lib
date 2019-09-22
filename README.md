# GraphBash
Graph based tool for Crash Bash MM exploration

Since the PSX Main RAM covers only all of `0x80XXXXXX`, the first 2 digits will often times be
ommited when refering to addresses.

Notable values:
- Index of A: `0`
- Index of CANCEL: `29`
- Address of current index value: `0x0BF19C`
- Address offset per index unit: `168 bytes`
- Address of index 0: `0x0BDB58`

## Analysing the RAM for a single item
Here is the dumped RAM for the object at index 0 (address `0x0BDB58`):
```
Offset | 0        4        8        C
  0x00 | 10008000 00000000 00000000 00000000
  0x10 | 00000000 00000000 00000000 00000000
  0x20 | 00001000 00001000 00001000 00000000
  0x30 | 00000000 00000000 00000000 00000000
  0x40 | 00000000 00000000 00000000 00000000
  0x50 | 00010400 8001C448 8001CC08 800BDC00
  0x60 | 00000000 00000000 00000000 800B38A4
  0x70 | 0000D0FF 000000FF 00000000 00000002
  0x80 | 00000000 00000000 00000000 00000000
  0x90 | 00000000 00000000 00000000 00000000
  0xA0 | 00000000 00000000
```

In the above RAM dump, offsets `0x70`, `0x74` and `0x7C` are all modified upon selection or
deselection of the item. More specifically, `0x70` and `0x74` move like crazy as long as the item is
selected, while `0x7C` becomes and remains `0x00000002`. Upon deselction, they take the values
`0x0000D0FF`, `0x000000FF` and `0x00000000` respectively.

Starting at offset `0x54`, up until offset `0x6C`, all the values seem to represent addresses in
memory (notice how all non-0 values start with `0x80`). Comparing with the rest of the items, we
notice that...
- offsets `0x54` and `0x58` never change and have the values `0x8001C448` and `0x8001CC08`
  respectively, for all items.
- offset `0x5C` changes for all items and represent the address of the next selectable in the
  list/array.
- offset `0x60` also changes for all items and represent the address of the _previous_ selectable in
  the list/array. We are most likely dealing with a doubly-chained list.
- offsets `0x64` and `0x68` seem to always have the value `0x00000000`.
- offset `0x6C` changes, and seem to always have a value of 4 greater than the same offset of the
  previous item, starting at `0x800B38A4` for item 0.

Knowing this, addresse `0x8001C448`, `0x8001CC08` both represent addresses worth investigating, but
addresses `0x800B38A4`+ all have a greater chance at representing interesting values.

## Investigating newly discovered addresses

### `0x800B38A4`+

```
Address | 0        4        8        C
 0B38A0 |          00000041 00000042 00000043
 0B38B0 | 00000044 00000045 00000046 00000047
 0B38C0 | 00000048 00000049 0000004A 0000004B
 0B38D0 | 0000004C 0000004D 0000004E 0000004F
 0B38E0 | 00000050 00000051 00000052 00000053
 0B38F0 | 00000054 00000055 00000056 00000057
 0B3900 | 00000058 00000059 0000005A 00000040
 0B3910 | 0000003C #Starting with RAM addresses again
```

Yeah... I was wrong in thinking this would be useful. Those are just the ASCII values of all the
characters in the grid. Turns out that segment of memory is all text. Yay!
![BEST GAME!](http://i.imgur.com/W5mu8k6.png)

### `0x8001C448` and `0x8001CC08`

Even though I tried, I couldn't really make out anything of what was going on in there. It looked
very much like garbage, and changing the values down there really didn't seem to affect the behavior
of the cursor position (which, by the way, is what we've been searching for all along).

### Desperate approach: Skimming through memory

I decided to start with the most logical starting point: Item 0. I started by poking all the
addresses upwards of that starting point until I noticed a certain pattern – parentheses, lowercase
"x"s, capital "P"s and "8"s forming columns in the ASCII translation of the RAM. Of course, as they
are, these characters mean nothing, but they do catch the eye. Modifying a single one of these
values succesfully changed the behavior of moving the cursor from the A item.

Starting at address `0x0BD5D0`, we have a series of values which seem to control exactly what
happens for each action and each selection. There is apparently 16 bytes associated to each index.
These are distributed as such:
- 2 bytes for the X position of the item (read on load)
- 2 bytes for tye Y position of the item (read on load)
- 4 bytes pointing to a string to write down in the field up there (remember, we're actually
  supposed to write a savefile name here)
- **4 bytes that control the horizontal movement**
- **4 bytes that control the vertical movement**

By the way, if you change address `0x0BD720` to have the value `0x01FF00`, the whole thing now
becomes impossible, as the cursor simply doesn't go out of bounds anymore. ¯\\\_(ツ)\_/¯ A *little*
bit of testing is all that would have been required to avoid this game breaking glitch. Thank you
Eurocom for overlooking this :)

## What now?

Well, finding this is very, very useful to me. It is now possible to predict exactly how the cursor
will move through memory, and that for all 8 directions and all indexes. All I'll need is a dumping
of the PSX RAM before the whole process starts.

It should be noted though, that this discovery is a little bit of a game changer for the algorithm I
will be developping. It will be unchanging for most values, but every 3/20 indexes (rough
calculation, may be wrong), there is a chance its behavior will have been modified by a previously
visited index.

I still need to investigate exactly how they will be affected, but I am confident it will be
important to take this fact into account when creating the graph that will eventually represent all
of Bash's Memory Manipulation possibilities.

# Known effects

| Index | Address    | Good/Bad | Effect                                                                 | Notes                                     |
| ----- | ---------- | -------- | ---------------------------------------------------------------------- | ----------------------------------------- |
| -63   | `0x0BB270` |          | Name selection screen crashes on load.                                 | Temporary &mdash; Recovers on hub reload. |
| -72   | `0x0BAC94` | Bad      | Navigating Left in name seletion screen stops functioning as expected. | Temporary &mdash; Recovers on hub reload. |


# Log from 2016/07/26 (<http://pastebin.com/kewAgjvH>)
Crash bash Memory Manipulation
------------------------------
So. Stuff happened in Crash Bash lately, and as the first real Memory Manipulation to be discovered
in a game I have some remote interest in (It's a PS1 game of the Sprash sort...), I needed to jump
in an try to uncover its secrets. Quickly, after playing around with it, I found the memory adress
that contains which letter/action of the "ENTER NAME" screen is selected... Those actions are in a
simple array, with A being 0 and CANCEL being 29, and it happens that they never fool-proofed
pressing Down-left on V, which takes you to ID 34. Great. Now from there you can do 5 things:

- Press DOWN: Nothing happens...
- Press LEFT: You move to 45, then you're stuck.
- Press UP: You move to 48, then you're stuck.
- Press UP-LEFT: You move to 59, from here only pressing UP does something, and it brings you on H.
- Press RIGHT: Here is where all the magic happens. It brings you to -94.

Now the wonderful thing is that it probably uses that number as the index of the array in which all
these actions are stored. I noticed that after every press of a button that does not leave the
cursor in place, one value becomes 2, while the one that was previously changed to 2 becomes 0. My
guess: 2 means "Selected", while 0 means "Not selected". In the end, every visited address becomes
0, except for the last which remains 2, and that for as long as the game is not restarted (We're
directly modifying the game code after all...)

By observing the RAM throughout the Panic Dash code, I managed to find a small change in the index,
and map it to 2 addresses. There should therefore be a change of 3 "units" between addresses
`0B83E4` and `0B85DC`, which gives us 504 bytes or 168 bytes per "unit". In this case, this size also
corresponds to the size of the letter/action objects in memory. I confirmed this by comparing the
change in modified addresses and the change in index to multiple steps in the code, and it
corresponds perfectly.

So what now? Well, knowing this, we can theorize that ANY 4-byte in memory that has a 168-byte
integral offset from address `0x000004` can be changed to 2, and then to 0. What's beautiful with this
is that 0 is a magic value in computers. It represents "nothing" and if, say, the value modified was
a pointer to a function, then the whole sequence of operations executed by that function will be
skipped.

Now, I have not investigated what the changed values represent, and still do not understand how some
of these changes can cause confusing things like enable Instawin, or (this strangest effect if you
ask me), make the timer jump from 1:00 to 0:05 consistently... Anyway... Finding exactly which value
does what would take too long, and to be honest I don't really want to be the one breaking the game
even more. All I wanted here was to help understand what was happening under the hood, and give us
a better understanding of what could be possible using this glitch.

Verdict: We are lucky that this glitch has so many useful effects. Some of them were completely
unpredictable, while some others make sense. Actually scrap that. They are all completely
unpredictable. I mean, what? Instawin on everything, timer jumping to 5 seconds, auto-receiving
wumpa fruits and stuff? Wow. With a little bit of wizardry, I wouldn't be surprised if we could use
it to bypass requirements, but that's as far as it could go in the WARP rooms.

Also, no Pokémon-like programming will be possible with this kind of memory manipulation. It's too
basic. This makes me a bit sad :(


Anyway, I wanted to spend the rest of this bin exploring the effect of each of the possible actions
on the index, which is, here, clearly the magical value. Unfortunately, as the following table
shows, the changes seem to follow no logic whatsoever. I do not know whether or not we will
eventually be able to find a specific logic, but either way it seems it wouldn't help to know the
specifics, unlike what I initially thought. What we should keep in mind though, is that the behavior
for each input is set per value. We are just blindly navigating a one-way portals maze, and the
entrance is intersection "34"... I don't know, I felt like that was a nice metaphor :^)

```
        v      d
  ->    34 |      |
R ->   -94 | -120 |
D ->  -143 |  -49 |
U ->  -199 |  -56 |
L ->  -161 |   38 |
U ->  -145 |   16 |
L ->  -134 |   11 |
D ->  -131 |    3 |
R ->  -259 | -128 |
D ->  -387 | -128 |
L ->  -476 |  -89 |
L ->  -473 |    3 |
R ->  -457 |   16 |
U ->  -456 |    1 |
R ->  -578 | -122 |
L ->  -576 |    2 |
U ->  -568 |    8 |
L ->  -504 |   64 |
R ->  -593 |  -89 |
U ->  -693 | -100 |
L ->  -784 |  -91 |
L ->  -716 |   68 |
L ->  -780 |  -64 |
R ->  -886 | -106 |
R ->  -866 |   20 |
U ->  -838 |   28 |
R ->  -919 |  -81 |
L ->  -917 |    2 |
L -> -1005 |  -88 |
L -> -1070 |  -65 |
D -> -1071 |   -1 |
R -> -1051 |   20 |
L -> -1142 |  -91 |
R -> -1223 |  -81 |
U -> -1190 |   33 |
```

Using my observations, I decided to rework the currently used codes. I figured I could make them
"better", or at least shorter. Considering a shorter code means fewer destroyed memory, I came to
the conclusion that a shorter code was likely to make the game more stable, with low risks of
breaking it further.


Following here are my updated/new codes. Keep in mind that I have not tested them as thoroughly as
I should have and therefore they need to be confirmed viable before being used in runs. I used a
similar format as the most recent post on the forum, since the codes most likely won't be used
individually anyway.

```
"Panic Dash": (Final value = -1190)
(was  R D U L U L D R D L L R U R L U L R U L L L R R U R L L L D R L R U)
 now  R L L R R L R R R U L R U L L L L L L R
Side effects: Cannot use WARP room changer right after using the code... Crash out of N. Ballism?
```

```
"PD" -> "Time code": (Final value = -1608)
(was  L R U L R R L D R L R L U D L R R)
 now  R L L R U R
```

```
"PD" -> "Instaboss": (Final value = -1399)
(was  R L R U D L U L U U L)
 now  Couldn't find anything shorter
```

```
"TC" -> "Instaboss": (Final value = -1399)
(was  NONE)
 now  U L L U U L
Note: Doesn't seem to crash. Doesn't crash Bear, nor Papu. Didn't test the other 3 freaks...
```

```
"IB" -> "Instawin": (Final value = -1569)
(was  L R U D L R L R L L L R L R R L U R D U U R R L R L U R U)
 now  L R U D L D R R U U D U R U
crsh  U _ (Note: Index -1381 actually fucks up AI and other stuff real bad...)
Note: Seems to work with everything. Removes scary Fox visual shenanigans.
```


"All in one": (To be fair, that's all we really care about isn't it?)
```
R L L R R L R R R U L R U L L L L L L R R L L R U R U L L U U L L R U D L D R R U U D U R U
```


List of values that cause a crash:
 - 1381
 - 1383
 - 1400
 - 1409
