CFLAGS = -Wall -Wextra -g -std=gnu++20
CC = g++

random: random.o
	$(CC) $(CFLAGS) -o $@ random.o

clean:
	-rm -f *.o random

.cpp.o:
	$(CC) $(CFLAGS) -c $<
