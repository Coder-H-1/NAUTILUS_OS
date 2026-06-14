# Custom_Baremetal_Script_for_RPI_3B-1.2v


*This folder is like 64-bit kernel low level OS FOR Raspberry Pi 3 Model B+ v1.2*

*I mainly made this to get a low level learning with memory and working*

## My focus was mainly on : 
    1. Input/Output - read and write operations for SD card or USB 
    2. GPIO - write and read controls 
    3. HDMI output - using framebuffer for debugging errors and working


## How I intent it to work :

    Suppose I need a set of outputs through GPIO pin (17).

    I will need to provide a script representing on and off and delay time through USB.
        - more details provided in Script.md

    RPI will read the USB for script and get instructions and executes it.
    