# Workflow

```shell
# use cargo generate to create the sub-project for the current day and download the input from AOC website using session cookie value from .envrc
just create day-01

# work on a part of the problem in watchmode running the test continuously 
just work day-01 1 # part 1

# when test passes, run the bin with the real input
just run day-01 1 # part 1
```
