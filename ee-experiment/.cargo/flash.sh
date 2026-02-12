#!/bin/bash
set -e

# ensure pico is connected in BOOTSEL
# {
#     picotool info > /dev/null 2>&1
# } || {
#     echo -e "\x1B[1;31m       Error\x1B[0m No Raspberry Pi Pico detected. Please connect your device in BOOTSEL mode and try again"
#     exit 1
# }

# convert to UF2 binary
echo -ne "\x1B[1;32m Converting\x1B[0m ELF to UF2 binary\r"
picotool uf2 convert "$1" -t elf "$1.uf2" > /dev/null
echo -e "\x1B[1;32m   Converted\x1B[0m ELF to UF2 binary   "


# echo -ne "\x1B[1;32m    Flashing\x1B[0m To Pi Pico\r"

# picotool load -v "$1.uf2"

# if [ $? -ne 0 ]; then
#     echo -e "\x1B[1;31m       Error\x1B[0m Failed to flash the Raspberry Pi Pico."
#     exit 1
# fi

# echo -e "\x1B[1;32m      Flashed\x1B[0m To Pi Pico. rebooting device..."
# picotool reboot
# echo -e "\x1B[1;32m        Done\x1B[0m"

