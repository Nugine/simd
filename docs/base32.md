# base32

## split_bits (*)

```txt
40x4 bits -> 5x8x4 bits

INPUT
    {????|??AA|AAAB|BBBB|CCCC|CDDD|DD??|????}
     0123 4567 89AB CDEF 0123 4567 89AB CDEF

swizzle u8
    {bbcccccd|aaaaabbb|ddddeeee|bbcccccd|efffffgg|ddddeeee|ggghhhhh|efffffgg}x4
    (10213243)

shr(4) u16
    {abbbbbcc|0000aaaa|cccddddd|0000bbcc|eeeeefff|0000dddd|ffgggggh|0000efff}x4

mullo u16
    {bbbbbcc0|000aaaaa|ddddd000|0bbccccc|fff00000|dddeeeee|h0000000|fffggggg}x4
        shl(1)              shl(3)              shl(5)          shl(7)

mullo u16
    {cccccd00|aaabbbbb|eeee0000|cccddddd|gg000000|eeefffff|00000000|ggghhhhh}x4
        shl(2)              shl(4)              shl(6)          shl(8)

2x and v256
    {00000000|000aaaaa|00000000|000ccccc|00000000|000eeeee|00000000|000ggggg}x4
    {00000000|000bbbbb|00000000|000ddddd|00000000|000fffff|00000000|000hhhhh}x4

shr(8) u16
    {000aaaaa|00000000|000ccccc|00000000|000eeeee|00000000|000ggggg|00000000}x4

or v256
    {000aaaaa|000bbbbb|000ccccc|000ddddd|000eeeee|000fffff|000ggggg|000hhhhh}x4

OUTPUT
    {000xyyyy}x32

SPEED (ops/byte)
    8/32
```

## split_bits (x86)

```txt
INPUT
    {????|??AA|AAAB|BBBB|CCCC|CDDD|DD??|????}
     0123 4567 89AB CDEF 0123 4567 89AB CDEF

swizzle u8
    {bbcccccd|aaaaabbb|ddddeeee|bbcccccd|efffffgg|ddddeeee|ggghhhhh|efffffgg}x4
    (10213243)

mulhi u16
    {000aaaaa|00000000|000ccccc|00000000|000eeeee|00000000|000ggggg|00000000}x4
            shl(5)            shl(7)            shl(9)            shl(11)

mullo u16
    {cccccd00|aaabbbbb|eeee0000|cccddddd|gg000000|eeefffff|00000000|ggghhhhh}x4
            shl(2)            shl(4)            shl(6)            shl(8)

and v256
    {00000000|000bbbbb|00000000|000ddddd|00000000|000fffff|00000000|000hhhhh}x4

or v256
    {000aaaaa|000bbbbb|000ccccc|000ddddd|000eeeee|000fffff|000ggggg|000hhhhh}x4

OUTPUT
    {000xyyyy}x32

SPEED (ops/byte)
    5/32
```

## encode_symbols (x86)

```txt
40x4 bits -> 8x32 bits

INPUT
    {000xyyyy}x32

2x swizzle u8
    {{ascii}}x32
    {{ascii}}x32

and v256
    {000x0000}x32

blendv u8
    {{ascii}}x32

OUTPUT
    {{ascii}}x32

SPEED (ops/byte)
    4/32 
```

## encode_symbols (arm)

```txt
INPUT
    {000xyyyy}x32

2x swizzle u8
    {{ascii}}x32
    {{ascii}}x32

cmplt u8
    {{000xyyyy > 00001111}}x32

bsl v256
    {{ascii}}x32

OUTPUT
    {{ascii}}x32

SPEED (ops/byte)
    4x2/32
```

## encode_symbols (aarch64)

```txt
INPUT
    {000xyyyy}x32

swizzle (32B table lookup)
    {{ascii}}x32

OUTPUT
    {{ascii}}x32

SPEED (ops/byte)
    1/32
```

## encode_symbols (wasm)

```txt
INPUT
    {000xyyyy}x32

2x swizzle u8
    {{ascii}}x32
    {{ascii}}x32

cmplt u8
    {{000xyyyy > 00001111}}x32

bsl v256 (mock aarch64)
    {{ascii}}x32

OUTPUT
    {{ascii}}x32

SPEED (ops/byte)
    4x2/32
```

## merge_bits (*)

```txt
INPUT
    {000aaaaa|000bbbbb|000ccccc|000ddddd|000eeeee|000fffff|000ggggg|000hhhhh}x4

and v256
    {000aaaaa|00000000|000ccccc|00000000|000eeeee|00000000|000ggggg|00000000}x4

mullo u16
    {aaaaa000|00000000|00ccccc0|00000000|e0000000|0000eeee|ggg00000|000000gg}x4
        shl(3)              shl(1)              shl(7)          shl(5)
        0        1        2       3       4         5        6        7
        8        9        A       B       C         D        E        F

shr(8) u16
    {000bbbbb|00000000|000ddddd|00000000|000fffff|00000000|000hhhhh|00000000}x4
        
mullo u16
    {bb000000|00000bbb|dddd0000|0000000d|0fffff00|00000000|000hhhhh|00000000}x4
        shl(6)              shl(4)            shl(2)            shl(0)
        0        1        2       3       4         5        6        7
        8        9        A       B       C         D        E        F

2x swizzle
    {aaaaa000|00ccccc0|0000eeee|000000gg|ggg00000|00000000|00000000|e0000000}x4
    {00000bbb|bb000000|dddd0000|0fffff00|000hhhhh|0000000d|00000000|00000000}x4
                     d          e   

        0,2,5,7,6,Z,Z,4,
        8,A,D,F,E,Z,Z,C,

        1,0,2,4,6,3,Z,Z,
        9,8,A,C,E,B,Z,Z,

or v256
    {aaaaabbb|bbccccc0|ddddeeee|0fffffgg|ggghhhhh|0000000d|00000000|e0000000}x4
        A       B0      C           D0      E       B1               D1
        0        1        2       3       4         5        6        7
        8        9        A       B       C         D        E        F

2x swizzle
    {A |B0|C |D0|E |A |B0|C |D0|E |0 |0 |0 |0 |0 |0 }x2
    {0 |B1|0 |D1|0 |0 |B1|0 |D1|E |0 |0 |0 |0 |0 |0 }x2

        0,1,2,3,4,
        8,9,A,B,C,
        Z,Z,Z,Z,Z,Z,

        Z,5,Z,7,Z,
        Z,D,Z,F,Z,
        Z,Z,Z,Z,Z,Z

or v256
    {ABCD|EABC|DE00|0000}x2

OUTPUT
    {AAAA|ABBB|BB00|0000}x2

SPEED (ops/byte)
    10/20
```

## merge_bits (x86)

```txt
INPUT
    {000aaaaa|000bbbbb|000ccccc|000ddddd|000eeeee|000fffff|000ggggg|000hhhhh}x4

maddubs epi16
        {000aaaaa|000bbbbb|000ccccc|000ddddd|000eeeee|000fffff|000ggggg|000hhhhh}x4
          shl(7)   shl(2)   shl(5)   shl(0)    shl(7)   shl(2)   shl(5)   shl(0)    
    BE  {0000aaaa|abbbbb00|000000cc|cccddddd|0000eeee|efffff00|000000gg|ggghhhhh}x4

shl(4) u32
    BE  {aaaaabbb|bb000000|00cccccd|dddd0000|eeeeefff|ff000000|00gggggh|hhhh0000}x4

blend i32
    BE  {aaaaabbb|bb000000|00cccccd|dddd0000|0000eeee|efffff00|000000gg|ggghhhhh}x4

    LE  {bb000000|aaaaabbb|dddd0000|00cccccd|efffff00|0000eeee|ggghhhhh|000000gg}x4
            B0       A       C0         B1      D0       C1      E          D1
            0        1       2          3       4        5       6          7
            8        9       A          B       C        D       E          F

       ({aaaaabbb|bbcccccd|ddddeeee|efffffgg|ggghhhhh|????????|????????|????????}x4)
            A         B       C       D       E

2x swizzle
    {A |B0|C0|D0|E |A |B0|C0|D0|E |0 |0 |0 |0 |0 |0 }x2
    {0 |B1|C1|D1|0 |0 |B1|C1|D1|0 |0 |0 |0 |0 |0 |0 }x2

    1,0,2,4,6,
    9,8,A,C,E,
    Z,Z,Z,Z,Z,Z

    Z,3,5,7,Z,
    Z,B,D,F,Z,
    Z,Z,Z,Z,Z,Z

or v256
    {ABCD|EABC|DE00|0000}x2

OUTPUT
    {AAAA|ABBB|BB00|0000}x2

SPEED (ops/byte)
    6/20
```

## decode_bits (rfc4648)

base32: A~Z, 2~7
    ch0 = 'A', len0 = 26
    ch1 = '2', len1 = 6
base32hex: 0~9, A~V
    ch0 = '0', len0 = 10
    ch1 = 'A', len1 = 22

```txt
8x32 bits -> 8x20 bits

INPUT
    {{byte}}x32

sub(ch0)
    {{byte - ch0}}x32

cmplt(len0)
    {{byte in range0}}x32

sub(ch1)
    {{byte - ch1}}x32

cmplt(len1)
    {{byte in range1}}x32

or v256
    {{byte is valid}}x32

any zero u8
    is invalid

and v256
    {{symbol(range0)}}x32

add(len0)
    {{byte - ch1 + len0}}x32

and v256
    {{symbol(range1)}}x32

or v256
    {{symbol}}x32
    ({000aaaaa|000bbbbb|000ccccc|000ddddd|000eeeee|000fffff|000ggggg|000hhhhh}x4)

merge_bits
    {AAAA|ABBB|BB00|0000}x2

OUTPUT
    {AAAA|ABBB|BB00|0000}x2

SPEED (ops/byte)
    20/20
```

## decode_bits (crockford)

"0123456789ABCDEFGHJKMNPQRSTVWXYZ"

0~9, A-Z

'I' 'L' 'O' are removed

ch0 = '0', len0 = 10
ch1 = 'A', len1 = 26

```txt
8x32 bits -> 8x20 bits

INPUT
    {{byte}}x32

sub(ch0)
    {{byte - ch0}}x32

cmplt(len0)
    {{byte in range0}}x32
    (m1)

sub(ch1)
    {{byte - ch1}}x32

cmplt(len1)
    {{byte in range1}}x32
    (m2)

sub('I')
    {{byte - 'I'}}x32

swizzle (16B table lookup)
    {{byte is 'I'|'L'|'O'}}x32
    (m3)

    (  JKMN PQRSTVWXY)
    (IL    O         )    
     0123456789ABCDEF

     [0,1,6]  = 0xff
     [others] = 0x00

andnot v256
or v256
    {{byte is valid}}x32

    (m1 | (m2 & (!m3)))

any zero u8
    is invalid

add(1)
    {{byte - 'L'}}x32

swizzle (16B table lookup)
    {{shift}}x32

    ( JKMN PQRSTVWXYZ)
    (L    O         )
     0123456789ABCDEF

    [0] = Z,  [1~4] = -2,
    [5] = Z,  [6~F] = -3,

and v256
    {{symbol(range0)}}x32

add(len0)
    {{byte - ch1 + len0}}x32

and v256
    {{symbol(range1)(needs shift)}}x32

add u8
    {{symbol(range1)}}x32

or v256
    {{symbol}}x32
    ({000aaaaa|000bbbbb|000ccccc|000ddddd|000eeeee|000fffff|000ggggg|000hhhhh}x4)

merge_bits
    {AAAA|ABBB|BB00|0000}x2

OUTPUT
    {AAAA|ABBB|BB00|0000}x2

SPEED (ops/byte)
    26/20
```
