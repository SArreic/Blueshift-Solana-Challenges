.globl entrypoint

.equ TOKEN_ACCOUNT_BALANCE, 0x00a0
.equ TOKEN_ACCOUNT_DATA_LEN, 0x0058
.equ ACCOUNT_METADATA_SIZE, 0x00a5

.text
entrypoint:
    stxdw [r10-8], r1
    
    ldxdw r2, [r1+TOKEN_ACCOUNT_DATA_LEN]
    
    add64 r1, ACCOUNT_METADATA_SIZE
    add64 r1, r2
    
    ldxdw r4, [r1+0]
    
    ldxdw r1, [r10-8]
    
    ldxdw r5, [r1+TOKEN_ACCOUNT_BALANCE]
    
    jge r4, r5, fail
    
    lddw r0, 0
    exit

fail:
    lddw r1, error_msg
    lddw r2, 17
    call sol_log_
    lddw r0, 1
    exit

.rodata
error_msg: .ascii "Slippage exceeded"