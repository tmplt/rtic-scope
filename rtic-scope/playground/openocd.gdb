target extended-remote :3333

# Print demangled symbols
set print asm-demangle on

break rust_begin_unwind
break HardFault


# NOTE: pick ONE of these (see cortex-m-quickstart for more details)
#monitor tpiu config internal itm.bin uart off 16000000
# monitor tpiu config external uart off 16000000 2000000 # requires an external USART

# set EXCEVTENA; clear PCSAMPLENA
# this must be ported to rust (ALSO, find manual ref)
#monitor mmw 0xE0001000 65536 4096

# Enable ITM port 0
# this must also be ported to Rust.
# monitor itm port 0 on


#load

# start the process but immidiately halt the processor
#stepi