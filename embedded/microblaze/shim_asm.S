 .text
 .align 2
 .globl gettimeofday
gettimeofday:
    get     r3,rfsl0
    swi     r3,r5,0
    get     r3,rfsl1
    swi     r3,r5,4
    rtsd    r15,8 
    add     r3,r0,r0

.globl outbyte
outbyte:
    rtsd    r15,8 
    put     r5,rfsl0

.globl exit
exit:
    put     r5,rfsl1
    nop
    .long   0xffffffff

