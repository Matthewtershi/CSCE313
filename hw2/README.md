# Lab 2 — Pipes

Build a program that does what the shell does when you type:

```
ls -al / | tr a-z A-Z
```

You are **not** reimplementing `ls` or `tr`. Those are real programs already on the
system. You are reimplementing the `|`. Your program starts both of them and connects
them together.

`tr` = "translate." It reads input, swaps characters one-for-one, writes output.
`tr a-z A-Z` turns lowercase into uppercase. Given no input it just sits and waits.

---

## Part 1 — The facts you need first

### Fact 1: a running program is a process

You type `./shell.opt`. Linux loads the program into memory and runs it. That running
thing is a **process**. It has an ID number called a **PID**.

### Fact 2: programs do not print to the screen — they print to the number 1

This is the fact everything else rests on.

When `ls` prints a filename it does **not** talk to your monitor. It asks Linux:
*"write these bytes to number 1."*

When `tr` reads input it asks Linux: *"give me bytes from number 0."*

Three numbers are standardized on Linux, for every program ever written:

| number | meaning                     | C name          |
|--------|-----------------------------|-----------------|
| 0      | where my input comes from   | `STDIN_FILENO`  |
| 1      | where my output goes        | `STDOUT_FILENO` |
| 2      | where my errors go          | `STDERR_FILENO` |

`STDIN_FILENO` is literally `#define`d as `0`. It is not special, it is just a name
for the number.

### Fact 3: Linux keeps a list, per process, of what those numbers mean

A normal process's list:

```
0  means:  the keyboard
1  means:  the screen
2  means:  the screen
```

When `ls` says "write to 1," Linux looks at line 1 of *that process's* list, sees
"the screen," and puts the bytes there.

These numbers are called **file descriptors**. Terrible name. They are just numbers:
0, 1, 2, 3, 4...

You can see a real process's list. Run this in one terminal:

```bash
tail -f /dev/null | tr a-z A-Z
```

and this in another:

```bash
ls -l /proc/$(pgrep -x tr)/fd
```

Real output from that pipeline:

```
===== PID 51112  (tail) =====
   fd 0  ->  pipe:[567697]
   fd 1  ->  pipe:[561014]     <---
   fd 2  ->  pipe:[567699]

===== PID 51113  (tr) =====
   fd 0  ->  pipe:[561014]     <---
   fd 1  ->  pipe:[567698]
   fd 2  ->  pipe:[567699]
```

Look at the two arrows. `tail`'s line 1 and `tr`'s line 0 name the **same pipe**,
`pipe:[561014]`. That matching number *is* the pipeline. Not a wire, not a channel —
two entries in two different lists happening to name the same kernel object.

### Fact 4: the whole assignment is "edit that list"

Change line 1 from "the screen" to "this pipe," and `ls` writes into the pipe. `ls`
never notices. It still just says "write to 1." It cannot tell the difference.

---

## Part 2 — The five system calls

### `pipe(fds)` — make a queue

```c
int fds[2];
pipe(fds);
```

Linux creates a queue in memory (bytes in one side, out the other, in order) and adds
**two new lines** to your list:

```
0  means:  the keyboard
1  means:  the screen
2  means:  the screen
3  means:  the queue, READING side      <-- new
4  means:  the queue, WRITING side      <-- new
```

Then it fills in your array with the two numbers it used:

- `fds[0]` = `3` = **read** end
- `fds[1]` = `4` = **write** end

**Why an array?** A C function can only return one value and `pipe` has to hand back
two numbers. So you give it an array to write them into. `fds` is nothing but a place
to receive two numbers.

Mnemonic for which is which: same order as stdin(0) / stdout(1). Index 0 = read,
index 1 = write.

### `fork()` — make a second copy of the running program

After `fork()` there are **two processes**. Identical: same code, same variables, same
list of numbers. Both sit on the exact same line — the one right after `fork()` — and
both keep running from there.

They tell each other apart because `fork` gives them **different return values**:

- the original gets: the PID of the copy (e.g. `51112`)
- the copy gets: **`0`**

That is the only difference.

```c
pid_t consumer_pid = fork();
if (consumer_pid == 0) {
        // I am the copy
}
// the original continues down here
```

### `execlp(...)` — replace the program

```c
execlp("ls", "ls", "-al", "/", NULL);
```

Means: **stop being `shell.opt`, start being `ls`.** Same process, same PID, same list
of numbers — but the running code is now `ls`'s code. Your lines after this one
**never run**. Your variables are gone.

The arguments:

| argument | why |
|----------|-----|
| `"ls"`   | program to find on `$PATH` (the `p` in `execlp`) |
| `"ls"`   | again — becomes the program's own `argv[0]`. Every program expects its own name as argument zero. You really do type it twice. |
| `"-al"`, `"/"` | the actual arguments |
| `NULL`   | marks the end of the argument list. Required — without it `exec` reads past the end. |

**The one thing that survives the swap is the list of numbers.** That is how you hand
the pipe to `ls`. Which is why all the `dup2`/`close` work happens *before* `exec`.

Anything after an `exec` call only runs if the exec **failed**, so that is the right
place to put `perror(...)`.

### `dup2(oldfd, newfd)` — copy one line of the list onto another

`dup2(3, 0)` means *"make line 0 say whatever line 3 says."*

```
before                            after
0  =  the keyboard                0  =  the queue, read side   <-- changed
3  =  the queue, read side        3  =  the queue, read side   <-- untouched
```

Read it as `dup2(from, to)` — the **second** argument is the one that gets
overwritten. Everyone gets this backwards at first.

Note line 3 is **still there**. `dup2` *copied*, it did not move. That is the `dup` in
`dup2` — duplicate. Two lines now name the same queue.

### `close(fd)` — delete a line from the list

`close(3)` means *"erase line 3 from my list."*

```
before                            after
0  =  the queue, read side        0  =  the queue, read side   <-- still works!
3  =  the queue, read side        (deleted)
```

**`close` does not tear down the pipe.** After a `dup2`, line 0 still names the queue,
so reading still works perfectly. You deleted a spare copy of a line.

If `close` actually severed the connection the program could not possibly work. Good
sanity check.

#### Why `close` is the whole ballgame

Linux counts how many lines, **across every process on the machine**, point at the
queue's **writing** side. When that count hits **zero**, Linux tells the reader *"no
more data will ever arrive."* That signal is **EOF**. `tr` sees it, flushes, exits.

If even one line anywhere still points at the writing side, the count is not zero, so
Linux keeps `tr` asleep waiting for bytes that will never come. **The program hangs
forever.**

`fork` copies the list, so after two forks **three** processes hold a line pointing at
the write side:

| process | write end | read end |
|---------|-----------|----------|
| `ls`    | **keeps** (it is the writer) | must close |
| `tr`    | must close — otherwise it is a writer on the pipe it is reading, and waits for itself forever | **keeps** (via line 0) |
| `main`  | must close — it is not in the pipeline at all | must close |

That is why there are `close` calls in three places.

### `waitpid(pid, NULL, 0)` — pause until that process finishes

It does **not** kill anything. The opposite: it waits for the child to end on its own,
then collects its exit status. Two reasons it is here:

- Without it, `main` exits instantly and your bash prompt prints in the middle of
  `tr`'s output.
- A finished process leaves a small record behind (a **zombie**) until its parent
  picks it up. `waitpid` picks it up.

---

## Part 3 — The three functions in `shell.c`

| function     | what it becomes | what goes in it |
|--------------|-----------------|-----------------|
| `producer()` | `ls`            | one `exec` call. Called "producer" because it *produces* bytes into the pipe. |
| `consumer()` | `tr`            | one `exec` call. It *consumes* bytes out of the pipe. |
| `main()`     | the shell       | `pipe`, two `fork`s, the `dup2`/`close` bookkeeping, two `waitpid`s |

### Why two forks?

**`exec` destroys the process that calls it, and you only have one process.**

If `main` called `execlp("ls", ...)` directly, your program would *become* `ls`. It
would list the files, exit, and there would be nobody left to run `tr`.

So you make copies first and let the **copies** be destroyed.

Two programs to run → two copies needed → **two forks**.

```
main (you)
  |
  +-- fork --> copy #1 --> point line 0 at pipe-read  --> exec tr
  |
  +-- fork --> copy #2 --> point line 1 at pipe-write --> exec ls
  |
  +-- close both ends, waitpid x2
```

### Does the fork order matter?

**No.** Forking `ls` first works identically — verified by building both versions and
diffing the output. The order in the starter comments is convention, not requirement.

Same for the `waitpid` order here. The convention is "consumer first" because in a
pipeline with lots of data the writer can block once the pipe buffer fills (64 KB on
Linux), so you generally want the reader being drained. `ls -al /` is about 2 KB, so
it never comes up in this lab.

**What genuinely does matter is closing every descriptor you are not using.**

---

## Part 4 — The timeline

Read straight down. No jumping.

1. You run `./shell.opt`. One process. List: `0=keyboard 1=screen 2=screen`
2. `pipe(fds)` runs. Linux makes the queue. List becomes
   `0=keyboard 1=screen 2=screen 3=queue-read 4=queue-write`. `fds[0]`=3, `fds[1]`=4.
3. First `fork()` runs. Now **two** processes with that identical list. The original
   gets back `51112`; the copy gets back `0`.
4. The copy sees `consumer_pid == 0` and enters the `if`. It runs
   `dup2(fds[0], STDIN_FILENO)` → line 0 now says queue-read. It runs `close(fds[0])`
   and `close(fds[1])` → lines 3 and 4 deleted. Its list is now
   `0=queue-read 1=screen 2=screen`. Then `consumer()` → `execlp("tr", ...)`. **This
   process is now `tr`**, holding that list.
5. `tr` asks Linux for bytes from 0. The queue is empty, so Linux puts `tr` **to
   sleep**. It is not broken or erroring — it is parked. Blocking *is* the
   synchronization mechanism.
6. Meanwhile the original process — which skipped that `if` — reaches the second
   `fork()`. A new copy appears; it gets back `0`.
7. That copy runs `dup2(fds[1], STDOUT_FILENO)` → line 1 now says queue-write. Closes
   3 and 4. Its list is `0=keyboard 1=queue-write 2=screen`. Then `producer()` →
   `execlp("ls", ...)`. **This process is now `ls`.**
8. `ls` lists `/` and says "write to 1." Linux checks its list: line 1 = queue-write.
   The bytes go into the queue. `ls` thinks it printed to a screen. It exits — and
   **its lines are deleted automatically on exit**. Now only `main` still points at
   queue-write.
9. Back in `main`: `close(fds[0]); close(fds[1]);`. Now **nobody** points at
   queue-write. The count hits zero.
10. Linux wakes `tr` up and hands it EOF. `tr` drains the queue, uppercases everything,
    and says "write to 1." `tr`'s line 1 was **never touched** — it still says screen.
    That is why you see output. Then `tr` exits.
11. `main`'s two `waitpid` calls return. `main` returns 0. Done.

**The two-sentence version:** `ls` writes to number 1 and `tr` reads from number 0, and
neither can tell where those numbers lead. You point them at the same queue, then
delete every leftover line pointing at the queue's write side so `tr` knows when to
stop.

---

## Part 5 — Building and running

You are on Windows. `fork()`, `pipe()` and `unistd.h` do not exist there — MSYS2/MinGW
gcc will fail. Use WSL (Ubuntu 24.04, already has `gcc` and `make`):

```bash
wsl -d Ubuntu
cd /mnt/c/Users/matth/VSC/General/CSCE313/hw2/lab2.assignment
make
./shell.opt
```

### Make targets

| command      | produces    | flags added                     | use for |
|--------------|-------------|---------------------------------|---------|
| `make`       | `shell.opt` | `-Ofast -flto=auto`             | the graded build |
| `make dbg`   | `shell.dbg` | `-Og -ggdb3`                    | stepping through in gdb |
| `make san`   | `shell.san` | `-Og -ggdb3 -fsanitize=address` | catching memory bugs |
| `make clean` | —           | —                               | deletes all three |

Base flags are `-std=c18 -Wall -Wextra -Wpedantic -Wconversion -fanalyzer`. Expect
noise about things like assigning `pid_t` to `int`. Warnings do not fail the build
(there is no `-Werror`), but fix them anyway.

### Testing

The fast check — your output should be byte-identical to the real pipeline:

```bash
diff <(./shell.opt) <(ls -al / | tr a-z A-Z)
```

Silence means pass. The only line that legitimately differs is `PROC`, because the
process count changes between the two runs — which is exactly why the autograder
filters it out before comparing.

Then run the real thing:

```bash
./run_autograder
```

It needs valgrind, which is not installed in this WSL yet:

```bash
sudo apt install valgrind
```

### Grading

The autograder script is **34 / 33 / 33**, which is *not* the 34/66 split the course
webpage lists:

| points | test |
|--------|------|
| 34 | compiles (`make -j` exits 0, under a 1-minute timeout) |
| 33 | output matches the real pipeline after filtering the `PROC` line |
| 33 | valgrind clean — `--leak-check=full --error-exitcode=100` |

The leak check is a third of the grade, so unclosed descriptors and stray allocations
actually cost you.

---

## Part 6 — Failure modes

| symptom | cause |
|---------|-------|
| hangs forever, no output | somebody still holds the **write** end. Usually `main` forgot `close(fds[1])`, or the `tr` child forgot it. |
| hangs *after* printing everything | same thing — `tr` printed what it had but never got EOF |
| output is lowercase | `tr` never got the data; check `dup2` in the producer |
| nothing happens at all | `exec` called in `main` instead of in a child — your program became `ls` and died |
| `ls` output goes to the terminal, not the pipe | `dup2` args backwards. It is `dup2(from, to)`; the **second** one is overwritten |
| prompt prints in the middle of output | missing `waitpid` |
| `exec` fails silently | forgot the `NULL` terminator on the argument list |

### Two experiments worth doing

**1. Make it hang on purpose.** Comment out the two `close(fds[...])` calls in `main`,
rebuild, run. It prints everything, then hangs forever. Ctrl-C to kill it. Put them
back and it exits cleanly. This single experiment teaches the EOF rule better than any
explanation.

**2. Look at your own program's list.** Add `sleep(30);` right after the two `close`
calls in `main`, rebuild, run it in one WSL terminal, and in another run:

```bash
ls -l /proc/$(pgrep -x ls)/fd
ls -l /proc/$(pgrep -x tr)/fd
```

You will see `ls` with fd 1 → some `pipe:[N]` and `tr` with fd 0 → the **same**
`pipe:[N]`. Your own matching numbers, exactly like the `tail | tr` output in Part 1.
This is what the handout's `fdgrok.ps1` prints.

---

## Part 7 — Optional: the same thing in Rust

Section 5 of the handout. Same five calls, via the `nix` crate:
`nix::unistd::{pipe, fork, dup2, close, execvp}` and `nix::sys::wait::waitpid`.

Two differences from C:

- Arguments must be `CString` — C-style NUL-terminated strings.
- Every call returns a `Result`, so you are forced to handle failure instead of
  silently ignoring a return value the way C lets you.

`fork()` is `unsafe` in Rust, for the reason the Microsoft paper linked in the handout
argues: in a multithreaded program the child inherits only the calling thread, so any
lock held by another thread at fork time stays locked forever in the child.
