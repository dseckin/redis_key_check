redis key check program
------------------------------------

cargo run 
  Creates the key file (all the keys)
  uses the SCAN method so that it can be used with huge databases

cargo run --diff 
 Check the previously written keys against the current ones and 
 Write the new ones and the deleted ones to a new file
