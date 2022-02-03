# Wordle tester
A wordle implementation that I made for a competition of wordle solvers.

Runs tests using a list of words and outputs the total turns taken to solve all of them.

WIP

## Usage
Currently the program takes 2 arguments
```sh
wordle-tester <RUNFILE> <WORDSFILE>
```

- `RUNFILE` is the file with the list of all the executables that you want to run
- `WORDSFILE` is the file with the list of words to use in the test

## Testee format

```sh
<executable> <WORDSFILE>
```

- The executable being tested needs to take the words list file as an argument
- It should not output anything else besides the guesses
- Input will be provided as follows:
    - `y` for letters that are in the answer, and not in the correct position
    - `g` for letters in the correct position
    - `b` for non-matches
