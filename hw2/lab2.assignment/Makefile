opt: shell.opt
dbg: shell.dbg
san: shell.san

CCFLAGS = -std=c18 -Wall -Wextra -Wpedantic -Wconversion -fanalyzer

shell.opt: CCFLAGS += -Ofast -flto=auto
shell.opt: shell.c
	$(CC) $(CCFLAGS) -o $@ $^

shell.dbg: CCFLAGS += -Og -ggdb3
shell.dbg: shell.c
	$(CC) $(CCFLAGS) -o $@ $^

shell.san: CCFLAGS += -Og -ggdb3 -fsanitize=address
shell.san: shell.c
	$(CC) $(CCFLAGS) -o $@ $^

.PHONY: clean
clean:
	rm -f shell.opt shell.dbg shell.san
