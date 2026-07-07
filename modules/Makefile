CC = gcc
CFLAGS = -shared -fPIC -O2 -I../include
LDFLAGS = -lm

MODULES = panda.so
SRCS = $(MODULES:.so=.c)

.PHONY: all clean

all: $(MODULES)

panda.so: panda.c
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)

clean:
	rm -f $(MODULES)
