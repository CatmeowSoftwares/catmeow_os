[org 0x7c00]
mov ah, 0x0e
mov bx, label

mov al, [char]
char:
    db 0
    mov ah, 0
    int 0x16

labal_again:
    mov al, [bx]
    cmp al, 0
    je exit
    int 0x10
    inc bx
    jmp labal_again


label:
    db "meow :3", 0
loop:
    inc al
    cmp al, 'Z' + 1
    je exit
    int 0x10
    jmp loop


exit:
    mov ah, 0x0e
    mov al, 'c'
    int 0x10
    jmp $



times 510 - ($-$$) db 0
db 0x55, 0xaa