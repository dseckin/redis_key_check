redis key check program
------------------------------------

cargo run 
  Creates the key file (all the keys)
  uses the SCAN method so that it can be used with huge databases

cargo run --diff 
 Check the previously written keys against the current ones. 
 At the end, write the new ones as well as the deleted ones to a new file
