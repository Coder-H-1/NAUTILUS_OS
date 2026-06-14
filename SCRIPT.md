# This is the format of the script that is to be provided to get output through GPIO pins and HDMI.

    I only implemented OUTPUT only, It is very hard to work with very low level stuff T-T 

## Keys
####    **`pin*number`** represents select PIN number, where `number` is a value between (1 to 40).

####    **`>`** represents the start of command. Everything before `>` will be considered as configurations.

####    **`+`** represents PIN **`ON`** 

####    **`-`** represents PIN **`OFF`**

####    **`number`** represents **`delay time`** in milliseconds, where number should be a natural number.

####    **`random number ?`** represents a random number between (0 to `number`), where `number` should be a natural number. `?` must be used after `number`    

# SYNTAX 

    pin pin_number > ON/OFF delay ON/OFF delay            # runs only once

    pin pin_number inf > ON/OFF delay ON/OFF delay        # runs infinitely 

    for random number : 
    pin pin_number inf > ON/OFF random number ? ON/OFF random number ?      # runs infinitely 

# EXAMPLES:
    1. SCRIPT FOR BASIC ON and OFF (executes only once)

        pin 7 > + 1 - 1

        
        This line selects pin 7 and ON for 1 second then OFF for 1 second and program closes.

    

    2. SCRIPT FOR BASIC ON and OFF (executes infinitely)
     
        pin 7 inf > + 1 - 2


        This line selects pin 7 and ON for 1 second then OFF for 2 second and repeats process after `>`.

    -

    3. SCRIPT FOR BASIC ON and OFF with random delay (executes infinitely)

        pin 7 inf > + random 10 ? - random 5 ? 
         

        This line selects pin 7 and ON for (a random number between 0 and 10) and OFF for (a random number between 0 and 5) and repeats after `>`.   


    4. 