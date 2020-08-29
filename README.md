# bvarint

A Better Varint encoding that preserves `memcmp` order.

Based on "A Better Varint..." slide from [D. Richard Hipp](https://en.wikipedia.org/wiki/D._Richard_Hipp)'s [talk](https://youtu.be/gpxnbly9bz4?t=2386):

> We use variable length integers.
>
> [snip: how traditional VLQ works]
>
> This was a mistake. If you need variable length integers, don't do them this way. Instead, do them like this where the first byte tells you the magnitude of the integer.
>
> This is very important for efficiency in parsing. The other thing is that you can actually compare two integers using `memcmp` without having to decode them.

